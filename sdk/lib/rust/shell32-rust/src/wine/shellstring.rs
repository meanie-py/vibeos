//! Shell string helpers — translation of dll/win32/shell32/wine/shellstring.c.
//!
//! STRRET<->string conversion (`StrRetToStrN*`), OLE/ANSI string converters
//! (`StrToOleStr*`/`OleStrToStrN*`) and path-quote escaping (`CheckEscapes*`).
//! All exports are WINAPI (stdcall on i386 → `_StrRetToStrNA@16` … matched by
//! `#[no_mangle] extern "system"`). TRACE/FIXME/debugstr machinery dropped.

use crate::types::*;
use ntdll_sys::nt::SIZE_T;
// One declaration per external symbol: island.rs.
use crate::island::LocalAlloc;
use core::ptr::{self, write_volatile};

// ── STRRET / ITEMIDLIST layout (shtypes.idl) — file-local, hoist candidates ────────────
const MAX_PATH: usize = 260; // shtypes.idl

/// STRRET union (NONAMELESSUNION → named `u`).


pub use crate::shellcom::{STRRET, STRRET_u};
pub type LPSTRRET = *mut STRRET;

pub use crate::island::SHITEMID;

#[repr(C)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

// STRRET_TYPE (shtypes.idl)
const STRRET_WSTR: UINT = 0; // shtypes.idl
const STRRET_OFFSET: UINT = 1; // shtypes.idl
const STRRET_CSTR: UINT = 2; // shtypes.idl

const CP_ACP: UINT = 0; // winnls.h — system default ANSI code page
const LPTR: UINT = 0x0040; // winbase.h — LMEM_FIXED(0x0) | LMEM_ZEROINIT(0x40)

// ── imports (resolved via shell32's existing import libs at link) ──────────────────────
extern "system" {
    // kernel32
    fn GetVersion() -> DWORD;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    fn LocalFree(hMem: HANDLE) -> HANDLE;
    fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: INT) -> LPSTR;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn lstrlenW(lpString: LPCWSTR) -> INT;
    // ole32
    fn CoTaskMemFree(pv: LPVOID);
    // shlwapi
    fn SHAnsiToUnicode(pszSrc: LPCSTR, pwszDst: LPWSTR, cwchBuf: INT) -> DWORD;
    fn SHUnicodeToAnsi(pwszSrc: LPCWSTR, pszDst: LPSTR, cchBuf: INT) -> BOOL;
    fn StrPBrkW(psz: LPCWSTR, pszSet: LPCWSTR) -> LPWSTR;
}

extern "C" {
    // msvcrt — wine/unicode.h strcpyW/strlenW macros expand to these.
    fn wcscpy(dest: LPWSTR, src: LPCWSTR) -> LPWSTR;
    fn wcslen(s: LPCWSTR) -> usize;
}

/// SHELL_OsIsUnicode (shell32_main.h): high-bit of version clear ⇒ emulating NT.
#[inline]
unsafe fn shell_os_is_unicode() -> bool {
    (GetVersion() & 0x80000000) == 0
}

// ───────────────────────── STRRET functions ──────────────────────────

#[no_mangle]
pub unsafe extern "system" fn StrRetToStrNA(
    dest: LPSTR,
    len: DWORD,
    src: LPSTRRET,
    pidl: *const ITEMIDLIST,
) -> BOOL {
    if dest.is_null() {
        return FALSE;
    }

    match (*src).uType {
        STRRET_WSTR => {
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*src).u.pOleStr,
                -1,
                dest,
                len as INT,
                ptr::null(),
                ptr::null_mut(),
            );
            CoTaskMemFree((*src).u.pOleStr as LPVOID);
        }
        STRRET_CSTR => {
            lstrcpynA(dest, (*src).u.cStr.as_ptr(), len as INT);
        }
        STRRET_OFFSET => {
            let p = (pidl as LPCSTR).add((*src).u.uOffset as usize);
            lstrcpynA(dest, p, len as INT);
        }
        _ => {
            if len != 0 {
                write_volatile(dest, 0);
            }
            return FALSE;
        }
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn StrRetToStrNW(
    dest: LPWSTR,
    len: DWORD,
    src: LPSTRRET,
    pidl: *const ITEMIDLIST,
) -> BOOL {
    if dest.is_null() {
        return FALSE;
    }

    match (*src).uType {
        STRRET_WSTR => {
            lstrcpynW(dest, (*src).u.pOleStr, len as INT);
            CoTaskMemFree((*src).u.pOleStr as LPVOID);
        }
        STRRET_CSTR => {
            if MultiByteToWideChar(CP_ACP, 0, (*src).u.cStr.as_ptr(), -1, dest, len as INT) == 0
                && len != 0
            {
                write_volatile(dest.add((len - 1) as usize), 0);
            }
        }
        STRRET_OFFSET => {
            let p = (pidl as LPCSTR).add((*src).u.uOffset as usize);
            if MultiByteToWideChar(CP_ACP, 0, p, -1, dest, len as INT) == 0 && len != 0 {
                write_volatile(dest.add((len - 1) as usize), 0);
            }
        }
        _ => {
            if len != 0 {
                write_volatile(dest, 0);
            }
            return FALSE;
        }
    }
    TRUE
}

/// StrRetToStrN [SHELL32.96] — converts a STRRET to a normal string.
#[no_mangle]
pub unsafe extern "system" fn StrRetToStrNAW(
    dest: LPVOID,
    len: DWORD,
    src: LPSTRRET,
    pidl: *const ITEMIDLIST,
) -> BOOL {
    if shell_os_is_unicode() {
        StrRetToStrNW(dest as LPWSTR, len, src, pidl)
    } else {
        StrRetToStrNA(dest as LPSTR, len, src, pidl)
    }
}

