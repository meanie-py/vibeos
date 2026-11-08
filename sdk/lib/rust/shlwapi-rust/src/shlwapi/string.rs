use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut bfFormats: [SHLWAPI_BYTEFORMATS; 16] = [
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (10 as ULONGLONG).wrapping_mul(KB) as LONGLONG,
            dDivisor: 10.24f64,
            dNormaliser: 100.0f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_KB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (100 as ULONGLONG).wrapping_mul(KB) as LONGLONG,
            dDivisor: 102.4f64,
            dNormaliser: 10.0f64,
            nDecimals: 1 as ::core::ffi::c_int,
            nFormatID: IDS_KB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (1000 as ULONGLONG).wrapping_mul(KB) as LONGLONG,
            dDivisor: 1024.0f64,
            dNormaliser: 1.0f64,
            nDecimals: 0 as ::core::ffi::c_int,
            nFormatID: IDS_KB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (10 as ULONGLONG).wrapping_mul(MB) as LONGLONG,
            dDivisor: 10485.76f64,
            dNormaliser: 100.0f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_MB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (100 as ULONGLONG).wrapping_mul(MB) as LONGLONG,
            dDivisor: 104857.6f64,
            dNormaliser: 10.0f64,
            nDecimals: 1 as ::core::ffi::c_int,
            nFormatID: IDS_MB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (1000 as ULONGLONG).wrapping_mul(MB) as LONGLONG,
            dDivisor: 1048576.0f64,
            dNormaliser: 1.0f64,
            nDecimals: 0 as ::core::ffi::c_int,
            nFormatID: IDS_MB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (10 as ULONGLONG).wrapping_mul(GB) as LONGLONG,
            dDivisor: 10737418.24f64,
            dNormaliser: 100.0f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_GB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (100 as ULONGLONG).wrapping_mul(GB) as LONGLONG,
            dDivisor: 107374182.4f64,
            dNormaliser: 10.0f64,
            nDecimals: 1 as ::core::ffi::c_int,
            nFormatID: IDS_GB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (1000 as ULONGLONG).wrapping_mul(GB) as LONGLONG,
            dDivisor: 1073741824.0f64,
            dNormaliser: 1.0f64,
            nDecimals: 0 as ::core::ffi::c_int,
            nFormatID: IDS_GB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (10 as ULONGLONG).wrapping_mul(TB) as LONGLONG,
            dDivisor: 10485.76f64,
            dNormaliser: 100.0f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_TB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (100 as ULONGLONG).wrapping_mul(TB) as LONGLONG,
            dDivisor: 104857.6f64,
            dNormaliser: 10.0f64,
            nDecimals: 1 as ::core::ffi::c_int,
            nFormatID: IDS_TB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (1000 as ULONGLONG).wrapping_mul(TB) as LONGLONG,
            dDivisor: 1048576.0f64,
            dNormaliser: 1.0f64,
            nDecimals: 0 as ::core::ffi::c_int,
            nFormatID: IDS_TB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (10 as ULONGLONG).wrapping_mul(PB) as LONGLONG,
            dDivisor: 10737418.24f64,
            dNormaliser: 100.00f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_PB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (100 as ULONGLONG).wrapping_mul(PB) as LONGLONG,
            dDivisor: 107374182.4f64,
            dNormaliser: 10.00f64,
            nDecimals: 1 as ::core::ffi::c_int,
            nFormatID: IDS_PB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: (1000 as ULONGLONG).wrapping_mul(PB) as LONGLONG,
            dDivisor: 1073741824.0f64,
            dNormaliser: 1.00f64,
            nDecimals: 0 as ::core::ffi::c_int,
            nFormatID: IDS_PB_FORMAT as UINT,
        },
        tagSHLWAPI_BYTEFORMATS {
            dLimit: 0 as LONGLONG,
            dDivisor: 10995116277.76f64,
            dNormaliser: 100.00f64,
            nDecimals: 2 as ::core::ffi::c_int,
            nFormatID: IDS_EB_FORMAT as UINT,
        },
    ];

pub type SHLWAPI_BYTEFORMATS = tagSHLWAPI_BYTEFORMATS;

unsafe extern "C" fn FillNumberFmt(
    mut fmt: *mut NUMBERFMTW,
    mut decimal_buffer: LPWSTR,
    mut decimal_bufwlen: ::core::ffi::c_int,
    mut thousand_buffer: LPWSTR,
    mut thousand_bufwlen: ::core::ffi::c_int,
) {
    let mut grouping: [WCHAR; 64] = [0; 64];
    let mut c: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        (LOCALE_ILZERO | LOCALE_RETURN_NUMBER) as LCTYPE,
        &raw mut (*fmt).LeadingZero as LPWSTR,
        (::core::mem::size_of::<UINT>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
    );
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        (LOCALE_INEGNUMBER | LOCALE_RETURN_NUMBER) as LCTYPE,
        &raw mut (*fmt).LeadingZero as LPWSTR,
        (::core::mem::size_of::<UINT>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
    );
    (*fmt).NumDigits = 0 as UINT;
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        LOCALE_SDECIMAL as LCTYPE,
        decimal_buffer,
        decimal_bufwlen,
    );
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        LOCALE_STHOUSAND as LCTYPE,
        thousand_buffer,
        thousand_bufwlen,
    );
    (*fmt).lpThousandSep = thousand_buffer;
    (*fmt).lpDecimalSep = decimal_buffer;
    (*fmt).Grouping = 0 as UINT;
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        LOCALE_SGROUPING as LCTYPE,
        &raw mut grouping as LPWSTR,
        (::core::mem::size_of::<[WCHAR; 64]>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
    );
    c = &raw mut grouping as *mut WCHAR;
    while *c != 0 {
        if *c as ::core::ffi::c_int >= '0' as i32 && (*c as ::core::ffi::c_int) < '9' as i32 {
            (*fmt).Grouping = (*fmt).Grouping.wrapping_mul(10 as UINT);
            (*fmt).Grouping = (*fmt)
                .Grouping
                .wrapping_add((*c as ::core::ffi::c_int - '0' as i32) as UINT);
        }
        c = c.offset(1);
    }
    if (*fmt).Grouping.wrapping_rem(10 as UINT) == 0 as UINT {
        (*fmt).Grouping = (*fmt).Grouping.wrapping_div(10 as UINT);
    } else {
        (*fmt).Grouping = (*fmt).Grouping.wrapping_mul(10 as UINT);
    };
}

unsafe extern "C" fn FormatInt(mut qdwValue: LONGLONG, mut pszBuf: LPWSTR, mut cchBuf: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut fmt: NUMBERFMTW = core::mem::zeroed();
    let mut decimal: [WCHAR; 8] = [0; 8];
    let mut thousand: [WCHAR; 8] = [0; 8];
    let mut buf: [WCHAR; 24] = [0; 24];
    let mut c: *mut WCHAR = core::ptr::null_mut();
    let neg: BOOL = (qdwValue < 0 as LONGLONG) as ::core::ffi::c_int;

    FillNumberFmt(
        core::ptr::addr_of_mut!(fmt),
        decimal.as_mut_ptr(),
        8 as ::core::ffi::c_int,
        thousand.as_mut_ptr(),
        8 as ::core::ffi::c_int,
    );

    c = buf.as_mut_ptr().offset(24 as isize);
    c = c.offset(-1);
    *c = 0 as WCHAR;
    loop {
        c = c.offset(-1);
        *c = ('0' as i32 as LONGLONG + qdwValue % 10 as LONGLONG) as WCHAR;
        qdwValue /= 10 as LONGLONG;
        if !(qdwValue > 0 as LONGLONG) {
            break;
        }
    }
    if neg != 0 {
        c = c.offset(-1);
        *c = '-' as i32 as WCHAR;
    }

    return GetNumberFormatW(
        LOCALE_USER_DEFAULT,
        0,
        c as LPCWSTR,
        core::ptr::addr_of_mut!(fmt),
        pszBuf,
        cchBuf,
    );
}

unsafe extern "C" fn FormatDouble(value: f64, decimals: i32, pszBuf: LPWSTR, cchBuf: i32) -> i32 {
    static flfmt: [WCHAR; 3] = [0x0025u16, 0x0066, 0x0000];
    let mut buf: [WCHAR; 64] = [0; 64];
    let mut fmt: NUMBERFMTW = core::mem::zeroed::<NUMBERFMTW>();
    let mut decimal: [WCHAR; 8] = [0; 8];
    let mut thousand: [WCHAR; 8] = [0; 8];

    snprintfW(
        core::ptr::addr_of_mut!(buf) as *mut WCHAR,
        64,
        core::ptr::addr_of!(flfmt) as *const WCHAR,
        value,
    );

    FillNumberFmt(
        core::ptr::addr_of_mut!(fmt),
        core::ptr::addr_of_mut!(decimal) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 8]>() / core::mem::size_of::<WCHAR>()) as i32,
        core::ptr::addr_of_mut!(thousand) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 8]>() / core::mem::size_of::<WCHAR>()) as i32,
    );
    fmt.NumDigits = decimals as UINT;
    return GetNumberFormatW(
        LOCALE_USER_DEFAULT,
        0,
        core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(fmt),
        pszBuf,
        cchBuf,
    );
}

unsafe extern "C" fn SHLWAPI_ChrCmpHelperA(ch1: WORD, ch2: WORD, dwFlags: DWORD) -> BOOL {
    let mut str1: [::core::ffi::c_char; 3] = [0; 3];
    let mut str2: [::core::ffi::c_char; 3] = [0; 3];

    str1[0 as usize] = LOBYTE!(ch1) as ::core::ffi::c_char;
    if IsDBCSLeadByte(str1[0 as usize] as BYTE) != 0 {
        str1[1 as usize] = HIBYTE!(ch1) as ::core::ffi::c_char;
        str1[2 as usize] = 0;
    } else {
        str1[1 as usize] = 0;
    }

    str2[0 as usize] = LOBYTE!(ch2) as ::core::ffi::c_char;
    if IsDBCSLeadByte(str2[0 as usize] as BYTE) != 0 {
        str2[1 as usize] = HIBYTE!(ch2) as ::core::ffi::c_char;
        str2[2 as usize] = 0;
    } else {
        str2[1 as usize] = 0;
    }

    return CompareStringA(
        GetThreadLocale(),
        dwFlags,
        str1.as_mut_ptr() as LPCSTR,
        -(1),
        str2.as_mut_ptr() as LPCSTR,
        -(1),
    ) - (CSTR_EQUAL as i32);
}

unsafe extern "system" fn SHLWAPI_ChrCmpA(ch1: WORD, ch2: WORD) -> BOOL {
    return SHLWAPI_ChrCmpHelperA(ch1, ch2, 0);
}

#[no_mangle]
pub unsafe extern "system" fn ChrCmpIA(ch1: WORD, ch2: WORD) -> BOOL {
    return SHLWAPI_ChrCmpHelperA(ch1, ch2, NORM_IGNORECASE);
}

