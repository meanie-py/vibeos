//! strsafe.h inline string helpers (no link symbol — implemented faithfully in Rust).
//!
//! The copy-family (StringCbCopyW / StringCbCopyNExW) is implemented directly. The
//! printf-family (StringCchPrintfW / StringCbPrintfW) is variadic; Rust cannot *define*
//! a C-variadic function on stable, so those two are provided as macros in lib.rs that
//! forward to user32's variadic `wsprintfW` (the same vsnwprintf-class engine strsafe
//! wraps). The copy helpers return an HRESULT (S_OK / STRSAFE_E_INSUFFICIENT_BUFFER).
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::prelude::*;

// HRESULT / STRSAFE_LPWSTR / STRSAFE_LPCWSTR and the STRSAFE_* flag constants
// (NO_TRUNCATION=0x1000, NULL_ON_FAILURE=0x800) all come from crate::prelude (ntdll_sys).

pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x8007_0057u32 as HRESULT;

// S_OK is also in msgina_consts; redefine here to keep strsafe self-contained.
const S_OK_HR: HRESULT = 0;

/// StringCbCopyW(pszDest, cbDest, pszSrc): copy a NUL-terminated wide string into a
/// byte-sized destination buffer, always NUL-terminating, never overflowing.
#[inline]
pub unsafe fn StringCbCopyW(pszDest: PWSTR, cbDest: usize, pszSrc: PCWSTR) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<u16>();
    if cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i: usize = 0;
    while i < cchDest - 1 && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0u16;
    if *pszSrc.add(i) != 0 {
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        S_OK_HR
    }
}

/// StringCbCopyNExW(pszDest, cbDest, pszSrc, cbToCopy, ppszDestEnd, pcbRemaining, dwFlags):
/// copy up to cbToCopy bytes of pszSrc into the byte-sized pszDest, NUL-terminating.
/// Honors STRSAFE_NO_TRUNCATION / STRSAFE_NULL_ON_FAILURE (faithful subset).
#[inline]
pub unsafe fn StringCbCopyNExW(
    pszDest: PWSTR,
    cbDest: usize,
    pszSrc: PCWSTR,
    cbToCopy: usize,
    ppszDestEnd: *mut PWSTR,
    pcbRemaining: *mut usize,
    dwFlags: ULONG,
) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<u16>();
    let cchToCopy = cbToCopy / core::mem::size_of::<u16>();
    if cchDest == 0 {
        if !pszDest.is_null() && cbDest > 0 && (dwFlags & STRSAFE_NULL_ON_FAILURE) != 0 {
            // can't even write a NUL into a 0-cch buffer
        }
        return STRSAFE_E_INVALID_PARAMETER;
    }

    let mut i: usize = 0;
    let mut truncated = false;
    while i < cchToCopy && *pszSrc.add(i) != 0 {
        if i >= cchDest - 1 {
            truncated = true;
            break;
        }
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }

    if truncated && (dwFlags & STRSAFE_NO_TRUNCATION) != 0 {
        if (dwFlags & STRSAFE_NULL_ON_FAILURE) != 0 {
            *pszDest.add(0) = 0u16;
        }
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }

    *pszDest.add(i) = 0u16;
    if !ppszDestEnd.is_null() {
        *ppszDestEnd = pszDest.add(i);
    }
    if !pcbRemaining.is_null() {
        *pcbRemaining = (cchDest - i) * core::mem::size_of::<u16>();
    }
    S_OK_HR
}