// ───────────────────────── OLESTR functions ──────────────────────────

/// StrToOleStrA — MultiByteToWideChar into a MAX_PATH buffer.
unsafe fn str_to_ole_str_a(lpWideCharStr: LPWSTR, lpMultiByteString: LPCSTR) -> INT {
    MultiByteToWideChar(CP_ACP, 0, lpMultiByteString, -1, lpWideCharStr, MAX_PATH as INT)
}

/// StrToOleStrW — strcpyW + strlenW.
unsafe fn str_to_ole_str_w(lpWideCharStr: LPWSTR, lpWString: LPCWSTR) -> INT {
    wcscpy(lpWideCharStr, lpWString);
    wcslen(lpWideCharStr) as INT
}

/// StrToOleStr [SHELL32.163]
#[no_mangle]
pub unsafe extern "system" fn StrToOleStrAW(lpWideCharStr: LPWSTR, lpString: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        str_to_ole_str_w(lpWideCharStr, lpString as LPCWSTR) as BOOL
    } else {
        str_to_ole_str_a(lpWideCharStr, lpString as LPCSTR) as BOOL
    }
}

/// StrToOleStrNA
unsafe fn str_to_ole_str_na(lpWide: LPWSTR, nWide: INT, lpStrA: LPCSTR, nStr: INT) -> BOOL {
    MultiByteToWideChar(CP_ACP, 0, lpStrA, nStr, lpWide, nWide)
}

/// StrToOleStrNW
unsafe fn str_to_ole_str_nw(lpWide: LPWSTR, nWide: INT, lpStrW: LPCWSTR, _nStr: INT) -> BOOL {
    if !lstrcpynW(lpWide, lpStrW, nWide).is_null() {
        return lstrlenW(lpWide);
    }
    FALSE
}

/// StrToOleStrN [SHELL32.79]
#[no_mangle]
pub unsafe extern "system" fn StrToOleStrNAW(
    lpWide: LPWSTR,
    nWide: INT,
    lpStr: LPCVOID,
    nStr: INT,
) -> BOOL {
    if shell_os_is_unicode() {
        str_to_ole_str_nw(lpWide, nWide, lpStr as LPCWSTR, nStr)
    } else {
        str_to_ole_str_na(lpWide, nWide, lpStr as LPCSTR, nStr)
    }
}

/// OleStrToStrNA
unsafe fn ole_str_to_str_na(lpStr: LPSTR, nStr: INT, lpOle: LPCWSTR, nOle: INT) -> BOOL {
    WideCharToMultiByte(
        CP_ACP,
        0,
        lpOle,
        nOle,
        lpStr,
        nStr,
        ptr::null(),
        ptr::null_mut(),
    )
}

/// OleStrToStrNW
unsafe fn ole_str_to_str_nw(lpwStr: LPWSTR, nwStr: INT, lpOle: LPCWSTR, _nOle: INT) -> BOOL {
    if !lstrcpynW(lpwStr, lpOle, nwStr).is_null() {
        return lstrlenW(lpwStr);
    }
    FALSE
}

/// OleStrToStrN [SHELL32.78]
#[no_mangle]
pub unsafe extern "system" fn OleStrToStrNAW(
    lpOut: LPVOID,
    nOut: INT,
    lpIn: LPCVOID,
    nIn: INT,
) -> BOOL {
    if shell_os_is_unicode() {
        ole_str_to_str_nw(lpOut as LPWSTR, nOut, lpIn as LPCWSTR, nIn)
    } else {
        ole_str_to_str_na(lpOut as LPSTR, nOut, lpIn as LPCWSTR, nIn)
    }
}

/// CheckEscapesA [SHELL32.@] — quote a path that contains special characters.
#[no_mangle]
pub unsafe extern "system" fn CheckEscapesA(string: LPSTR, len: DWORD) {
    if string.is_null() || *string == 0 {
        return;
    }

    let wString = LocalAlloc(LPTR, ((len as usize).wrapping_mul(2)) as SIZE_T) as LPWSTR;
    if wString.is_null() {
        return;
    }

    SHAnsiToUnicode(string, wString, len as INT);
    CheckEscapesW(wString, len);
    SHUnicodeToAnsi(wString, string, len as INT);
    LocalFree(wString as HANDLE);
}

/// CheckEscapesW [SHELL32.@] — see CheckEscapesA.
#[no_mangle]
pub unsafe extern "system" fn CheckEscapesW(string: LPWSTR, len: DWORD) {
    let size = lstrlenW(string); // INT

    // L" \",;^"
    const SET: [WCHAR; 6] = [0x20, 0x22, 0x2C, 0x3B, 0x5E, 0x00];

    if !StrPBrkW(string, SET.as_ptr()).is_null() && (size as DWORD) + 2 <= len {
        let mut s = string.offset((size - 1) as isize);
        let mut d = string.offset((size + 2) as isize);
        write_volatile(d, 0);
        d = d.offset(-1);
        write_volatile(d, b'"' as WCHAR);
        d = d.offset(-1);
        while d > string {
            let v = *s;
            write_volatile(d, v);
            d = d.offset(-1);
            s = s.offset(-1);
        }
        write_volatile(d, b'"' as WCHAR);
    }
}

// ADDED-TO-TYPES?: STRRET / STRRET_u / LPSTRRET, SHITEMID, ITEMIDLIST,
// STRRET_WSTR/STRRET_OFFSET/STRRET_CSTR, MAX_PATH (260), CP_ACP (0), LPTR (0x40).
// These (esp. STRRET/ITEMIDLIST/SHITEMID + the STRRET_* consts) are shared across the
// PIDL/shell-struct waves and should be hoisted into crate::types.
