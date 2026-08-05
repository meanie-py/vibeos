//! strsafe.h's bounded string ops, ported from the header.
//!
//! `STRSAFEAPI` is `static __inline HRESULT WINAPI`, so no object file in the tree defines
//! these — every C caller inlined its own copy, and flipping syssetup's .c took the last one
//! with it. Same situation and same treatment as shell32's `shellutils.h` inlines: port the
//! header body rather than re-home a wrapper in a .c, so the next flip cannot repeat it.
//!
//! Faithful to `sdk/include/psdk/strsafe.h`: the worker functions truncate and NUL-terminate on
//! overflow and report `STRSAFE_E_INSUFFICIENT_BUFFER`; they do NOT panic. That distinction is
//! the whole point of the API, and it is also why none of this uses slice indexing.
use crate::prelude::*;

// strsafe.h:83,103,104
// winerror.h `#define S_OK ((HRESULT)0L)` — a macro, so it never reaches the island.
pub const S_OK: HRESULT = 0;
pub const STRSAFE_MAX_CCH: usize = 2147483647;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x80070057u32 as HRESULT;

/// strsafe.h `StringCopyWorkerW`
unsafe fn StringCopyWorkerW(mut pszDest: *mut WCHAR, mut cchDest: usize, mut pszSrc: *const WCHAR) -> HRESULT {
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
unsafe fn StringLengthWorkerW(mut psz: *const WCHAR, mut cchMax: usize, pcchLength: *mut usize) -> HRESULT {
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
pub unsafe extern "system" fn StringCchCopyW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCbCopyW(pszDest: *mut WCHAR, cbDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<WCHAR>();
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyWorkerW(pszDest, cchDest, pszSrc)
}

#[no_mangle]
pub unsafe extern "system" fn StringCchCatW(pszDest: *mut WCHAR, cchDest: usize, pszSrc: *const WCHAR) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCatWorkerW(pszDest, cchDest, pszSrc)
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
    ap: *mut core::ffi::c_void,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut hr: HRESULT = S_OK;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        let cchMax = cchDest - 1;
        let iRet = _vsnwprintf(pszDest, cchMax, pszFormat, ap as va_list);
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
