use crate::ntwrapper::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
use win32k_sys::win32k::*;
use crate::internal::*;
use crate::winbase::*;
use crate::FIELD_OFFSET;
use core::ffi::c_void;

extern "C" {
    fn strlen(s: LPCSTR) -> usize;
    fn wcslen(s: LPCWSTR) -> usize;
}
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: core::ffi::c_int, ) -> core::ffi::c_int;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: core::ffi::c_int, lpMultiByteStr: LPSTR, cbMultiByte: core::ffi::c_int, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL, ) -> core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// (no file-scope declarations)

// wine/unicode.h static-inline helpers (inlined per-TU in C; no external symbol).
#[inline]
unsafe fn toupperW(ch: WCHAR) -> WCHAR {
    towupper(ch as core::ffi::c_int) as WCHAR
}
#[inline]
unsafe fn tolowerW(ch: WCHAR) -> WCHAR {
    towlower(ch as core::ffi::c_int) as WCHAR
}
#[inline]
unsafe fn struprW(str: *mut WCHAR) -> *mut WCHAR {
    let mut p = str;
    while *p != 0 {
        *p = toupperW(*p);
        p = p.add(1);
    }
    str
}
#[inline]
unsafe fn strlwrW(str: *mut WCHAR) -> *mut WCHAR {
    let mut p = str;
    while *p != 0 {
        *p = tolowerW(*p);
        p = p.add(1);
    }
    str
}

unsafe fn GetC1Type(Ch: WCHAR) -> WORD {
    let mut Ch = Ch;
    let mut CharType: WORD = 0;

    if GetStringTypeW(CT_CTYPE1, &mut Ch, 1, &mut CharType) == 0 {
        return 0;
    }

    CharType
}

#[no_mangle]
pub unsafe extern "system" fn CharLowerA(str: LPSTR) -> LPSTR {
    if HIWORD(str as UINT_PTR as DWORD) == 0 {
        let mut ch: core::ffi::c_char = LOWORD(str as UINT_PTR as DWORD) as core::ffi::c_char;
        CharLowerBuffA(&mut ch, 1);
        return (ch as u8 as UINT_PTR) as LPSTR;
    }

    CharLowerBuffA(str, strlen(str) as DWORD);

    str
}