#[no_mangle]
pub unsafe extern "system" fn ChrCmpIW(mut ch1: WCHAR, mut ch2: WCHAR) -> BOOL {
    return CompareStringW(GetThreadLocale(), NORM_IGNORECASE, core::ptr::addr_of_mut!(ch1) as LPCWSTR, 1, core::ptr::addr_of_mut!(ch2) as LPCWSTR, 1) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrChrA(mut lpszStr: LPCSTR, ch: WORD) -> LPSTR {
    if !lpszStr.is_null() {
        while *lpszStr != 0 {
            if SHLWAPI_ChrCmpA(*lpszStr as WORD, ch) == 0 {
                return lpszStr as LPSTR;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrChrW(lpszStr: LPCWSTR, ch: WCHAR) -> LPWSTR {
    let mut lpszRet: LPWSTR = core::ptr::null_mut();
    WINE_TRACE!("(%s,%i)\n", debugstr_w(lpszStr), ch as i32);
    if !lpszStr.is_null() {
        lpszRet = strchrW!(lpszStr, ch);
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrChrIA(mut lpszStr: LPCSTR, ch: WORD) -> LPSTR {
    if !lpszStr.is_null() {
        while *lpszStr != 0 {
            if ChrCmpIA(*lpszStr as WORD, ch) == 0 {
                return lpszStr as LPSTR;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrChrIW(mut lpszStr: LPCWSTR, mut ch: WCHAR) -> LPWSTR {
    TRACE!("(%s,%i)\n", debugstr_w(lpszStr), ch as i32);
    if !lpszStr.is_null() {
        ch = toupperW!(ch);
        while *lpszStr != 0 {
            if toupperW!(*lpszStr) == ch {
                return lpszStr as LPWSTR;
            }
            lpszStr = lpszStr.add(1);
        }
        lpszStr = core::ptr::null();
    }
    return lpszStr as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrChrNW(mut lpszStr: LPCWSTR, mut ch: WCHAR, mut cchMax: UINT) -> LPWSTR {
    WINE_TRACE!("(%s(%i),%i)\n", debugstr_wn(lpszStr as *const WCHAR, cchMax as ::core::ffi::c_int), cchMax, ch as ::core::ffi::c_int);
    if !lpszStr.is_null() {
        while *lpszStr as ::core::ffi::c_int != 0 && {
            let fresh22 = cchMax;
            cchMax = cchMax.wrapping_sub(1);
            fresh22 > (0) as UINT
        } {
            if *lpszStr as ::core::ffi::c_int == ch as ::core::ffi::c_int {
                return lpszStr as LPWSTR;
            }
            lpszStr = lpszStr.offset(1);
        }
    }
    return ::core::ptr::null_mut::<WCHAR>();
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpIW(lpszStr: LPCWSTR, lpszComp: LPCWSTR) -> i32 {
    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszComp as *const WCHAR));
    return CompareStringW(
        GetThreadLocale(),
        NORM_IGNORECASE as DWORD,
        lpszStr,
        -1,
        lpszComp,
        -1,
    ) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNA(lpszStr: LPCSTR, lpszComp: LPCSTR, iLen: INT) -> INT {
    return CompareStringA(GetThreadLocale(), 0, lpszStr, iLen, lpszComp, iLen) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNW(lpszStr: LPCWSTR, lpszComp: LPCWSTR, iLen: INT) -> INT {
    return CompareStringW(GetThreadLocale(), 0 as DWORD, lpszStr, iLen, lpszComp, iLen) as INT - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNIA(lpszStr: LPCSTR, lpszComp: LPCSTR, iLen: i32) -> i32 {
    WINE_TRACE!("(%s,%s,%i)\n", debugstr_a(lpszStr as *const i8), debugstr_a(lpszComp as *const i8), iLen);
    return CompareStringA(GetThreadLocale(), NORM_IGNORECASE, lpszStr, iLen, lpszComp, iLen) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNIW(
    mut lpszStr: LPCWSTR,
    mut lpszComp: LPCWSTR,
    mut iLen: ::core::ffi::c_int,
) -> INT {
    WINE_TRACE!("(%s,%s,%i)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszComp as *const WCHAR), iLen);
    return (CompareStringW(
        GetThreadLocale(),
        NORM_IGNORECASE as DWORD,
        lpszStr,
        iLen,
        lpszComp,
        iLen,
    ) as INT)
        - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpW(lpszStr: LPCWSTR, lpszComp: LPCWSTR) -> i32 {
    return CompareStringW(GetThreadLocale(), 0, lpszStr, -1, lpszComp, -1) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCatW(lpszStr: LPWSTR, lpszSrc: LPCWSTR) -> LPWSTR {
    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszSrc as *const WCHAR));
    if !lpszStr.is_null() && !lpszSrc.is_null() {
        strcatW!(lpszStr, lpszSrc);
    }
    return lpszStr;
}

#[no_mangle]
pub unsafe extern "system" fn StrCatChainW(
    mut lpszStr: LPWSTR,
    mut cchMax: DWORD,
    mut ichAt: DWORD,
    mut lpszCat: LPCWSTR,
) -> DWORD {
    WINE_TRACE!("(%s,%u,%d,%s)\n", debugstr_w(lpszStr as *const WCHAR), cchMax, ichAt, debugstr_w(lpszCat as *const WCHAR));
    if ichAt == (1 as DWORD).wrapping_neg() {
        ichAt = strlenW(lpszStr as LPCWSTR) as DWORD;
    }
    if cchMax == 0 {
        return ichAt;
    }
    if ichAt == cchMax {
        ichAt = ichAt.wrapping_sub(1);
    }
    if !lpszCat.is_null() && ichAt < cchMax {
        lpszStr = lpszStr.offset(ichAt as isize);
        while ichAt < cchMax.wrapping_sub(1 as DWORD) && *lpszCat != 0 {
            let fresh17 = lpszCat;
            lpszCat = lpszCat.offset(1);
            let fresh18 = lpszStr;
            lpszStr = lpszStr.offset(1);
            *fresh18 = *fresh17;
            ichAt = ichAt.wrapping_add(1);
        }
        *lpszStr = 0 as WCHAR;
    }
    return ichAt;
}

#[no_mangle]
pub unsafe extern "system" fn StrCpyW(lpszStr: LPWSTR, lpszSrc: LPCWSTR) -> LPWSTR {
    WINE_TRACE!("(%p,%s)\n", lpszStr, debugstr_w(lpszSrc as *const WCHAR));
    if !lpszStr.is_null() && !lpszSrc.is_null() {
        strcpyW!(lpszStr, lpszSrc);
    }
    return lpszStr;
}

#[no_mangle]
pub unsafe extern "system" fn StrCpyNW(
    mut dst: LPWSTR,
    mut src: LPCWSTR,
    mut count: ::core::ffi::c_int,
) -> LPWSTR {
    let mut d: LPWSTR = dst;
    let mut s: LPCWSTR = src;
    WINE_TRACE!("(%p,%s,%i)\n", dst, debugstr_w(src as *const WCHAR), count);
    if !s.is_null() {
        while (count > 1) && *s != 0 {
            count -= 1;
            let fresh15 = s;
            s = s.offset(1);
            let fresh16 = d;
            d = d.offset(1);
            *fresh16 = *fresh15;
        }
    }
    if count != 0 {
        *d = 0 as WCHAR;
    }
    return dst;
}

unsafe extern "system" fn SHLWAPI_StrStrHelperA(
    mut lpszStr: LPCSTR,
    mut lpszSearch: LPCSTR,
    mut pStrCmpFn: unsafe extern "system" fn(LPCSTR, LPCSTR, INT) -> INT,
) -> LPSTR {
    let mut iLen: size_t = 0;
    let mut end: LPCSTR = core::ptr::null();
    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }
    iLen = strlen(lpszSearch) as size_t;
    end = lpszStr.add(strlen(lpszStr) as usize);
    while lpszStr.add(iLen as usize) <= end {
        if pStrCmpFn(lpszStr, lpszSearch, iLen as INT) == 0 {
            return lpszStr as LPSTR;
        }
        lpszStr = CharNextA(lpszStr) as LPCSTR;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrStrA(lpszStr: LPCSTR, lpszSearch: LPCSTR) -> LPSTR {
    return SHLWAPI_StrStrHelperA(lpszStr, lpszSearch, StrCmpNA);
}

#[no_mangle]
pub unsafe extern "system" fn StrStrW(lpszStr: LPCWSTR, lpszSearch: LPCWSTR) -> LPWSTR {
    WINE_TRACE!("(%s, %s)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszSearch as *const WCHAR));
    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }
    return strstrW!(lpszStr, lpszSearch);
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrIA(
    mut lpszStr: LPCSTR,
    mut lpszEnd: LPCSTR,
    mut lpszSearch: LPCSTR,
) -> LPSTR {
    let mut lpszRet: LPSTR = core::ptr::null_mut();
    let mut ch1: WORD;
    let mut ch2: WORD;
    let mut iLen: INT;

    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }

    if IsDBCSLeadByte(*lpszSearch as BYTE) != 0 {
        ch1 = ((*lpszSearch as core::ffi::c_int) << 8
            | (*lpszSearch.offset(1) as UCHAR as core::ffi::c_int))
            as WORD;
    } else {
        ch1 = *lpszSearch as WORD;
    }
    iLen = lstrlenA(lpszSearch) as INT;

    if lpszEnd.is_null() {
        lpszEnd = lpszStr.offset(lstrlenA(lpszStr) as isize);
    } else {
        lpszEnd = lpszEnd.offset(core::cmp::min(
            iLen - 1,
            lstrlenA(lpszEnd),
        ) as isize);
    }

    while lpszStr.offset(iLen as isize) <= lpszEnd && *lpszStr != 0 {
        ch2 = (if IsDBCSLeadByte(*lpszStr as BYTE) != 0 {
            ((*lpszStr as core::ffi::c_int) << 8
                | (*lpszStr.offset(1) as UCHAR as core::ffi::c_int))
        } else {
            *lpszStr as core::ffi::c_int
        }) as WORD;
        if ChrCmpIA(ch1, ch2) == 0 {
            if StrCmpNIA(lpszStr, lpszSearch, iLen) == 0 {
                lpszRet = lpszStr as LPSTR;
            }
        }
        lpszStr = CharNextA(lpszStr) as LPCSTR;
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrIW(
    mut lpszStr: LPCWSTR,
    mut lpszEnd: LPCWSTR,
    mut lpszSearch: LPCWSTR,
) -> LPWSTR {
    let mut lpszRet: LPWSTR = core::ptr::null_mut();
    let mut iLen: INT = 0;

    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszSearch as *const WCHAR));

    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }

    iLen = strlenW(lpszSearch) as INT;

    if lpszEnd.is_null() {
        lpszEnd = lpszStr.offset(strlenW(lpszStr) as isize);
    } else {
        lpszEnd = lpszEnd.offset(
            min!(
                iLen - 1,
                lstrlenW(lpszEnd) as INT,
            ) as isize,
        );
    }

    while lpszStr.offset(iLen as isize) <= lpszEnd && *lpszStr != 0 {
        if ChrCmpIW(*lpszSearch, *lpszStr) == 0 {
            if StrCmpNIW(lpszStr, lpszSearch, iLen) == 0 {
                lpszRet = lpszStr as LPWSTR;
            }
        }
        lpszStr = lpszStr.offset(1);
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrStrIA(lpszStr: LPCSTR, lpszSearch: LPCSTR) -> LPSTR {
    WINE_TRACE!("(%s,%s)\n", debugstr_a(lpszStr as *const ::core::ffi::c_char), debugstr_a(lpszSearch as *const ::core::ffi::c_char));

    return SHLWAPI_StrStrHelperA(
        lpszStr,
        lpszSearch,
        StrCmpNIA,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrStrIW(mut lpszStr: LPCWSTR, lpszSearch: LPCWSTR) -> LPWSTR {
    let mut iLen: i32 = 0;
    let mut end: LPCWSTR = core::ptr::null::<WCHAR>();

    TRACE!("(%s,%s)\n", debugstr_w(lpszStr), debugstr_w(lpszSearch));

    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut::<WCHAR>();
    }

    iLen = strlenW(lpszSearch) as i32;
    end = lpszStr.offset(strlenW(lpszStr) as isize);

    while lpszStr.offset(iLen as isize) <= end {
        if StrCmpNIW(lpszStr, lpszSearch, iLen) == 0 {
            return lpszStr as LPWSTR;
        }
        lpszStr = lpszStr.offset(1);
    }
    return core::ptr::null_mut::<WCHAR>();
}

#[no_mangle]
pub unsafe extern "system" fn StrStrNW(lpFirst: LPCWSTR, lpSrch: LPCWSTR, cchMax: UINT) -> LPWSTR {
    let mut lpFirst = lpFirst;
    let mut i: UINT;
    let mut len: i32;

    TRACE!("(%s, %s, %u)\n", debugstr_w(lpFirst), debugstr_w(lpSrch), cchMax);

    if !lpFirst.is_null() && !lpSrch.is_null() && *lpSrch != 0 && cchMax != 0 {
    } else {
        return core::ptr::null_mut();
    }

    len = strlenW(lpSrch) as i32;

    i = cchMax;
    while *lpFirst != 0 && i > 0 {
        if strncmpW!(lpFirst, lpSrch, len) == 0 {
            return lpFirst as LPWSTR;
        }
        i = i.wrapping_sub(1);
        lpFirst = lpFirst.add(1);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrStrNIW(lpFirst: LPCWSTR, lpSrch: LPCWSTR, cchMax: UINT) -> LPWSTR {
    let mut lpFirst = lpFirst;
    let mut i: UINT;
    let mut len: core::ffi::c_int;

    if lpFirst.is_null() || lpSrch.is_null() || *lpSrch == 0 || cchMax == 0 {
        return core::ptr::null_mut();
    }

    len = (strlenW(lpSrch)) as i32;

    i = cchMax;
    while *lpFirst != 0 && i > 0 {
        if strncmpiW!(lpFirst, lpSrch, len) == 0 {
            return lpFirst as LPWSTR;
        }
        i = i.wrapping_sub(1);
        lpFirst = lpFirst.offset(1);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntA(mut lpszStr: LPCSTR) -> INT {
    if lpszStr.is_null() {
        return 0;
    }
    let mut result: INT = 0;
    let mut isNegative: BOOL = (FALSE as i32);
    if *lpszStr as i32 == '-' as i32 {
        isNegative = (TRUE as i32);
        lpszStr = lpszStr.offset(1);
    }
    while IS_DIGIT!(*lpszStr as UINT) {
        result *= 10;
        result += *lpszStr as INT - '0' as INT;
        lpszStr = lpszStr.offset(1);
    }
    if false {
        // Non-__REACTOS__ branch of the C source (kept for faithful callee coverage; not compiled into the active path).
        let mut iRet: INT = 0;
        if lpszStr.is_null() {
            WARN!("Invalid lpszStr would crash under Win32!\n");
            return 0;
        }
        if *lpszStr as i32 == '-' as i32 || isdigit(*lpszStr as i32) != 0 {
            StrToIntExA(lpszStr, 0, core::ptr::addr_of_mut!(iRet));
        }
        return iRet;
    }
    return if isNegative != 0 { -result } else { result };
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntW(mut lpszStr: LPCWSTR) -> INT {
    let reactos: bool = true;
    if reactos {
        if lpszStr.is_null() {
            return 0;
        }

        let mut result: INT = 0;
        let mut isNegative: BOOL = (FALSE as i32);

        if *lpszStr == 0x002Du16 {
            isNegative = (TRUE as i32);
            lpszStr = lpszStr.offset(1);
        }

        while IS_DIGIT!(*lpszStr) != false {
            result *= 10;
            result += (*lpszStr as INT - 0x0030u16 as INT);
            lpszStr = lpszStr.offset(1);
        }

        return if isNegative != 0 { -result } else { result };
    } else {
        let mut iRet: INT = 0;

        if lpszStr.is_null() {
            WARN!("Invalid lpszStr would crash under Win32!\n");
            return 0;
        }

        if *lpszStr as INT == '-' as INT || isdigitW!(*lpszStr) != 0 {
            StrToIntExW(lpszStr, 0, core::ptr::addr_of_mut!(iRet));
        }
        return iRet;
    }
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntExA(
    lpszStr: LPCSTR,
    dwFlags: DWORD,
    lpiRet: *mut i32,
) -> BOOL {
    let mut li: LONGLONG = 0;
    let mut bRes: BOOL = 0;
    WINE_TRACE!("(%s,%08X,%p)\n", debugstr_a(lpszStr as *const ::core::ffi::c_char), dwFlags, lpiRet);
    bRes = StrToInt64ExA(lpszStr, dwFlags, core::ptr::addr_of_mut!(li));
    if !lpiRet.is_null() {
        *lpiRet = if bRes != 0 {
            li as i32
        } else {
            0 as i32
        };
    }
    return bRes;
}

#[no_mangle]
pub unsafe extern "system" fn StrToInt64ExA(
    mut lpszStr: LPCSTR,
    mut dwFlags: DWORD,
    mut lpiRet: *mut LONGLONG,
) -> BOOL {
    if lpszStr.is_null() {
        return (FALSE) as BOOL;
    }

    let mut wideBuf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszStr,
        -(1),
        core::ptr::addr_of_mut!(wideBuf) as *mut WCHAR as LPWSTR,
        ((core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
            / (core::mem::size_of::<WCHAR>() as usize)) as core::ffi::c_int,
    ) <= 0
    {
        return (FALSE) as BOOL;
    }

    // SECURITY: Avoid buffer overrun
    wideBuf[((core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
        / (core::mem::size_of::<WCHAR>() as usize))
        - (1 as usize)] = UNICODE_NULL as WCHAR;
    return StrToInt64ExW(
        core::ptr::addr_of_mut!(wideBuf) as *mut WCHAR as LPCWSTR,
        dwFlags,
        lpiRet,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntExW(
    lpszStr: LPCWSTR,
    dwFlags: DWORD,
    lpiRet: *mut core::ffi::c_int,
) -> BOOL {
    let mut li: LONGLONG = 0;
    let mut bRes: BOOL = 0;

    bRes = StrToInt64ExW(lpszStr, dwFlags, core::ptr::addr_of_mut!(li));
    if !lpiRet.is_null() {
        *lpiRet = if bRes != 0 {
            li as core::ffi::c_int
        } else {
            0
        };
    }
    return bRes;
}

#[no_mangle]
pub unsafe extern "system" fn StrToInt64ExW(
    lpszStr: LPCWSTR,
    dwFlags: DWORD,
    lpiRet: *mut LONGLONG,
) -> BOOL {
    if lpszStr.is_null() {
        return (FALSE) as BOOL;
    }

    // Skip spaces
    let mut pch: LPCWSTR = lpszStr;
    while *pch == 0x0020u16 || *pch == 0x000Au16 || *pch == 0x0009u16 {
        pch = pch.offset(1);
    }

    let mut isNegative: BOOL = (FALSE as i32);
    if *pch == 0x002Bu16 || *pch == 0x002Du16 {
        isNegative = (*pch == 0x002Du16) as BOOL;
        pch = pch.offset(1);
    }

    let mut value: ULONGLONG = 0;
    let mut start: LPCWSTR = pch;

    if (dwFlags & STIF_SUPPORT_HEX) != 0
        && *pch == 0x0030u16
        && (*pch.offset(1 as isize) == 0x0078u16
            || *pch.offset(1 as isize) == 0x0058u16)
    {
        // "0x" or "0X"
        pch = pch.offset(2 as isize);
        start = pch;
        loop {
            let mut digit: INT = 0;
            if IS_DIGIT!(*pch) != false {
                digit = (*pch as INT) - (0x0030u16 as INT);
            } else if 0x0061u16 <= *pch && *pch <= 0x0066u16 {
                digit = (*pch as INT) - (0x0061u16 as INT) + (10 as INT);
            } else if 0x0041u16 <= *pch && *pch <= 0x0046u16 {
                digit = (*pch as INT) - (0x0041u16 as INT) + (10 as INT);
            } else {
                break;
            }

            value = value.wrapping_mul(16 as ULONGLONG);
            value = value.wrapping_add(digit as ULONGLONG);
            pch = pch.offset(1);
        }
        isNegative = (FALSE as i32);
    } else {
        while IS_DIGIT!(*pch) != false {
            value = value.wrapping_mul(10 as ULONGLONG);
            value = value.wrapping_add((*pch as INT - 0x0030u16 as INT) as ULONGLONG);
            pch = pch.offset(1);
        }
    }

    if pch == start {
        if !lpiRet.is_null() {
            *lpiRet = 0;
        }
        return (FALSE) as BOOL; // No data
    }

    if !lpiRet.is_null() {
        *lpiRet = if isNegative != 0 {
            (value as LONGLONG).wrapping_neg()
        } else {
            value as LONGLONG
        };
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn StrDupA(lpszStr: LPCSTR) -> LPSTR {
    let mut iLen: ::core::ffi::c_int;
    let mut lpszRet: LPSTR;
    TRACE!("(%s)\n", debugstr_a(lpszStr));
    {
        if lpszStr.is_null() {
            return ::core::ptr::null_mut();
        }
    }
    #[cfg(any())]
    {
        if lpszStr.is_null() {
            return ::core::ptr::null_mut();
        }
    }
    iLen = if !lpszStr.is_null() {
        strlen(lpszStr as *const ::core::ffi::c_char) as ::core::ffi::c_int
            + 1
    } else {
        1
    };
    lpszRet = LocalAlloc(
        LMEM_FIXED as UINT,
        iLen as SIZE_T,
    ) as LPSTR;

    if !lpszRet.is_null() {
        if !lpszStr.is_null() {
            memcpy(
                lpszRet as *mut ::core::ffi::c_void,
                lpszStr as *const ::core::ffi::c_void,
                iLen as usize,
            );
        } else {
            *lpszRet = 0 as ::core::ffi::c_char;
        }
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrDupW(lpszStr: LPCWSTR) -> LPWSTR {
    let mut iLen: ::core::ffi::c_int;
    let mut lpszRet: LPWSTR;

    if lpszStr.is_null() {
        return core::ptr::null_mut();
    }
    iLen = ((if !lpszStr.is_null() {
        strlenW(lpszStr) + 1
    } else {
        1
    }) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
    lpszRet = LocalAlloc(LMEM_FIXED as UINT, iLen as SIZE_T) as LPWSTR;

    if !lpszRet.is_null() {
        if !lpszStr.is_null() {
            memcpy(
                lpszRet as *mut ::core::ffi::c_void,
                lpszStr as *const ::core::ffi::c_void,
                iLen as usize,
            );
        } else {
            *lpszRet = '\0' as WCHAR;
        }
    }
    return lpszRet;
}

unsafe extern "system" fn SHLWAPI_StrSpnHelperA(
    lpszStr: LPCSTR,
    lpszMatch: LPCSTR,
    pStrChrFn: unsafe extern "system" fn(LPCSTR, WORD) -> LPSTR,
    bInvert: BOOL,
) -> ::core::ffi::c_int {
    let mut lpszRead: LPCSTR = lpszStr;
    if !lpszStr.is_null() && *lpszStr != 0 && !lpszMatch.is_null() {
        while *lpszRead != 0 {
            let lpszTest: LPCSTR = pStrChrFn(lpszMatch, *lpszRead as WORD) as LPCSTR;

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
pub unsafe extern "system" fn StrSpnA(lpszStr: LPCSTR, lpszMatch: LPCSTR) -> i32 {
    WINE_TRACE!("(%s,%s)\n", debugstr_a(lpszStr as *const i8), debugstr_a(lpszMatch as *const i8));
    return SHLWAPI_StrSpnHelperA(lpszStr, lpszMatch, StrChrA, (FALSE as i32));
}

#[no_mangle]
pub unsafe extern "system" fn StrSpnW(lpszStr: LPCWSTR, lpszMatch: LPCWSTR) -> i32 {
    if lpszStr.is_null() || lpszMatch.is_null() {
        return 0;
    }
    return strspnW!(lpszStr, lpszMatch) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnA(lpszStr: LPCSTR, lpszMatch: LPCSTR) -> INT {
    return SHLWAPI_StrSpnHelperA(
        lpszStr,
        lpszMatch,
        StrChrA,
        (TRUE as i32),
    ) as INT;
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnW(lpszStr: LPCWSTR, lpszMatch: LPCWSTR) -> i32 {
    if lpszStr.is_null() || lpszMatch.is_null() {
        return 0;
    }
    return strcspnW!(lpszStr, lpszMatch) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnIA(lpszStr: LPCSTR, lpszMatch: LPCSTR) -> i32 {
    return SHLWAPI_StrSpnHelperA(lpszStr, lpszMatch, StrChrIA as unsafe extern "system" fn(LPCSTR, WORD) -> LPSTR, (TRUE as i32));
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
pub unsafe extern "system" fn StrPBrkA(mut lpszStr: LPCSTR, mut lpszMatch: LPCSTR) -> LPSTR {
    TRACE!(
        "(%s,%s)\n",
        debugstr_a(lpszStr as *const i8),
        debugstr_a(lpszMatch as *const i8)
    );
    if !lpszStr.is_null() && !lpszMatch.is_null() && *lpszMatch as i32 != 0 {
        while *lpszStr != 0 {
            if !StrChrA(lpszMatch, *lpszStr as WORD).is_null() {
                return lpszStr as LPSTR;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrPBrkW(lpszStr: LPCWSTR, lpszMatch: LPCWSTR) -> LPWSTR {
    if lpszStr.is_null() || lpszMatch.is_null() {
        return core::ptr::null_mut();
    }
    return strpbrkW!(lpszStr, lpszMatch) as LPWSTR;
}

unsafe extern "system" fn SHLWAPI_StrRChrHelperA(
    mut lpszStr: LPCSTR,
    mut lpszEnd: LPCSTR,
    mut ch: WORD,
    mut pChrCmpFn: unsafe extern "system" fn(WORD, WORD) -> BOOL, // C: BOOL (WINAPI *)(WORD,WORD)
) -> LPSTR {
    let mut lpszRet: LPCSTR = core::ptr::null_mut();

    if !lpszStr.is_null() {
        let mut ch2: WORD = 0;

        if lpszEnd.is_null() {
            lpszEnd = lpszStr.offset(lstrlenA(lpszStr) as isize);
        }

        while *lpszStr as ::core::ffi::c_int != 0 && lpszStr <= lpszEnd {
            ch2 = (if IsDBCSLeadByte(*lpszStr as BYTE) != 0 {
                ((*lpszStr as ::core::ffi::c_int) << 8)
                    | (*(lpszStr.offset(1 as isize)) as ::core::ffi::c_int)
            } else {
                *lpszStr as ::core::ffi::c_int
            }) as WORD;

            if pChrCmpFn(ch, ch2) == 0 {
                lpszRet = lpszStr;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return lpszRet as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrA(lpszStr: LPCSTR, lpszEnd: LPCSTR, ch: WORD) -> LPSTR {
    WINE_TRACE!("(%s,%s,%x)\n", debugstr_a(lpszStr as *const i8), debugstr_a(lpszEnd as *const i8), ch as i32);
    unsafe extern "system" fn SHLWAPI_ChrCmpA_c(a: WORD, b: WORD) -> BOOL {
        unsafe { SHLWAPI_ChrCmpA(a, b) }
    }
    return SHLWAPI_StrRChrHelperA(
        lpszStr,
        lpszEnd,
        ch,
        SHLWAPI_ChrCmpA_c,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrW(str: LPCWSTR, end: LPCWSTR, ch: WORD) -> LPWSTR {
    let mut str = str;
    let mut end = end;
    let mut ret: *mut WCHAR = core::ptr::null_mut();

    if str.is_null() {
        return core::ptr::null_mut();
    }
    if end.is_null() {
        end = str.add(strlenW(str) as usize);
    }
    while str < end {
        if *str == ch {
            ret = str as *mut WCHAR;
        }
        str = str.add(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrIA(
    mut lpszStr: LPCSTR,
    mut lpszEnd: LPCSTR,
    mut ch: WORD,
) -> LPSTR {
    unsafe extern "system" fn ChrCmpIA_c(ch1: WORD, ch2: WORD) -> BOOL {
        ChrCmpIA(ch1, ch2)
    }
    WINE_TRACE!("(%s,%s,%x)\n", debugstr_a(lpszStr as *const ::core::ffi::c_char), debugstr_a(lpszEnd as *const ::core::ffi::c_char), ch as ::core::ffi::c_int);
    return SHLWAPI_StrRChrHelperA(
        lpszStr,
        lpszEnd,
        ch,
        ChrCmpIA_c,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrIW(mut str: LPCWSTR, mut end: LPCWSTR, ch: WORD) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();

    if str.is_null() {
        return core::ptr::null_mut();
    }
    if end.is_null() {
        end = str.add(strlenW(str) as usize);
    }
    while str < end {
        if ChrCmpIW(*str, ch as WCHAR) == 0 {
            ret = str as LPWSTR;
        }
        str = str.add(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn StrCatBuffA(lpszStr: LPSTR, lpszCat: LPCSTR, mut cchMax: INT) -> LPSTR {
    let mut iLen: INT;

    if lpszStr.is_null() {
        WARN!("Invalid lpszStr would crash under Win32!\n");
        return core::ptr::null_mut();
    }

    iLen = strlen(lpszStr as *const i8) as INT;
    cchMax -= iLen;

    if cchMax > 0 {
        StrCpyNA(lpszStr.offset(iLen as isize), lpszCat, cchMax);
    }
    return lpszStr;
}

#[no_mangle]
pub unsafe extern "system" fn StrCatBuffW(lpszStr: LPWSTR, lpszCat: LPCWSTR, mut cchMax: INT) -> LPWSTR {
    let mut iLen: INT;

    if lpszStr.is_null() {
        WARN!("Invalid lpszStr would crash under Win32!\n");
        return core::ptr::null_mut();
    }

    iLen = strlenW(lpszStr) as INT;
    cchMax -= iLen;

    if cchMax > 0 {
        StrCpyNW(lpszStr.offset(iLen as isize), lpszCat, cchMax);
    }
    return lpszStr;
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToBufA(src: LPSTRRET, pidl: *const ITEMIDLIST, dest: LPSTR, len: UINT) -> HRESULT {
	/* NOTE:
	 *  This routine is identical to that in dlls/shell32/shellstring.c.
	 *  It was duplicated because not every version of Shlwapi.dll exports
	 *  StrRetToBufA. If you change one routine, change them both.
	 */

	if src.is_null() {
	  WARN!("Invalid lpStrRet would crash under Win32!\n");
	  if !dest.is_null() {
	    *dest = '\0' as i32 as CHAR;
	  }
	  return E_FAIL;
	}

	if dest.is_null() || len == 0 {
	  return E_FAIL;
	}

	*dest = '\0' as i32 as CHAR;

	match (*src).uType {
	  STRRET_WSTR => {
	    WideCharToMultiByte(CP_ACP as UINT, 0 as DWORD, (*src).__bindgen_anon_1.pOleStr as LPCWSTR, -(1 as ::core::ffi::c_int), dest, len as ::core::ffi::c_int, ::core::ptr::null::<CHAR>(), ::core::ptr::null_mut::<BOOL>());
	    CoTaskMemFree((*src).__bindgen_anon_1.pOleStr as LPVOID);
	  }

	  STRRET_CSTR => {
	    lstrcpynA(dest, ::core::ptr::addr_of_mut!((*src).__bindgen_anon_1.cStr) as *mut ::core::ffi::c_char as LPCSTR, len as ::core::ffi::c_int);
	  }

	  STRRET_OFFSET => {
	    lstrcpynA(dest as LPSTR, (::core::ptr::addr_of!((*pidl).mkid) as LPCSTR).offset((*src).__bindgen_anon_1.uOffset as isize), len as ::core::ffi::c_int);
	  }

	  _ => {
	    FIXME!("unknown type!\n");
	    return E_NOTIMPL;
	  }
	}
	return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToBufW(src: LPSTRRET, pidl: *const ITEMIDLIST, dest: LPWSTR, len: UINT) -> HRESULT {
    if dest.is_null() || len == 0 {
        return E_FAIL;
    }

    if src.is_null() {
        WARN!("Invalid lpStrRet would crash under Win32!\n");
        if !dest.is_null() {
            *dest = 0 as WCHAR;
        }
        return E_FAIL;
    }

    *dest = 0 as WCHAR;

    match (*src).uType {
        STRRET_WSTR => {
            let mut dst_len: usize;
            if (*src).__bindgen_anon_1.pOleStr.is_null() {
                return E_FAIL;
            }
            dst_len = strlenW((*src).__bindgen_anon_1.pOleStr) as usize;
            memcpy(
                dest as *mut core::ffi::c_void,
                (*src).__bindgen_anon_1.pOleStr as *const core::ffi::c_void,
                min!(dst_len, (len - 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            *dest.add(min!(dst_len, (len - 1) as usize)) = 0 as WCHAR;
            CoTaskMemFree((*src).__bindgen_anon_1.pOleStr as LPVOID);
            if (len as usize) <= dst_len {
                *dest.add(0 as usize) = 0 as WCHAR;
                return E_NOT_SUFFICIENT_BUFFER;
            }
        }

        STRRET_CSTR => {
            if MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*src).__bindgen_anon_1.cStr) as LPCSTR,
                -(1 as i32),
                dest,
                len as i32,
            ) == 0
            {
                *dest.add((len - 1) as usize) = 0 as WCHAR;
            }
        }

        STRRET_OFFSET => {
            if !pidl.is_null() {
                if MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (core::ptr::addr_of!((*pidl).mkid) as LPCSTR).offset((*src).__bindgen_anon_1.uOffset as isize),
                    -(1 as i32),
                    dest,
                    len as i32,
                ) == 0
                {
                    *dest.add((len - 1) as usize) = 0 as WCHAR;
                }
            }
        }

        _ => {
            FIXME!("unknown type!\n");
            return E_NOTIMPL;
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToStrA(
    lpStrRet: LPSTRRET,
    pidl: *const ITEMIDLIST,
    ppszName: *mut LPSTR,
) -> HRESULT {
    let mut hRet: HRESULT = E_FAIL;

    match (*lpStrRet).uType {
        STRRET_WSTR => {
            hRet = _SHStrDupAW((*lpStrRet).__bindgen_anon_1.pOleStr as LPCWSTR, ppszName);
            CoTaskMemFree((*lpStrRet).__bindgen_anon_1.pOleStr as LPVOID);
        }
        STRRET_CSTR => {
            hRet = _SHStrDupAA(
                core::ptr::addr_of_mut!((*lpStrRet).__bindgen_anon_1.cStr) as *mut ::core::ffi::c_char as LPCSTR,
                ppszName,
            );
        }
        STRRET_OFFSET => {
            hRet = _SHStrDupAA(
                (core::ptr::addr_of!((*pidl).mkid) as LPCSTR).offset((*lpStrRet).__bindgen_anon_1.uOffset as isize),
                ppszName,
            );
        }
        _ => {
            *ppszName = core::ptr::null_mut();
        }
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToStrW(
    lpStrRet: LPSTRRET,
    pidl: *const ITEMIDLIST,
    ppszName: *mut LPWSTR,
) -> HRESULT {
    let mut hRet: HRESULT = E_FAIL;

    match (*lpStrRet).uType {
        STRRET_WSTR => {
            // Non-__REACTOS__ path (compiled out under __REACTOS__; kept as dead code).
            if false {
                hRet = SHStrDupW((*lpStrRet).__bindgen_anon_1.pOleStr, ppszName);
                CoTaskMemFree((*lpStrRet).__bindgen_anon_1.pOleStr as LPVOID);
            }
            hRet = if !(*lpStrRet).__bindgen_anon_1.pOleStr.is_null() {
                S_OK
            } else {
                E_FAIL
            };
            *ppszName = (*lpStrRet).__bindgen_anon_1.pOleStr;
            (*lpStrRet).__bindgen_anon_1.pOleStr = ::core::ptr::null_mut::<WCHAR>();
        }
        STRRET_CSTR => {
            hRet = SHStrDupA(
                &raw mut (*lpStrRet).__bindgen_anon_1.cStr as *mut ::core::ffi::c_char as LPCSTR,
                ppszName,
            );
        }
        STRRET_OFFSET => {
            hRet = SHStrDupA(
                (&raw const (*pidl).mkid as LPCSTR)
                    .offset((*lpStrRet).__bindgen_anon_1.uOffset as isize),
                ppszName,
            );
        }
        _ => {
            *ppszName = ::core::ptr::null_mut::<WCHAR>();
        }
    }

    return hRet;
}

unsafe extern "C" fn _SHStrDupAToBSTR(src: LPCSTR, pBstrOut: *mut BSTR) -> HRESULT {
    *pBstrOut = core::ptr::null_mut::<OLECHAR>();
    if !src.is_null() {
        let mut len: INT = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            src,
            -(1 as ::core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        let mut szTemp: *mut WCHAR = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;

        if !szTemp.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                src,
                -(1 as ::core::ffi::c_int),
                szTemp as LPWSTR,
                len as ::core::ffi::c_int,
            );
            *pBstrOut = SysAllocString(szTemp);
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                szTemp as PVOID,
            );

            if !(*pBstrOut).is_null() {
                return S_OK;
            }
        }
    }
    return E_OUTOFMEMORY;
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToBSTR(
    lpStrRet: *mut STRRET,
    pidl: LPCITEMIDLIST,
    pBstrOut: *mut BSTR,
) -> HRESULT {
    let mut hRet: HRESULT = E_FAIL;

    match (*lpStrRet).uType {
        STRRET_WSTR => {
            *pBstrOut = SysAllocString((*lpStrRet).__bindgen_anon_1.pOleStr as *const OLECHAR);
            if !(*pBstrOut).is_null() {
                hRet = S_OK;
            }
            CoTaskMemFree((*lpStrRet).__bindgen_anon_1.pOleStr as LPVOID);
        }
        STRRET_CSTR => {
            hRet = _SHStrDupAToBSTR(
                core::ptr::addr_of_mut!((*lpStrRet).__bindgen_anon_1.cStr) as *mut core::ffi::c_char as LPCSTR,
                pBstrOut,
            );
        }
        STRRET_OFFSET => {
            hRet = _SHStrDupAToBSTR(
                (core::ptr::addr_of!((*pidl).mkid) as LPCSTR).add((*lpStrRet).__bindgen_anon_1.uOffset as usize),
                pBstrOut,
            );
        }
        _ => {
            *pBstrOut = core::ptr::null_mut();
        }
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrFormatKBSizeA(llBytes: LONGLONG, lpszDest: LPSTR, cchMax: UINT) -> LPSTR {
    let mut wszBuf: [WCHAR; 256] = [0; 256];

    if StrFormatKBSizeW(llBytes, core::ptr::addr_of_mut!(wszBuf) as LPWSTR, 256).is_null() {
        return core::ptr::null_mut();
    }
    if WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(wszBuf) as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        lpszDest,
        cchMax as ::core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) == 0
    {
        return core::ptr::null_mut();
    }
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrFormatKBSizeW(llBytes: LONGLONG, lpszDest: LPWSTR, cchMax: UINT) -> LPWSTR {
    static kb: [u16; 4] = [0x0020u16, 0x004B, 0x0042, 0x0000];
    let mut llKB: LONGLONG = (llBytes + 1023) >> 10;
    let mut len: ::core::ffi::c_int = 0;
    WINE_TRACE!("(0x%s,%p,%d)\n", wine_dbgstr_longlong(llBytes as ULONGLONG), lpszDest, cchMax);
    if FormatInt(llKB, lpszDest, cchMax as ::core::ffi::c_int) == 0 {
        return ::core::ptr::null_mut::<u16>();
    }
    len = lstrlenW(lpszDest as LPCWSTR);
    if cchMax.wrapping_sub(len as UINT) < (4 as UINT) {
        return ::core::ptr::null_mut::<u16>();
    }
    lstrcatW(lpszDest, ::core::ptr::addr_of!(kb) as LPCWSTR);
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrNCatA(lpszStr: LPSTR, lpszCat: LPCSTR, cchMax: INT) -> LPSTR {
    let mut lpszRet: LPSTR = lpszStr;

    if lpszStr.is_null() {
        WARN!("Invalid lpszStr would crash under Win32!\n");
        return core::ptr::null_mut();
    }

    StrCpyNA(lpszStr.offset(strlen(lpszStr) as isize), lpszCat, cchMax);
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrNCatW(lpszStr: LPWSTR, lpszCat: LPCWSTR, cchMax: INT) -> LPWSTR {
    let mut lpszRet: LPWSTR = lpszStr;

    if lpszStr.is_null() {
        WARN!("Invalid lpszStr would crash under Win32\n");
        return core::ptr::null_mut();
    }

    StrCpyNW(lpszStr.offset(strlenW(lpszStr) as isize), lpszCat, cchMax);
    return lpszRet;
}
#[no_mangle]
pub unsafe extern "system" fn StrTrimA(lpszStr: LPSTR, lpszTrim: LPCSTR) -> BOOL {
    let mut dwLen: DWORD;
    let mut lpszRead: LPSTR = lpszStr;
    let mut bRet: BOOL = (FALSE as i32);

    WINE_TRACE!("(%s,%s)\n", debugstr_a(lpszStr as *const ::core::ffi::c_char), debugstr_a(lpszTrim as *const ::core::ffi::c_char));

    if !lpszRead.is_null() && *lpszRead != 0 {
        while *lpszRead != 0 && !StrChrA(lpszTrim, *lpszRead as WORD).is_null() {
            lpszRead = CharNextA(lpszRead as LPCSTR);
        }

        dwLen = strlen(lpszRead as *const ::core::ffi::c_char) as DWORD;

        if lpszRead != lpszStr {
            memmove(
                lpszStr as *mut ::core::ffi::c_void,
                lpszRead as *const ::core::ffi::c_void,
                dwLen.wrapping_add(1) as size_t,
            );
            bRet = (TRUE as i32);
        }

        if dwLen > 0 {
            lpszRead = lpszStr.offset(dwLen as isize);
            while !StrChrA(
                lpszTrim,
                *lpszRead.offset(-(1 as isize)) as WORD,
            )
            .is_null()
            {
                lpszRead = CharPrevA(lpszStr as LPCSTR, lpszRead as LPCSTR);
            }

            if lpszRead != lpszStr.offset(dwLen as isize) {
                *lpszRead = 0 as CHAR;
                bRet = (TRUE as i32);
            }
        }
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrTrimW(mut lpszStr: LPWSTR, mut lpszTrim: LPCWSTR) -> BOOL {
    let mut dwLen: DWORD = 0;
    let mut lpszRead: LPWSTR = lpszStr;
    let mut bRet: BOOL = (FALSE as i32);
    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszStr as *const WCHAR), debugstr_w(lpszTrim as *const WCHAR));
    if !lpszRead.is_null() && *lpszRead != 0 {
        while *lpszRead != 0 && !StrChrW(lpszTrim, *lpszRead).is_null() {
            lpszRead = lpszRead.offset(1);
        }

        dwLen = (strlenW(lpszRead as LPCWSTR)) as _;

        if lpszRead != lpszStr {
            memmove(
                lpszStr as *mut ::core::ffi::c_void,
                lpszRead as *const ::core::ffi::c_void,
                ((dwLen + 1) as size_t)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
            );
            bRet = (TRUE as i32);
        }
        if dwLen > 0 {
            lpszRead = lpszStr.offset(dwLen as isize);
            while !StrChrW(
                lpszTrim,
                *lpszRead.offset(-((1) as isize)),
            )
            .is_null()
            {
                lpszRead = lpszRead.offset(-1);
            }

            if lpszRead != lpszStr.offset(dwLen as isize) {
                *lpszRead = '\0' as WCHAR;
                bRet = (TRUE as i32);
            }
        }
    }
    return bRet;
}

unsafe extern "C" fn _SHStrDupAA(mut src: LPCSTR, mut dest: *mut LPSTR) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !src.is_null() {
        len = lstrlenA(src) + 1 as ::core::ffi::c_int;
        *dest = CoTaskMemAlloc(len as SIZE_T) as LPSTR;
    } else {
        *dest = ::core::ptr::null_mut::<CHAR>();
    }
    if !(*dest).is_null() {
        lstrcpynA(*dest, src, len);
        hr = S_OK as HRESULT;
    } else {
        hr = E_OUTOFMEMORY as HRESULT;
    }
    WINE_TRACE!("%s->(%p)\n", debugstr_a(src as *const ::core::ffi::c_char), *dest);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHStrDupA(lpszStr: LPCSTR, lppszDest: *mut LPWSTR) -> HRESULT {
    let mut hRet: HRESULT;
    let mut len: ::core::ffi::c_int = 0;

    if !lpszStr.is_null() {
        len = (MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszStr,
            -1,
            core::ptr::null_mut::<WCHAR>(),
            0,
        ) as usize * core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
        *lppszDest = CoTaskMemAlloc((len) as SIZE_T) as LPWSTR;
    } else {
        *lppszDest = core::ptr::null_mut::<WCHAR>();
    }

    if !(*lppszDest).is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszStr,
            -1,
            *lppszDest,
            (len as usize / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        );
        hRet = S_OK;
    } else {
        hRet = E_OUTOFMEMORY;
    }

    return hRet;
}

unsafe extern "C" fn _SHStrDupAW(src: LPCWSTR, dest: *mut LPSTR) -> HRESULT {
    let hr: HRESULT;
    let mut len: core::ffi::c_int = 0 as core::ffi::c_int;
    if !src.is_null() {
        len = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            src,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut::<CHAR>(),
            0 as core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        *dest = CoTaskMemAlloc(len as SIZE_T) as LPSTR;
    } else {
        *dest = core::ptr::null_mut::<CHAR>();
    }

    if !(*dest).is_null() {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            src,
            -(1 as core::ffi::c_int),
            *dest,
            len,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        hr = S_OK as HRESULT;
    } else {
        hr = E_OUTOFMEMORY as HRESULT;
    }

    TRACE!("%s->(%p)\n", debugstr_w(src as *const WCHAR), *dest);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHStrDupW(src: LPCWSTR, dest: *mut LPWSTR) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut len: ::core::ffi::c_int = 0;

    if !src.is_null() {
        len = ((lstrlenW(src) + 1) as usize
            * core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
        *dest = CoTaskMemAlloc(len as SIZE_T) as LPWSTR;
    } else {
        *dest = core::ptr::null_mut::<WCHAR>();
    }

    if !(*dest).is_null() {
        memcpy(
            *dest as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            len as usize,
        );
        hr = S_OK;
    } else {
        hr = E_OUTOFMEMORY;
    }

    return hr;
}

unsafe extern "C" fn SHLWAPI_WriteReverseNum(mut lpszOut: LPWSTR, mut dwNum: DWORD) -> LPWSTR {
    let fresh20 = lpszOut;
    lpszOut = lpszOut.offset(-1);
    *fresh20 = '\0' as i32 as WCHAR;
    loop {
        let mut dwNextDigit: DWORD = dwNum.wrapping_rem(10 as DWORD);
        let fresh21 = lpszOut;
        lpszOut = lpszOut.offset(-1);
        *fresh21 = ('0' as i32 as DWORD).wrapping_add(dwNextDigit) as WCHAR;
        dwNum = dwNum.wrapping_sub(dwNextDigit).wrapping_div(10 as DWORD);
        if !(dwNum > 0) {
            break;
        }
    }
    return lpszOut;
}

unsafe extern "C" fn SHLWAPI_FormatSignificant(mut lpszNum: LPWSTR, mut dwDigits: ::core::ffi::c_int) -> ::core::ffi::c_int {
    while *lpszNum != 0 {
        lpszNum = lpszNum.offset(1);
        dwDigits -= 1;
        if dwDigits == 0 {
            while *lpszNum != 0 {
                let fresh19 = lpszNum;
                lpszNum = lpszNum.offset(1);
                *fresh19 = '0' as i32 as WCHAR;
            }
            return 0;
        }
    }
    return dwDigits;
}

unsafe extern "C" fn SHLWAPI_WriteTimeClass(lpszOut: LPWSTR, dwValue: DWORD, uClassStringId: UINT, mut iDigits: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut szBuff: [WCHAR; 64] = [0; 64];
    let mut szOut: *mut WCHAR = (core::ptr::addr_of_mut!(szBuff) as *mut WCHAR).offset(32);

    szOut = SHLWAPI_WriteReverseNum(szOut, dwValue);
    iDigits = SHLWAPI_FormatSignificant(szOut.offset(1), iDigits);
    *szOut = ' ' as WCHAR;
    LoadStringW(shlwapi_hInstance, uClassStringId, (core::ptr::addr_of_mut!(szBuff) as *mut WCHAR).offset(32), 32);
    strcatW!(lpszOut, szOut);
    return iDigits;
}

#[no_mangle]
pub unsafe extern "system" fn StrFromTimeIntervalA(
    mut lpszStr: LPSTR,
    mut cchMax: UINT,
    mut dwMS: DWORD,
    mut iDigits: ::core::ffi::c_int,
) -> INT {
    let mut iRet: INT = 0 as INT;
    WINE_TRACE!("(%p,%d,%d,%d)\n", lpszStr, cchMax, dwMS, iDigits);

    if !lpszStr.is_null() && cchMax != 0 {
        let mut szBuff: [WCHAR; 128] = [0; 128];
        StrFromTimeIntervalW(
            core::ptr::addr_of_mut!(szBuff) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 128]>()
                / core::mem::size_of::<WCHAR>()) as UINT,
            dwMS,
            iDigits,
        );
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpszStr,
            cchMax as ::core::ffi::c_int,
            0 as *const CHAR,
            0 as *mut BOOL,
        );
    }
    return iRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrFromTimeIntervalW(
    lpszStr: LPWSTR,
    cchMax: UINT,
    mut dwMS: DWORD,
    mut iDigits: INT,
) -> INT {
    let mut iRet: INT = 0;
    if !lpszStr.is_null() && cchMax != 0 {
        let mut szCopy: [WCHAR; 128] = [0; 128];
        let mut dwHours: DWORD = 0;
        let mut dwMinutes: DWORD = 0;

        if iDigits == 0 || cchMax == 1 {
            *lpszStr = 0 as WCHAR;
            return 0;
        }

        /* Calculate the time classes */
        dwMS = dwMS.wrapping_add(500).wrapping_div(1000);
        dwHours = dwMS.wrapping_div(3600);
        dwMS = dwMS.wrapping_sub(dwHours.wrapping_mul(3600));
        dwMinutes = dwMS.wrapping_div(60);
        dwMS = dwMS.wrapping_sub(dwMinutes.wrapping_mul(60));

        szCopy[0] = 0 as WCHAR;

        if dwHours != 0 {
            iDigits = SHLWAPI_WriteTimeClass(
                &raw mut szCopy as LPWSTR,
                dwHours,
                IDS_TIME_INTERVAL_HOURS,
                iDigits,
            );
        }

        if dwMinutes != 0 && iDigits != 0 {
            iDigits = SHLWAPI_WriteTimeClass(
                &raw mut szCopy as LPWSTR,
                dwMinutes,
                IDS_TIME_INTERVAL_MINUTES,
                iDigits,
            );
        }

        if iDigits != 0 {
            /* Always write seconds if we have significant digits */
            SHLWAPI_WriteTimeClass(
                &raw mut szCopy as LPWSTR,
                dwMS,
                IDS_TIME_INTERVAL_SECONDS,
                iDigits,
            );
        }

        lstrcpynW(
            lpszStr,
            &raw mut szCopy as *mut WCHAR as LPCWSTR,
            cchMax as INT,
        );
        iRet = strlenW(lpszStr) as INT;
    }
    return iRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrIsIntlEqualA(
    mut bCase: BOOL,
    mut lpszStr: LPCSTR,
    mut lpszComp: LPCSTR,
    mut iLen: ::core::ffi::c_int,
) -> BOOL {
    let mut dwFlags: DWORD = 0;
    WINE_TRACE!("(%d,%s,%s,%d)\n", bCase, debugstr_a(lpszStr as *const ::core::ffi::c_char), debugstr_a(lpszComp as *const ::core::ffi::c_char), iLen);
    dwFlags = 0x10000000 as DWORD;
    if bCase == 0 {
        dwFlags |= NORM_IGNORECASE as DWORD;
    }
    return (CompareStringA(GetThreadLocale(), dwFlags, lpszStr, iLen, lpszComp, iLen) == (CSTR_EQUAL as i32))
        as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn StrIsIntlEqualW(
    bCase: BOOL,
    lpszStr: LPCWSTR,
    lpszComp: LPCWSTR,
    iLen: i32,
) -> BOOL {
    let mut dwFlags: DWORD;

    /* FIXME: This flag is undocumented and unknown by our CompareString.
     *        We need a define for it.
     */
    dwFlags = 0x10000000;
    if bCase == 0 {
        dwFlags |= NORM_IGNORECASE;
    }

    return (CompareStringW(GetThreadLocale(), dwFlags, lpszStr, iLen, lpszComp, iLen) == (CSTR_EQUAL as i32)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn StrCpyNXA(
    mut lpszDest: LPSTR,
    mut lpszSrc: LPCSTR,
    mut iLen: ::core::ffi::c_int,
) -> LPSTR {
    WINE_TRACE!("(%p,%s,%i)\n", lpszDest, debugstr_a(lpszSrc as *const ::core::ffi::c_char), iLen);
    if !lpszDest.is_null() && !lpszSrc.is_null() && iLen > 0 {
        loop {
            let fresh23 = iLen;
            iLen = iLen - 1;
            if !(fresh23 > 1 && *lpszSrc as ::core::ffi::c_int != 0) {
                break;
            }
            let fresh24 = lpszSrc;
            lpszSrc = lpszSrc.offset(1);
            let fresh25 = lpszDest;
            lpszDest = lpszDest.offset(1);
            *fresh25 = *fresh24;
        }
        if iLen >= 0 {
            *lpszDest = '\0' as i32 as CHAR;
        }
    }
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrCpyNXW(
    mut lpszDest: LPWSTR,
    mut lpszSrc: LPCWSTR,
    mut iLen: ::core::ffi::c_int,
) -> LPWSTR {
    DPRINT1!(
        "(%p,%s,%i)\n",
        lpszDest,
        debugstr_w(lpszSrc as *const WCHAR),
        iLen
    );
    if !lpszDest.is_null() && !lpszSrc.is_null() && iLen > 0 {
        loop {
            let fresh26 = iLen;
            iLen = iLen - 1;
            if !(fresh26 > 1 && *lpszSrc as ::core::ffi::c_int != 0) {
                break;
            }
            let fresh27 = lpszSrc;
            lpszSrc = lpszSrc.offset(1);
            let fresh28 = lpszDest;
            lpszDest = lpszDest.offset(1);
            *fresh28 = *fresh27;
        }
        if iLen >= 0 {
            *lpszDest = '\0' as i32 as WCHAR;
        }
    }
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpLogicalW(mut lpszStr: LPCWSTR, mut lpszComp: LPCWSTR) -> INT {
    let mut iDiff: INT = 0;

    if !lpszStr.is_null() && !lpszComp.is_null() {
        while *lpszStr != 0 {
            if *lpszComp == 0 {
                return 1;
            } else if isdigitW!(*lpszStr) != 0 {
                let mut iStr: INT = 0;
                let mut iComp: INT = 0;

                if isdigitW!(*lpszComp) == 0 {
                    return -1;
                }

                /* Compare the numbers */
                StrToIntExW(lpszStr, 0, core::ptr::addr_of_mut!(iStr));
                StrToIntExW(lpszComp, 0, core::ptr::addr_of_mut!(iComp));

                if iStr < iComp {
                    return -1;
                } else if iStr > iComp {
                    return 1;
                }

                /* Skip */
                while isdigitW!(*lpszStr) != 0 {
                    lpszStr = lpszStr.offset(1);
                }
                while isdigitW!(*lpszComp) != 0 {
                    lpszComp = lpszComp.offset(1);
                }
            } else if isdigitW!(*lpszComp) != 0 {
                return 1;
            } else {
                iDiff = ChrCmpIW(*lpszStr, *lpszComp);
                if iDiff > 0 {
                    return 1;
                } else if iDiff < 0 {
                    return -1;
                }

                lpszStr = lpszStr.offset(1);
                lpszComp = lpszComp.offset(1);
            }
        }
        if *lpszComp != 0 {
            return -1;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn StrFormatByteSizeW(llBytes: i64, lpszDest: LPWSTR, cchMax: u32) -> LPWSTR {
    // C's `#ifdef __REACTOS__` arm, live on every compile. The bought body wrapped both arms
    // in an invented cfg name nothing sets, which shipped Wine's `#else`: " ?B" with the
    // resource ID truncated in as the unit prefix.
    static bfFormats: [SHLWAPI_BYTEFORMATS; 16] = [
        SHLWAPI_BYTEFORMATS { dLimit: (10*KB) as LONGLONG, dDivisor: 10.24, dNormaliser: 100.0, nDecimals: 2, nFormatID: IDS_KB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (100*KB) as LONGLONG, dDivisor: 102.4, dNormaliser: 10.0, nDecimals: 1, nFormatID: IDS_KB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (1000*KB) as LONGLONG, dDivisor: 1024.0, dNormaliser: 1.0, nDecimals: 0, nFormatID: IDS_KB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (10*MB) as LONGLONG, dDivisor: 10485.76, dNormaliser: 100.0, nDecimals: 2, nFormatID: IDS_MB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (100*MB) as LONGLONG, dDivisor: 104857.6, dNormaliser: 10.0, nDecimals: 1, nFormatID: IDS_MB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (1000*MB) as LONGLONG, dDivisor: 1048576.0, dNormaliser: 1.0, nDecimals: 0, nFormatID: IDS_MB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (10*GB) as LONGLONG, dDivisor: 10737418.24, dNormaliser: 100.0, nDecimals: 2, nFormatID: IDS_GB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (100*GB) as LONGLONG, dDivisor: 107374182.4, dNormaliser: 10.0, nDecimals: 1, nFormatID: IDS_GB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (1000*GB) as LONGLONG, dDivisor: 1073741824.0, dNormaliser: 1.0, nDecimals: 0, nFormatID: IDS_GB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (10*TB) as LONGLONG, dDivisor: 10485.76, dNormaliser: 100.0, nDecimals: 2, nFormatID: IDS_TB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (100*TB) as LONGLONG, dDivisor: 104857.6, dNormaliser: 10.0, nDecimals: 1, nFormatID: IDS_TB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (1000*TB) as LONGLONG, dDivisor: 1048576.0, dNormaliser: 1.0, nDecimals: 0, nFormatID: IDS_TB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (10*PB) as LONGLONG, dDivisor: 10737418.24, dNormaliser: 100.00, nDecimals: 2, nFormatID: IDS_PB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (100*PB) as LONGLONG, dDivisor: 107374182.4, dNormaliser: 10.00, nDecimals: 1, nFormatID: IDS_PB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: (1000*PB) as LONGLONG, dDivisor: 1073741824.0, dNormaliser: 1.00, nDecimals: 0, nFormatID: IDS_PB_FORMAT },
        SHLWAPI_BYTEFORMATS { dLimit: 0, dDivisor: 10995116277.76, dNormaliser: 100.00, nDecimals: 2, nFormatID: IDS_EB_FORMAT },
    ];


    let mut dBytes: f64;
    let mut i: u32 = 0;

    if lpszDest.is_null() || cchMax == 0 {
        return lpszDest;
    }

    if llBytes < 1024 {
        let mut wszBytesFormat: [u16; 64] = [0; 64];
        LoadStringW(shlwapi_hInstance, IDS_BYTES_FORMAT, wszBytesFormat.as_mut_ptr(), 64);
        snprintfW(lpszDest, cchMax as usize, wszBytesFormat.as_ptr(), llBytes as i32);
        return lpszDest;
    }

    /* Note that if this loop completes without finding a match, i will be
     * pointing at the last entry, which is a catch all for > 1000 PB
     */
    while (i as usize) < (core::mem::size_of::<[SHLWAPI_BYTEFORMATS; 16]>() / core::mem::size_of::<SHLWAPI_BYTEFORMATS>() - 1 as usize) {
        if llBytes < bfFormats[i as usize].dLimit {
            break;
        }
        i += 1;
    }

    /* Above 1 TB we encounter problems with FP accuracy. So for amounts above
     * this number we integer shift down by 1 MB first. The table above has
     * the divisors scaled down from the '< 10 TB' entry onwards, to account
     * for this. We also add a small fudge factor to get the correct result for
     * counts that lie exactly on a 1024 byte boundary.
     */
    if i > 8 {
        dBytes = (llBytes >> 20) as f64 + 0.001; /* Scale down by 1 MB */
    } else {
        dBytes = llBytes as f64 + 0.00001;
    }

    dBytes = floor(dBytes / bfFormats[i as usize].dDivisor) / bfFormats[i as usize].dNormaliser;

    {
        let mut szBuff: [u16; 40] = [0; 40];
        let mut wszFormat: [u16; 40] = [0; 40];
        if FormatDouble(dBytes, bfFormats[i as usize].nDecimals, szBuff.as_mut_ptr(), ARRAYSIZE!(szBuff) as i32) == 0 {
            return core::ptr::null_mut();
        }
        LoadStringW(shlwapi_hInstance, bfFormats[i as usize].nFormatID, wszFormat.as_mut_ptr(), ARRAYSIZE!(wszFormat) as i32);
        snprintfW(lpszDest, cchMax as usize, wszFormat.as_ptr(), szBuff.as_ptr());
    }


    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrFormatByteSize64A(mut llBytes: LONGLONG, mut lpszDest: LPSTR, mut cchMax: UINT) -> LPSTR {
    let mut wszBuff: [WCHAR; 32] = [0; 32];
    StrFormatByteSizeW(
        llBytes,
        core::ptr::addr_of_mut!(wszBuff) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as UINT,
    );
    if !lpszDest.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(wszBuff) as *mut WCHAR as LPCWSTR,
            -1,
            lpszDest,
            cchMax as ::core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn StrFormatByteSizeA(
    dwBytes: DWORD,
    lpszDest: LPSTR,
    cchMax: UINT,
) -> LPSTR {
    // TRACE("(%d,%p,%d)\n", dwBytes, lpszDest, cchMax);
    StrFormatByteSize64A((dwBytes) as LONGLONG, lpszDest, cchMax)
}

#[no_mangle]
pub unsafe extern "system" fn SHTruncateString(mut lpStr: LPSTR, mut size: DWORD) -> DWORD {
    if !lpStr.is_null() && size != 0 {
        let mut lastByte: LPSTR = lpStr.offset(size as isize - 1 as isize);

        while lpStr < lastByte {
            lpStr = lpStr.offset(
                (if IsDBCSLeadByte(*lpStr as BYTE) != 0 {
                    2
                } else {
                    1
                }) as isize,
            );
        }

        if lpStr == lastByte && IsDBCSLeadByte(*lpStr as BYTE) != 0 {
            *lpStr = '\0' as i32 as CHAR;
            size = size.wrapping_sub(1);
        }
        return size;
    }
    return 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHStripMneumonicA(lpszStr: LPCSTR) -> ::core::ffi::c_char {
    let mut lpszIter: LPSTR = ::core::ptr::null_mut::<CHAR>();
    let mut lpszTmp: LPSTR = ::core::ptr::null_mut::<CHAR>();
    let mut ch: ::core::ffi::c_char = 0;

    ch = *lpszStr as ::core::ffi::c_char;

    lpszIter = StrChrA(lpszStr, '&' as i32 as WORD);
    if !lpszIter.is_null() {
        lpszTmp = CharNextA(lpszIter as LPCSTR);
        if *lpszTmp != 0 {
            if *lpszTmp as ::core::ffi::c_int != '&' as i32 {
                ch = *lpszTmp as ::core::ffi::c_char;
            }
            memmove(
                lpszIter as *mut ::core::ffi::c_void,
                lpszTmp as *const ::core::ffi::c_void,
                strlen(lpszTmp as *const ::core::ffi::c_char) + 1,
            );
        }
    }

    return ch;
}

#[no_mangle]
pub unsafe extern "system" fn SHStripMneumonicW(lpszStr: LPCWSTR) -> WCHAR {
    let mut lpszIter: LPWSTR;
    let mut lpszTmp: LPWSTR;
    let mut ch: WCHAR;

    ch = *lpszStr;

    lpszIter = StrChrW(lpszStr, '&' as u16);
    if !lpszIter.is_null() {
        lpszTmp = lpszIter.offset(1 as isize);
        if *lpszTmp != 0 {
            if *lpszTmp != '&' as u16 {
                ch = *lpszTmp;
            }

            memmove(
                lpszIter as *mut core::ffi::c_void,
                lpszTmp as *const core::ffi::c_void,
                (strlenW(lpszTmp) as usize + 1 as usize)
                    * core::mem::size_of::<WCHAR>(),
            );
        }
    }

    return ch;
}

#[no_mangle]
pub unsafe extern "system" fn SHAnsiToUnicodeCP(
    mut dwCp: DWORD,
    mut lpSrcStr: LPCSTR,
    mut lpDstStr: LPWSTR,
    mut iLen: ::core::ffi::c_int,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    dwRet = MultiByteToWideChar(
        dwCp as UINT,
        0 as DWORD,
        lpSrcStr,
        -(1 as ::core::ffi::c_int),
        lpDstStr,
        iLen,
    ) as DWORD;
    WINE_TRACE!("%s->%s,ret=%d\n", debugstr_a(lpSrcStr as *const ::core::ffi::c_char), debugstr_w(lpDstStr as *const WCHAR), dwRet);
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHAnsiToUnicode(lpSrcStr: LPCSTR, lpDstStr: LPWSTR, iLen: i32) -> DWORD {
    return SHAnsiToUnicodeCP(CP_ACP as DWORD, lpSrcStr, lpDstStr, iLen);
}

#[no_mangle]
pub unsafe extern "system" fn SHUnicodeToAnsiCP(mut CodePage: UINT, mut lpSrcStr: LPCWSTR, mut lpDstStr: LPSTR, mut dstlen: INT) -> DWORD {
    static emptyW: [WCHAR; 1] = [0];
    let mut len: INT = 0;
    let mut reqLen: INT = 0;
    let mut mem: LPSTR = core::ptr::null_mut();

    if lpDstStr.is_null() || dstlen == 0 {
        return 0;
    }

    if lpSrcStr.is_null() {
        lpSrcStr = core::ptr::addr_of!(emptyW) as *const WCHAR as LPCWSTR;
    }

    *lpDstStr = 0 as CHAR;

    len = (strlenW(lpSrcStr) + 1) as i32;

    'sw: {
        if CodePage == CP_WINUNICODE {
            CodePage = CP_UTF8;
        } else if CodePage == 0x0000C350
            || CodePage == CP_UTF7
            || CodePage == CP_UTF8
        {
        } else {
            break 'sw;
        }
        {
            let mut dwMode: DWORD = 0;
            let mut lenW: INT = len - 1;
            let mut needed: INT = dstlen - 1;
            let mut hr: HRESULT = 0;

            /* try the user supplied buffer first */
            hr = ConvertINetUnicodeToMultiByte(
                core::ptr::addr_of_mut!(dwMode),
                CodePage as DWORD,
                lpSrcStr,
                core::ptr::addr_of_mut!(lenW),
                lpDstStr,
                core::ptr::addr_of_mut!(needed),
            );
            if hr == S_OK {
                *lpDstStr.offset(needed as isize) = 0 as CHAR;
                return (needed + 1) as DWORD;
            }

            /* user buffer too small. exclude termination and copy as much as possible */
            lenW = len;
            hr = ConvertINetUnicodeToMultiByte(
                core::ptr::addr_of_mut!(dwMode),
                CodePage as DWORD,
                lpSrcStr,
                core::ptr::addr_of_mut!(lenW),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(needed),
            );
            needed += 1;
            mem = HeapAlloc(
                GetProcessHeap(),
                0,
                needed as SIZE_T,
            ) as LPSTR;
            if mem.is_null() {
                return 0;
            }

            hr = ConvertINetUnicodeToMultiByte(
                core::ptr::addr_of_mut!(dwMode),
                CodePage as DWORD,
                lpSrcStr,
                core::ptr::addr_of_mut!(len),
                mem,
                core::ptr::addr_of_mut!(needed),
            );
            if hr == S_OK {
                reqLen = SHTruncateString(mem, dstlen as DWORD) as INT;
                if reqLen > 0 {
                    memcpy(
                        lpDstStr as *mut core::ffi::c_void,
                        mem as *const core::ffi::c_void,
                        ((reqLen - 1) as SIZE_T) as _,
                    );
                }
            }
            HeapFree(
                GetProcessHeap(),
                0,
                mem as PVOID,
            );
            return 0;
        }
    }

    /* try the user supplied buffer first */
    reqLen = WideCharToMultiByte(
        CodePage as DWORD,
        0,
        lpSrcStr,
        len,
        lpDstStr,
        dstlen,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    if reqLen == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        reqLen = WideCharToMultiByte(
            CodePage as DWORD,
            0,
            lpSrcStr,
            len,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if reqLen != 0 {
            mem = HeapAlloc(
                GetProcessHeap(),
                0,
                reqLen as SIZE_T,
            ) as LPSTR;
            if !mem.is_null() {
                WideCharToMultiByte(
                    CodePage as DWORD,
                    0,
                    lpSrcStr,
                    len,
                    mem,
                    reqLen,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );

                reqLen = SHTruncateString(mem, (dstlen - 1) as DWORD) as INT;
                reqLen += 1;

                lstrcpynA(lpDstStr, mem as LPCSTR, reqLen);
                HeapFree(
                    GetProcessHeap(),
                    0,
                    mem as PVOID,
                );
                *lpDstStr.offset((reqLen - 1) as isize) = 0 as CHAR;
            }
        }
    }
    return reqLen as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHUnicodeToAnsi(lpSrcStr: LPCWSTR, lpDstStr: LPSTR, iLen: INT) -> INT {
    return SHUnicodeToAnsiCP(CP_ACP as UINT, lpSrcStr, lpDstStr, iLen) as INT;
}

#[no_mangle]
pub unsafe extern "system" fn SHAnsiToAnsi(lpszSrc: LPCSTR, lpszDst: LPSTR, iLen: i32) -> DWORD {
    let mut lpszRet: LPSTR;

    lpszRet = StrCpyNXA(lpszDst, lpszSrc, iLen);
    return (lpszRet.offset_from(lpszDst) as isize + 1 as isize) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHUnicodeToUnicode(
    lpszSrc: LPCWSTR,
    lpszDst: LPWSTR,
    iLen: ::core::ffi::c_int,
) -> DWORD {
    let mut lpszRet: LPWSTR = core::ptr::null_mut::<WCHAR>();
    WINE_TRACE!("(%s,%p,0x%08x)\n", debugstr_w(lpszSrc as *const WCHAR), lpszDst, iLen);
    lpszRet = StrCpyNXW(lpszDst, lpszSrc, iLen);
    return (lpszRet.offset_from(lpszDst) as ::core::ffi::c_longlong
        + 1 as ::core::ffi::c_longlong) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DoesStringRoundTripA(lpSrcStr: LPCSTR, lpDst: LPSTR, iLen: INT) -> BOOL {
    lstrcpynA(lpDst, lpSrcStr, iLen);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DoesStringRoundTripW(lpSrcStr: LPCWSTR, lpDst: LPSTR, iLen: INT) -> BOOL {
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    SHUnicodeToAnsi(lpSrcStr, lpDst, iLen);
    SHAnsiToUnicode(lpDst as LPCSTR, core::ptr::addr_of_mut!(szBuff) as LPWSTR, MAX_PATH as core::ffi::c_int);
    return (strcmpW!(lpSrcStr, core::ptr::addr_of_mut!(szBuff) as LPCWSTR) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHLoadIndirectString(
    mut src: LPCWSTR,
    mut dst: LPWSTR,
    mut dst_len: UINT,
    reserved: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut dllname: *mut WCHAR = core::ptr::null_mut();
    let mut hmod: HMODULE = core::ptr::null_mut();
    let mut hr: HRESULT = E_FAIL as HRESULT;
    let mut szExpanded: [WCHAR; 512] = [0; 512];

    TRACE!("(%s %p %08x %p)\n", debugstr_w(src), dst, dst_len, reserved);

    'end: {
        if *src.offset(0 as isize) == '@' as WCHAR {
            let mut index_str: *mut WCHAR;
            let mut index: i32;

            if !wcschr(src, '%' as WCHAR).is_null() {
                ExpandEnvironmentStringsW(
                    src,
                    core::ptr::addr_of_mut!(szExpanded) as *mut WCHAR,
                    ARRAY_SIZE!(szExpanded) as DWORD,
                );
                src = core::ptr::addr_of_mut!(szExpanded) as *mut WCHAR as LPCWSTR;
            }

            *dst.offset(0 as isize) = 0 as WCHAR;
            dllname = StrDupW(src.offset(1 as isize));
            index_str = strchrW!(dllname, ',' as WCHAR);

            if index_str.is_null() {
                break 'end;
            }

            *index_str = 0 as WCHAR;
            index_str = index_str.add(1);
            index = atoiW!(index_str);

            hmod = LoadLibraryExW(
                dllname as LPCWSTR,
                core::ptr::null_mut(),
                LOAD_LIBRARY_AS_DATAFILE as DWORD,
            ) as HMODULE;

            if hmod.is_null() {
                break 'end;
            }

            if index < 0 as i32 {
                if LoadStringW(
                    hmod as HINSTANCE,
                    (-index) as UINT,
                    dst,
                    dst_len as i32,
                ) != 0
                {
                    hr = S_OK as HRESULT;
                }
            } else {
                FIXME!("can't handle non-negative indices (%d)\n", index);
            }
        } else {
            if dst != src as LPWSTR {
                lstrcpynW(dst, src, dst_len as i32);
            }
            hr = S_OK as HRESULT;
        }
    }

    if !hmod.is_null() {
        FreeLibrary(hmod);
    }
    LocalFree(dllname as HLOCAL);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharSpaceA(mut c: CHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return ((GetStringTypeA(
        GetSystemDefaultLCID(),
        CT_CTYPE1 as DWORD,
        core::ptr::addr_of_mut!(c) as LPCSTR,
        1 as i32,
        core::ptr::addr_of_mut!(CharType),
    ) != 0)
        && ((CharType as i32) & (C1_SPACE as i32)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharSpaceW(mut wc: WCHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return (GetStringTypeW(
        CT_CTYPE1 as DWORD,
        core::ptr::addr_of_mut!(wc) as LPCWSTR,
        1 as core::ffi::c_int,
        core::ptr::addr_of_mut!(CharType),
    ) != 0
        && (CharType as core::ffi::c_int & C1_SPACE as core::ffi::c_int) != 0)
        as BOOL;
}
