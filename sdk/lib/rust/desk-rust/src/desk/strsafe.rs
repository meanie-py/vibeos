//! strsafe.h's bounded string ops, ported from the header.
//!
//! `STRSAFEAPI` is `static __inline HRESULT WINAPI`, so no object file in the tree defines
//! these — every C caller inlined its own copy, and flipping desk's .c takes the last one with
//! it. Same situation and same treatment as syssetup's `src/syssetup/strsafe.rs` and shell32's
//! `shellutils.h` inlines: port the header body rather than re-home a wrapper in a .c, so the
//! next flip cannot repeat it.
//!
//! This is the SECOND port of this header (syssetup has the first, a strict subset: no Cat-by-
//! bytes, no Length, no PrintfEx). Two copies is the detection event for extracting a shared
//! `strsafe-rust` crate — deliberately not done in the same change as desk's 2,200-error grind,
//! because it would re-open a landed module. Tracked on task #47.
//!
//! Faithful to `sdk/include/psdk/strsafe.h`: the worker functions truncate and NUL-terminate on
//! overflow and report `STRSAFE_E_INSUFFICIENT_BUFFER`; they do NOT panic. That distinction is
//! the whole point of the API, and it is also why none of this uses slice indexing.
use crate::prelude::*;

// strsafe.h:83,86-100,103,104
pub const STRSAFE_MAX_CCH: usize = 2147483647;
pub const STRSAFE_IGNORE_NULLS: DWORD = 0x00000100;
pub const STRSAFE_FILL_BEHIND_NULL: DWORD = 0x00000200;
pub const STRSAFE_FILL_ON_FAILURE: DWORD = 0x00000400;
pub const STRSAFE_NULL_ON_FAILURE: DWORD = 0x00000800;
pub const STRSAFE_NO_TRUNCATION: DWORD = 0x00001000;
pub const STRSAFE_VALID_FLAGS: DWORD = 0x000000FF
    | STRSAFE_IGNORE_NULLS
    | STRSAFE_FILL_BEHIND_NULL
    | STRSAFE_FILL_ON_FAILURE
    | STRSAFE_NULL_ON_FAILURE
    | STRSAFE_NO_TRUNCATION;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x80070057u32 as HRESULT;

// strsafe.h:100 `#define STRSAFE_GET_FILL_PATTERN(dwFlags) ((int)((dwFlags) & 0x000000FF))`
#[inline(always)]
fn fill_pattern(dwFlags: DWORD) -> c_int {
    (dwFlags & 0x000000FF) as c_int
}

extern "C" {
    /// msvcrt's `_vsnwprintf`. Variadic only in its `va_list` argument, which is a plain pointer
    /// on all three targets, so this IS expressible in stable Rust — unlike the `...` entry
    /// points, which live in dll/cpl/desk/rust_varargs_shim.c.
    fn _vsnwprintf(buf: *mut WCHAR, n: usize, fmt: *const WCHAR, ap: *mut c_void) -> c_int;
}

/// strsafe.h `StringCopyWorkerW`
unsafe fn StringCopyWorkerW(
    mut pszDest: *mut WCHAR,
    mut cchDest: usize,
    mut pszSrc: *const WCHAR,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        while cchDest != 0 && *pszSrc != 0 {
            *pszDest = *pszSrc;
            pszDest = pszDest.add(1);
            pszSrc = pszSrc.add(1);
            cchDest -= 1;
        }
        if cchDest == 0 {
            pszDest = pszDest.sub(1);
            hr = STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest = 0;
    }
    hr
}

/// strsafe.h `StringLengthWorkerW`
unsafe fn StringLengthWorkerW(
    mut psz: *const WCHAR,
    mut cchMax: usize,
    pcchLength: *mut usize,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let cchMaxPrev = cchMax;
    while cchMax != 0 && *psz != 0 {
        psz = psz.add(1);
        cchMax -= 1;
    }
    if cchMax == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    }
    if !pcchLength.is_null() {
        *pcchLength = if hr >= 0 { cchMaxPrev - cchMax } else { 0 };
    }
    hr
}

/// strsafe.h `StringCatWorkerW`
unsafe fn StringCatWorkerW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    let mut cchDestLength: usize = 0;
    let mut hr = StringLengthWorkerW(pszDest, cchDest, &mut cchDestLength);
    if hr >= 0 {
        hr = StringCopyWorkerW(pszDest.add(cchDestLength), cchDest - cchDestLength, pszSrc);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn StringCchCopyW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCbCopyW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<WCHAR>();
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCchCatW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCatWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCbCatW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<WCHAR>();
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCatWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCchLengthW(
    psz: *const WCHAR,
    cchMax: usize,
    pcchLength: *mut usize,
) -> HRESULT {
    if psz.is_null() || cchMax > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringLengthWorkerW(psz, cchMax, pcchLength)
}

/// strsafe.h `StringVPrintfWorkerW`, and the fixed-arity half of `StringCchPrintfW`.
///
/// The variadic entry point cannot live here: stable Rust can declare but not DEFINE a
/// C-variadic (`va_start`, rust-lang/rust#44930), so it is a trampoline in
/// rust_varargs_shim.c that does the `va_start` and calls this. `ap` arrives as a `void*`
/// because va_list is a single `char*` on all three targets.
#[no_mangle]
pub unsafe extern "C" fn StringCchPrintfW_v(
    mut pszDest: *mut WCHAR,
    cchDest: usize,
    pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut hr: HRESULT = S_OK;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        let cchMax = cchDest - 1;
        let iRet = _vsnwprintf(pszDest, cchMax, pszFormat, ap);
        if iRet < 0 || (iRet as usize) > cchMax {
            pszDest = pszDest.add(cchMax);
            *pszDest = 0;
            hr = STRSAFE_E_INSUFFICIENT_BUFFER;
        } else if (iRet as usize) == cchMax {
            pszDest = pszDest.add(cchMax);
            *pszDest = 0;
        }
    }
    hr
}