#[no_mangle]
pub unsafe extern "system" fn CharLowerBuffA(str: LPSTR, mut len: DWORD) -> DWORD {
    let lenW: DWORD;
    let strW: *mut WCHAR;
    if str.is_null() {
        return 0;
    }

    lenW = MultiByteToWideChar(CP_ACP, 0, str, len as i32, core::ptr::null_mut(), 0) as DWORD;
    strW = HeapAlloc(
        GetProcessHeap(),
        0,
        ((lenW as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    if !strW.is_null() {
        MultiByteToWideChar(CP_ACP, 0, str, len as i32, strW, lenW as i32);
        CharLowerBuffW(strW, lenW);
        len = WideCharToMultiByte(
            CP_ACP,
            0,
            strW,
            lenW as i32,
            str,
            len as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        HeapFree(GetProcessHeap(), 0, strW as *mut _);
        return len;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CharLowerBuffW(mut str: LPWSTR, mut len: DWORD) -> DWORD {
    let ret = len;
    if str.is_null() {
        return 0;
    }
    while len != 0 {
        *str = towlower(*str as _) as _;
        len -= 1;
        str = str.add(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CharLowerW(x: LPWSTR) -> LPWSTR {
    if HIWORD(x as UINT_PTR as DWORD) != 0 {
        strlwrW(x)
    } else {
        tolowerW(LOWORD(x as UINT_PTR as DWORD)) as UINT_PTR as LPWSTR
    }
}

#[no_mangle]
pub unsafe extern "system" fn CharPrevW(start: LPCWSTR, x: LPCWSTR) -> LPWSTR {
    if x > start {
        x.offset(-1) as LPWSTR
    } else {
        x as LPWSTR
    }
}

#[no_mangle]
pub unsafe extern "system" fn CharNextA(ptr: LPCSTR) -> LPSTR {
    if *ptr == 0 {
        return ptr as LPSTR;
    }
    if IsDBCSLeadByte(*ptr.offset(0) as u8) != 0 && *ptr.offset(1) != 0 {
        return ptr.offset(2) as LPSTR;
    }
    ptr.offset(1) as LPSTR
}

#[no_mangle]
pub unsafe extern "system" fn CharNextExA(codepage: WORD, ptr: LPCSTR, flags: DWORD) -> LPSTR {
    if *ptr == 0 {
        return ptr as LPSTR;
    }
    if IsDBCSLeadByteEx(codepage as u32, *ptr.offset(0) as u8) != 0 && *ptr.offset(1) != 0 {
        return ptr.offset(2) as LPSTR;
    }
    ptr.offset(1) as LPSTR
}

#[no_mangle]
pub unsafe extern "system" fn CharNextW(mut x: LPCWSTR) -> LPWSTR {
    if *x != 0 {
        x = x.add(1);
    }
    x as LPWSTR
}

#[no_mangle]
pub unsafe extern "system" fn CharPrevA(start: LPCSTR, mut ptr: LPCSTR) -> LPSTR {
    if ptr > start {
        ptr = ptr.offset(-1);
        if !gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_DBCSENABLED) != 0 {
            let mut ch: LPCSTR = ptr.offset(-1);
            let mut dbl: BOOL = FALSE as BOOL;

            while ch >= start {
                if IsDBCSLeadByte(*ch as u8) == 0 {
                    break;
                }
                dbl = if dbl != 0 { FALSE as BOOL } else { TRUE as BOOL };
                ch = ch.offset(-1);
            }
            if dbl != 0 {
                ptr = ptr.offset(-1);
            }
        }
    }
    ptr as LPSTR
}

#[no_mangle]
pub unsafe extern "system" fn CharPrevExA(
    codepage: WORD,
    start: LPCSTR,
    mut ptr: LPCSTR,
    flags: DWORD,
) -> LPSTR {
    if ptr > start {
        let mut dbl: BOOL = 0;

        ptr = ptr.offset(-1);
        let mut ch = ptr.offset(-1);
        while ch >= start {
            if IsDBCSLeadByteEx(codepage as u32, *ch as u8) == 0 {
                break;
            }
            dbl = (dbl == 0) as BOOL;
            ch = ch.offset(-1);
        }
        if dbl != 0 {
            ptr = ptr.offset(-1);
        }
    }
    ptr as LPSTR
}

#[no_mangle]
pub unsafe extern "system" fn CharToOemA(s: LPCSTR, d: LPSTR) -> BOOL {
    if s.is_null() || d.is_null() {
        return 0;
    }
    CharToOemBuffA(s, d, (strlen(s) + 1) as DWORD)
}

#[no_mangle]
pub unsafe extern "system" fn CharToOemBuffA(s: LPCSTR, d: LPSTR, len: DWORD) -> BOOL {

    if s.is_null() || d.is_null() {
        return 0;
    }

    let bufW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    if !bufW.is_null() {
        MultiByteToWideChar(CP_ACP, 0, s, len as core::ffi::c_int, bufW, len as core::ffi::c_int);
        WideCharToMultiByte(
            CP_OEMCP,
            0,
            bufW,
            len as core::ffi::c_int,
            d,
            len as core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, bufW as LPVOID);
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CharToOemBuffW(s: LPCWSTR, d: LPSTR, len: DWORD) -> BOOL {
    if s.is_null() || d.is_null() {
        return 0;
    }
    WideCharToMultiByte(
        CP_OEMCP,
        0,
        s,
        len as i32,
        d,
        len as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    1
}

#[no_mangle]
pub unsafe extern "system" fn CharToOemW(s: LPCWSTR, d: LPSTR) -> BOOL {
    if s.is_null() || d.is_null() {
        return 0;
    }
    CharToOemBuffW(s, d, (wcslen(s) + 1) as DWORD)
}

#[no_mangle]
pub unsafe extern "system" fn CharUpperA(str: LPSTR) -> LPSTR {
    if HIWORD(str as DWORD) == 0 {
        let mut ch: c_char = LOWORD(str as DWORD) as c_char;
        CharUpperBuffA(&mut ch, 1);
        return (ch as BYTE) as UINT_PTR as LPSTR;
    }

    CharUpperBuffA(str, strlen(str) as DWORD);

    str
}

#[no_mangle]
pub unsafe extern "system" fn CharUpperBuffA(str: LPSTR, mut len: DWORD) -> DWORD {

    let lenW: DWORD;
    let strW: *mut WCHAR;
    if str.is_null() {
        return 0;
    }

    lenW = MultiByteToWideChar(0, 0, str as LPCSTR, len as i32, core::ptr::null_mut(), 0) as DWORD;
    strW = HeapAlloc(GetProcessHeap(), 0, ((lenW as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
    if !strW.is_null() {
        MultiByteToWideChar(0, 0, str as LPCSTR, len as i32, strW, lenW as i32);
        CharUpperBuffW(strW, lenW);
        len = WideCharToMultiByte(0, 0, strW as LPCWSTR, lenW as i32, str, len as i32, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
        HeapFree(GetProcessHeap(), 0, strW as *mut core::ffi::c_void);
        return len;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CharUpperBuffW(mut str: LPWSTR, mut len: DWORD) -> DWORD {
    let ret = len;
    if str.is_null() {
        return 0;
    }
    while len != 0 {
        *str = towupper(*str as i32) as u16;
        len -= 1;
        str = str.add(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CharUpperW(x: LPWSTR) -> LPWSTR {

    if HIWORD(x as UINT_PTR as DWORD) != 0 {
        struprW(x)
    } else {
        toupperW(LOWORD(x as UINT_PTR as DWORD)) as UINT_PTR as LPWSTR
    }
}

#[no_mangle]
pub unsafe extern "system" fn IsCharAlphaA(Ch: CHAR) -> BOOL {
    let mut WCh: WCHAR = 0;

    MultiByteToWideChar(CP_ACP, 0, &Ch as *const CHAR, 1, &mut WCh as *mut WCHAR, 1);
    IsCharAlphaW(WCh)
}

#[no_mangle]
pub unsafe extern "system" fn IsCharAlphaNumericA(Ch: CHAR) -> BOOL {
    let mut WCh: WCHAR = 0;

    MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &mut WCh, 1);
    IsCharAlphaNumericW(WCh)
}

#[no_mangle]
pub unsafe extern "system" fn IsCharAlphaNumericW(Ch: WCHAR) -> BOOL {
    ((GetC1Type(Ch) & (C1_ALPHA | C1_DIGIT) as WORD) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IsCharAlphaW(Ch: WCHAR) -> BOOL {
    ((GetC1Type(Ch) & C1_ALPHA as WORD) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IsCharLowerA(Ch: CHAR) -> BOOL {
    let mut WCh: WCHAR = 0;

    MultiByteToWideChar(CP_ACP, 0, core::ptr::addr_of!(Ch), 1, core::ptr::addr_of_mut!(WCh), 1);
    IsCharLowerW(WCh)
}

#[no_mangle]
pub unsafe extern "system" fn IsCharLowerW(Ch: WCHAR) -> BOOL {
    ((GetC1Type(Ch) & C1_LOWER as WORD) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IsCharUpperA(Ch: CHAR) -> BOOL {
    let mut WCh: WCHAR = 0;

    MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &mut WCh, 1);
    IsCharUpperW(WCh)
}
#[no_mangle]
pub unsafe extern "system" fn IsCharUpperW(Ch: WCHAR) -> BOOL {
    ((GetC1Type(Ch) & (C1_UPPER as WORD)) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn OemToCharA(s: LPCSTR, d: LPSTR) -> BOOL {
    if s.is_null() || d.is_null() {
        return 0;
    }
    OemToCharBuffA(s, d, (strlen(s) + 1) as DWORD)
}

#[no_mangle]
pub unsafe extern "system" fn OemToCharBuffA(s: LPCSTR, d: LPSTR, len: DWORD) -> BOOL {
    let bufW: *mut WCHAR;

    if s.is_null() || d.is_null() {
        return FALSE as BOOL;
    }

    bufW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    if !bufW.is_null() {
        MultiByteToWideChar(CP_OEMCP, 0, s, len as i32, bufW, len as i32);
        WideCharToMultiByte(
            CP_ACP,
            0,
            bufW,
            len as i32,
            d,
            len as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, bufW as *mut core::ffi::c_void);
    }
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn OemToCharBuffW(s: LPCSTR, d: LPWSTR, len: DWORD) -> BOOL {
    if s.is_null() || d.is_null() {
        return FALSE as BOOL;
    }
    MultiByteToWideChar(CP_OEMCP, 0, s, len as i32, d, len as i32);
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn OemToCharW(s: LPCSTR, d: LPWSTR) -> BOOL {
    if s.is_null() || d.is_null() {
        return 0;
    }
    OemToCharBuffW(s, d, (strlen(s) + 1) as DWORD)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS user32.dll
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Text processing functions
//  * COPYRIGHT:   Copyright 2001-2004 Casper S. Hornstrup <chorns@users.sourceforge.net>
//  */
//
// #include <user32.h>
//
// static WORD
// GetC1Type(WCHAR Ch)
// {
//     WORD CharType;
//
//     if (! GetStringTypeW(CT_CTYPE1, &Ch, 1, &CharType))
//     {
//         return 0;
//     }
//
//     return CharType;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// CharLowerA(LPSTR str)
// {
//     if (!HIWORD(str))
//     {
//         char ch = LOWORD(str);
//         CharLowerBuffA( &ch, 1 );
//         return (LPSTR)(UINT_PTR)(BYTE)ch;
//     }
//
//     _SEH2_TRY
//     {
//         CharLowerBuffA( str, strlen(str) );
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         _SEH2_YIELD(return NULL);
//     }
//     _SEH2_END;
//
//     return str;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CharLowerBuffA(LPSTR str, DWORD len)
// {
//     DWORD lenW;
//     WCHAR *strW;
//     if (!str) return 0; /* YES */
//
//     lenW = MultiByteToWideChar(CP_ACP, 0, str, len, NULL, 0);
//     strW = HeapAlloc(GetProcessHeap(), 0, lenW * sizeof(WCHAR));
//     if (strW) {
//         MultiByteToWideChar(CP_ACP, 0, str, len, strW, lenW);
//         CharLowerBuffW(strW, lenW);
//         len = WideCharToMultiByte(CP_ACP, 0, strW, lenW, str, len, NULL, NULL);
//         HeapFree(GetProcessHeap(), 0, strW);
//         return len;
//     }
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CharLowerBuffW(LPWSTR str, DWORD len)
// {
//     DWORD ret = len;
//     if (!str) return 0; /* YES */
//     for (; len; len--, str++) *str = towlower(*str);
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// CharLowerW(LPWSTR x)
// {
//     if (HIWORD(x)) return strlwrW(x);
//     else return (LPWSTR)((UINT_PTR)tolowerW(LOWORD(x)));
// }
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// CharPrevW(LPCWSTR start, LPCWSTR x)
// {
//     if (x > start) return (LPWSTR)(x - 1);
//     else return (LPWSTR)x;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// CharNextA(LPCSTR ptr)
// {
//     if (!*ptr) return (LPSTR)ptr;
//     if (IsDBCSLeadByte(ptr[0]) && ptr[1]) return (LPSTR)(ptr + 2);
//     return (LPSTR)(ptr + 1);
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// CharNextExA(WORD codepage, LPCSTR ptr, DWORD flags)
// {
//     if (!*ptr) return (LPSTR)ptr;
//     if (IsDBCSLeadByteEx(codepage, ptr[0]) && ptr[1]) return (LPSTR)(ptr + 2);
//     return (LPSTR)(ptr + 1);
// }
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// CharNextW(LPCWSTR x)
// {
//     if (*x) x++;
//     return (LPWSTR)x;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// CharPrevA(LPCSTR start, LPCSTR ptr)
// {
//     if (ptr > start)
//     {
//         --ptr;
//         if (gpsi && gpsi->dwSRVIFlags & SRVINFO_DBCSENABLED)
//         {
//             LPCSTR ch;
//             BOOL dbl = FALSE;
//
//             for (ch = ptr - 1; ch >= start; --ch)
//             {
//                 if (!IsDBCSLeadByte(*ch))
//                     break;
//
//                 dbl = !dbl;
//             }
//             if (dbl) --ptr;
//         }
//     }
//     return (LPSTR)ptr;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// CharPrevExA(WORD codepage, LPCSTR start, LPCSTR ptr, DWORD flags)
// {
//     if (ptr > start)
//     {
//         LPCSTR ch;
//         BOOL dbl = FALSE;
//
//         --ptr;
//         for (ch = ptr - 1; ch >= start; --ch)
//         {
//             if (!IsDBCSLeadByteEx(codepage, *ch))
//                 break;
//
//             dbl = !dbl;
//         }
//         if (dbl) --ptr;
//     }
//     return (LPSTR)ptr;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CharToOemA(LPCSTR s, LPSTR d)
// {
//     if (!s || !d) return FALSE;
//     return CharToOemBuffA(s, d, strlen(s) + 1);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CharToOemBuffA(LPCSTR s, LPSTR d, DWORD len)
// {
//     WCHAR* bufW;
//
//     if ( !s || !d ) return FALSE;
//
//     bufW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     if (bufW) {
// 	    MultiByteToWideChar(CP_ACP, 0, s, len, bufW, len);
//         WideCharToMultiByte(CP_OEMCP, 0, bufW, len, d, len, NULL, NULL);
// 	    HeapFree(GetProcessHeap(), 0, bufW);
//     }
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CharToOemBuffW(LPCWSTR s, LPSTR d, DWORD len)
// {
//     if (!s || !d)
//         return FALSE;
//     WideCharToMultiByte(CP_OEMCP, 0, s, len, d, len, NULL, NULL);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CharToOemW(LPCWSTR s, LPSTR d)
// {
//     if ( !s || !d ) return FALSE;
//     return CharToOemBuffW(s, d, wcslen(s) + 1);
// }
//
// /*
//  * @implemented
//  */
// LPSTR WINAPI CharUpperA(LPSTR str)
// {
//     if (!HIWORD(str))
//     {
//         char ch = LOWORD(str);
//         CharUpperBuffA( &ch, 1 );
//         return (LPSTR)(UINT_PTR)(BYTE)ch;
//     }
//
//     _SEH2_TRY
//     {
//         CharUpperBuffA( str, strlen(str) );
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         _SEH2_YIELD(return NULL);
//     }
//     _SEH2_END;
//
//     return str;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CharUpperBuffA(LPSTR str, DWORD len)
// {
//     DWORD lenW;
//     WCHAR* strW;
//     if (!str) return 0; /* YES */
//
//     lenW = MultiByteToWideChar(CP_ACP, 0, str, len, NULL, 0);
//     strW = HeapAlloc(GetProcessHeap(), 0, lenW * sizeof(WCHAR));
//     if (strW) {
//         MultiByteToWideChar(CP_ACP, 0, str, len, strW, lenW);
//         CharUpperBuffW(strW, lenW);
//         len = WideCharToMultiByte(CP_ACP, 0, strW, lenW, str, len, NULL, NULL);
//         HeapFree(GetProcessHeap(), 0, strW);
//         return len;
//     }
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CharUpperBuffW(LPWSTR str, DWORD len)
// {
//     DWORD ret = len;
//     if (!str) return 0; /* YES */
//     for (; len; len--, str++) *str = towupper(*str);
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// CharUpperW(LPWSTR x)
// {
//     if (HIWORD(x)) return struprW(x);
//     else return (LPWSTR)((UINT_PTR)toupperW(LOWORD(x)));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharAlphaA(CHAR Ch)
// {
//     WCHAR WCh;
//
//     MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &WCh, 1);
//     return IsCharAlphaW(WCh);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharAlphaNumericA(CHAR Ch)
// {
//     WCHAR WCh;
//
//     MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &WCh, 1);
//     return IsCharAlphaNumericW(WCh);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharAlphaNumericW(WCHAR Ch)
// {
//     return (GetC1Type(Ch) & (C1_ALPHA|C1_DIGIT)) != 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharAlphaW(WCHAR Ch)
// {
//     return (GetC1Type(Ch) & C1_ALPHA) != 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharLowerA(CHAR Ch)
// {
//     WCHAR WCh;
//
//     MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &WCh, 1);
//     return IsCharLowerW(WCh);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharLowerW(WCHAR Ch)
// {
//     return (GetC1Type(Ch) & C1_LOWER) != 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharUpperA(CHAR Ch)
// {
//     WCHAR WCh;
//
//     MultiByteToWideChar(CP_ACP, 0, &Ch, 1, &WCh, 1);
//     return IsCharUpperW(WCh);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsCharUpperW(WCHAR Ch)
// {
//     return (GetC1Type(Ch) & C1_UPPER) != 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// OemToCharA(LPCSTR s, LPSTR d)
// {
//     if ( !s || !d ) return FALSE;
//     return OemToCharBuffA(s, d, strlen(s) + 1);
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI OemToCharBuffA(LPCSTR s, LPSTR d, DWORD len)
// {
//     WCHAR* bufW;
//
//     if ( !s || !d ) return FALSE;
//
//     bufW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     if (bufW) {
//         MultiByteToWideChar(CP_OEMCP, 0, s, len, bufW, len);
// 	    WideCharToMultiByte(CP_ACP, 0, bufW, len, d, len, NULL, NULL);
// 	    HeapFree(GetProcessHeap(), 0, bufW);
//     }
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// OemToCharBuffW(LPCSTR s, LPWSTR d, DWORD len)
// {
//     if ( !s || !d ) return FALSE;
//     MultiByteToWideChar(CP_OEMCP, 0, s, len, d, len);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI OemToCharW(LPCSTR s, LPWSTR d)
// {
//     if ( !s || !d ) return FALSE;
//     return OemToCharBuffW(s, d, strlen(s) + 1);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