/// The byte-counted twin. strsafe.h derives it from the same worker after dividing.
#[no_mangle]
pub unsafe extern "C" fn StringCbPrintfW_v(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    StringCchPrintfW_v(pszDest, cbDest / core::mem::size_of::<WCHAR>(), pszFormat, ap)
}

/// strsafe.h:1646 `StringVPrintfExWorkerW`. Ported in full rather than trimmed to the flags desk
/// happens to pass (it passes 0): the whole value of this API is that a caller can add
/// STRSAFE_FILL_ON_FAILURE later and have it work, and a partial port is a landmine that only
/// goes off for the person who does.
unsafe fn StringVPrintfExWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    cbDest: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
    mut pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut pszDestEnd: *mut WCHAR = pszDest;
    let mut cchRemaining: usize = 0;
    if dwFlags & !STRSAFE_VALID_FLAGS != 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        if dwFlags & STRSAFE_IGNORE_NULLS != 0 {
            if pszDest.is_null() && (cchDest != 0 || cbDest != 0) {
                hr = STRSAFE_E_INVALID_PARAMETER;
            }
            if pszFormat.is_null() {
                pszFormat = crate::w!("");
            }
        }
        if hr >= 0 {
            if cchDest == 0 {
                pszDestEnd = pszDest;
                cchRemaining = 0;
                if *pszFormat != 0 {
                    hr = if pszDest.is_null() {
                        STRSAFE_E_INVALID_PARAMETER
                    } else {
                        STRSAFE_E_INSUFFICIENT_BUFFER
                    };
                }
            } else {
                let cchMax = cchDest - 1;
                let iRet = _vsnwprintf(pszDest, cchMax, pszFormat, ap);
                if iRet < 0 || (iRet as usize) > cchMax {
                    pszDestEnd = pszDest.add(cchMax);
                    cchRemaining = 1;
                    *pszDestEnd = 0;
                    hr = STRSAFE_E_INSUFFICIENT_BUFFER;
                } else if (iRet as usize) == cchMax {
                    pszDestEnd = pszDest.add(cchMax);
                    cchRemaining = 1;
                    *pszDestEnd = 0;
                } else {
                    pszDestEnd = pszDest.add(iRet as usize);
                    cchRemaining = cchDest - iRet as usize;
                    if dwFlags & STRSAFE_FILL_BEHIND_NULL != 0 {
                        memset(
                            pszDestEnd.add(1) as *mut c_void,
                            fill_pattern(dwFlags),
                            (cchRemaining - 1) * core::mem::size_of::<WCHAR>()
                                + (cbDest % core::mem::size_of::<WCHAR>()),
                        );
                    }
                }
            }
        }
    }
    if hr < 0 && !pszDest.is_null() {
        if dwFlags & STRSAFE_FILL_ON_FAILURE != 0 {
            memset(pszDest as *mut c_void, fill_pattern(dwFlags), cbDest);
            if fill_pattern(dwFlags) == 0 {
                pszDestEnd = pszDest;
                cchRemaining = cchDest;
            } else if cchDest > 0 {
                pszDestEnd = pszDest.add(cchDest - 1);
                cchRemaining = 1;
                *pszDestEnd = 0;
            }
        }
        if dwFlags & (STRSAFE_NULL_ON_FAILURE | STRSAFE_NO_TRUNCATION) != 0 && cchDest > 0 {
            pszDestEnd = pszDest;
            cchRemaining = cchDest;
            *pszDestEnd = 0;
        }
    }
    if hr >= 0 || hr == STRSAFE_E_INSUFFICIENT_BUFFER {
        if !ppszDestEnd.is_null() {
            *ppszDestEnd = pszDestEnd;
        }
        if !pcchRemaining.is_null() {
            *pcchRemaining = cchRemaining;
        }
    }
    hr
}

/// strsafe.h:623 `StringCbPrintfExW`, minus the `va_start` its trampoline does.
/// NOTE the unit change on the way out: the worker reports CHARACTERS remaining and this
/// reports BYTES, plus whatever odd byte the caller's `cbDest` had. Dropping that conversion
/// would halve every subsequent bound in a `c += n; sizeRemain -= n` accumulation loop, which is
/// exactly how background.c builds its file-open filter string.
#[no_mangle]
pub unsafe extern "C" fn StringCbPrintfExW_v(
    pszDest: *mut WCHAR,
    cbDest: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcbRemaining: *mut usize,
    dwFlags: DWORD,
    pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<WCHAR>();
    let mut cchRemaining: usize = 0;
    let hr = if cchDest > STRSAFE_MAX_CCH {
        STRSAFE_E_INVALID_PARAMETER
    } else {
        StringVPrintfExWorkerW(
            pszDest,
            cchDest,
            cbDest,
            ppszDestEnd,
            &mut cchRemaining,
            dwFlags,
            pszFormat,
            ap,
        )
    };
    if (hr >= 0 || hr == STRSAFE_E_INSUFFICIENT_BUFFER) && !pcbRemaining.is_null() {
        *pcbRemaining = cchRemaining * core::mem::size_of::<WCHAR>()
            + (cbDest % core::mem::size_of::<WCHAR>());
    }
    hr
}
