//! `sdk/include/psdk/strsafe.h`, ported once.
//!
//! # Why this crate exists
//!
//! `STRSAFEAPI` is `static __inline HRESULT WINAPI`, so **no object file in the tree defines any
//! of these**. Every C caller inlines its own copy, which means the last `.c` a module flips
//! takes the module's only definition with it — and every flipped module has had to port the
//! header again. Four copies existed before this crate (msgina, syssetup, desk, and sysdm about
//! to be the fourth), no two of them the same subset, and one of them wrong (see below).
//!
//! # The type basis is core primitives, deliberately
//!
//! Consumers each have their own bindgen island, and a shared crate cannot depend on any of
//! them. It does not need to: `WCHAR`, `HRESULT` and `DWORD` are transparent aliases for `u16`,
//! `i32` and `u32` in every island in the tree, and Rust unifies type aliases structurally, so
//! a caller's `*mut WCHAR` *is* this crate's `*mut u16`. (Only fn-pointer ABI is nominal, and
//! nothing here takes a callback.)
//!
//! # Faithfulness
//!
//! The worker functions truncate, NUL-terminate and report `STRSAFE_E_INSUFFICIENT_BUFFER`;
//! they do NOT panic. That distinction is the entire point of the API, and it is also why none
//! of this uses slice indexing.
//!
//! msgina's copy did not have it: its `StringCchPrintfW!`/`StringCbPrintfW!` macros DROPPED the
//! count argument and forwarded to user32's `wsprintfW`, which bounds to 1024 wchars
//! internally. `shutdown.c`'s log-off string formats a resource-loaded format plus the user name
//! into a `WCHAR[256]` — so a long user name overran a stack buffer by up to 768 wchars where
//! the C truncated safely. Sharing one faithful port is what makes that class impossible.
//!
//! # Variadics
//!
//! Stable Rust can declare but not DEFINE a C-variadic (`va_start`, rust-lang/rust#44930), so
//! each consuming module carries a `rust_varargs_shim.c` whose 3-line trampolines do the
//! `va_start` and call the fixed-arity `*_v` twins here. The `va_list` is passed **by pointer**
//! (`&ap`), matching `varargs-rust`'s convention: `va_list` is a single `char*` on all three of
//! our Windows targets but an array-of-struct on SysV x86-64, and the by-pointer form is correct
//! on both — which is what lets the mechanism be exercised natively on the host.
#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]

pub type WCHAR = u16;
pub type HRESULT = i32;
pub type DWORD = u32;

use core::ffi::{c_int, c_void};

extern "C" {
    /// msvcrt's `_vsnwprintf`. Variadic only in its `va_list` argument, so — unlike the `...`
    /// entry points — this IS expressible in stable Rust.
    fn _vsnwprintf(buf: *mut WCHAR, n: usize, fmt: *const WCHAR, ap: *mut c_void) -> c_int;
    /// Imported, never hand-written: LLVM folds any byte-copy loop back into a call to the
    /// function itself, which is an infinite self-recursion (AGENTS.md).
    fn memset(dest: *mut c_void, ch: c_int, count: usize) -> *mut c_void;
}

// strsafe.h:83,86-100,103,104. `S_OK` is a winerror.h MACRO, so it never reaches any island.
pub const S_OK: HRESULT = 0;
pub const STRSAFE_MAX_CCH: usize = 2147483647;
pub const STRSAFE_IGNORE_NULLS: DWORD = 0x0000_0100;
pub const STRSAFE_FILL_BEHIND_NULL: DWORD = 0x0000_0200;
pub const STRSAFE_FILL_ON_FAILURE: DWORD = 0x0000_0400;
pub const STRSAFE_NULL_ON_FAILURE: DWORD = 0x0000_0800;
pub const STRSAFE_NO_TRUNCATION: DWORD = 0x0000_1000;
pub const STRSAFE_VALID_FLAGS: DWORD = 0x0000_00FF
    | STRSAFE_IGNORE_NULLS
    | STRSAFE_FILL_BEHIND_NULL
    | STRSAFE_FILL_ON_FAILURE
    | STRSAFE_NULL_ON_FAILURE
    | STRSAFE_NO_TRUNCATION;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;
pub const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x8007_0057u32 as HRESULT;

/// strsafe.h:100 `#define STRSAFE_GET_FILL_PATTERN(dwFlags) ((int)((dwFlags) & 0x000000FF))`
#[inline(always)]
fn fill_pattern(dwFlags: DWORD) -> c_int {
    (dwFlags & 0x0000_00FF) as c_int
}

/// The empty format `STRSAFE_IGNORE_NULLS` substitutes for a NULL one.
static EMPTY: [WCHAR; 1] = [0];
// ============================ workers ============================
// strsafe.h's `*WorkerW` functions, ported verbatim. Not exported: the header does not export
// them either, and every public entry point below is a thin bound-check plus a worker call.

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

/// strsafe.h:1172 `StringCopyNExWorkerW`.
///
/// Ported in full rather than trimmed to the flags today's callers happen to pass. The whole
/// value of the Ex family is that a caller can add `STRSAFE_FILL_ON_FAILURE` later and have it
/// work; a partial port is a landmine that only goes off for the person who does. (msgina's
/// hand-approximated copy dropped the fill paths, the `IGNORE_NULLS` handling, and got
/// `pcbRemaining` wrong — that is the copy this crate replaces.)
#[allow(clippy::too_many_arguments)]
unsafe fn StringCopyNExWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    cbDest: usize,
    mut pszSrc: *const WCHAR,
    mut cchToCopy: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut pszDestEnd: *mut WCHAR = pszDest;
    let mut cchRemaining: usize = 0;
    if dwFlags & !STRSAFE_VALID_FLAGS != 0 || cchToCopy > STRSAFE_MAX_CCH {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        if dwFlags & STRSAFE_IGNORE_NULLS != 0 {
            if pszDest.is_null() && (cchDest != 0 || cbDest != 0) {
                hr = STRSAFE_E_INVALID_PARAMETER;
            }
            if pszSrc.is_null() {
                pszSrc = EMPTY.as_ptr();
            }
        }
        if hr >= 0 {
            if cchDest == 0 {
                pszDestEnd = pszDest;
                cchRemaining = 0;
                if cchToCopy != 0 && *pszSrc != 0 {
                    hr = if pszDest.is_null() {
                        STRSAFE_E_INVALID_PARAMETER
                    } else {
                        STRSAFE_E_INSUFFICIENT_BUFFER
                    };
                }
            } else {
                pszDestEnd = pszDest;
                cchRemaining = cchDest;
                while cchRemaining != 0 && cchToCopy != 0 && *pszSrc != 0 {
                    *pszDestEnd = *pszSrc;
                    pszDestEnd = pszDestEnd.add(1);
                    pszSrc = pszSrc.add(1);
                    cchRemaining -= 1;
                    cchToCopy -= 1;
                }
                if cchRemaining > 0 {
                    if dwFlags & STRSAFE_FILL_BEHIND_NULL != 0 {
                        memset(
                            pszDestEnd.add(1) as *mut c_void,
                            fill_pattern(dwFlags),
                            (cchRemaining - 1) * core::mem::size_of::<WCHAR>()
                                + (cbDest % core::mem::size_of::<WCHAR>()),
                        );
                    }
                } else {
                    pszDestEnd = pszDestEnd.sub(1);
                    cchRemaining += 1;
                    hr = STRSAFE_E_INSUFFICIENT_BUFFER;
                }
                *pszDestEnd = 0;
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

/// strsafe.h:1646 `StringVPrintfExWorkerW`.
#[allow(clippy::too_many_arguments)]
unsafe fn StringVPrintfExWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    cbDest: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
    pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    printf_ex_worker(pszDest, cchDest, cbDest, ppszDestEnd, pcchRemaining, dwFlags, pszFormat,
        |d, n, f| _vsnwprintf(d, n, f, ap))
}

/// `StringVPrintfExWorkerW` with its one `_vsnwprintf` call handed in as `print(dest, cchMax,
/// format)`, so the variadic macro (`StringCbPrintfExW!`, which calls `_snwprintf` with the
/// arguments in place) and the `va_list` entry points share every flag and bound rule.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub unsafe fn printf_ex_worker(
    pszDest: *mut WCHAR,
    cchDest: usize,
    cbDest: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
    mut pszFormat: *const WCHAR,
    print: impl FnOnce(*mut WCHAR, usize, *const WCHAR) -> c_int,
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
                pszFormat = EMPTY.as_ptr();
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
                let iRet = print(pszDest, cchMax, pszFormat);
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

// ============================ public entry points ============================
// `#[no_mangle] extern "system"` so a module's remaining C (its varargs shim, or a partially
// flipped TU) links against the same definition the Rust callers use.

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

/// strsafe.h:275 `StringCchCopyNExW`.
#[no_mangle]
pub unsafe extern "system" fn StringCchCopyNExW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
    cchToCopy: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCopyNExWorkerW(
        pszDest,
        cchDest,
        cchDest * core::mem::size_of::<WCHAR>(),
        pszSrc,
        cchToCopy,
        ppszDestEnd,
        pcchRemaining,
        dwFlags,
    )
}

/// strsafe.h:299 `StringCbCopyNExW`.
///
/// NOTE the unit change on the way out: the worker reports CHARACTERS remaining and this
/// reports BYTES, plus whatever odd byte the caller's `cbDest` had. Dropping that conversion
/// halves every subsequent bound in a `p += n; remaining -= n` accumulation loop.
#[no_mangle]
pub unsafe extern "system" fn StringCbCopyNExW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
    cbToCopy: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcbRemaining: *mut usize,
    dwFlags: DWORD,
) -> HRESULT {
    let cchDest = cbDest / core::mem::size_of::<WCHAR>();
    let mut cchRemaining: usize = 0;
    let hr = if cchDest > STRSAFE_MAX_CCH {
        STRSAFE_E_INVALID_PARAMETER
    } else {
        StringCopyNExWorkerW(
            pszDest,
            cchDest,
            cbDest,
            pszSrc,
            cbToCopy / core::mem::size_of::<WCHAR>(),
            ppszDestEnd,
            &mut cchRemaining,
            dwFlags,
        )
    };
    if (hr >= 0 || hr == STRSAFE_E_INSUFFICIENT_BUFFER) && !pcbRemaining.is_null() {
        *pcbRemaining =
            cchRemaining * core::mem::size_of::<WCHAR>() + (cbDest % core::mem::size_of::<WCHAR>());
    }
    hr
}

// ---- the printf family, called DIRECTLY: no C trampoline at all ----
//
// Rust cannot DEFINE a C-variadic, but CALLING one has been stable for years. So a Rust
// caller does not need `StringCchPrintfW` to exist as a symbol (it is `static __inline` in
// strsafe.h, a phantom), nor a per-module C shim that defines one: these macros call msvcrt's
// variadic `_snwprintf` in place of the worker's `_vsnwprintf` and then apply
// `StringVPrintfWorkerW`'s truncation contract through `printf_worker_finish` below. Same
// arguments, same bounds, same HRESULTs. The argument types are checked by rustc's variadic
// rules, which reject exactly the types C's default promotions would have widened.

extern "C" {
    /// msvcrt's `_snwprintf`: `_vsnwprintf` with the arguments in place of the `va_list`.
    #[doc(hidden)]
    pub fn _snwprintf(buf: *mut WCHAR, n: usize, fmt: *const WCHAR, ...) -> c_int;
}

/// strsafe.h `StringVPrintfWorkerW` after its `_vsnwprintf` call, given that call's result.
/// `cchDest` is the caller's count, already checked to be non-zero.
#[doc(hidden)]
pub unsafe fn printf_worker_finish(pszDest: *mut WCHAR, cchDest: usize, iRet: c_int) -> HRESULT {
    let cchMax = cchDest - 1;
    if iRet < 0 || (iRet as usize) > cchMax {
        *pszDest.add(cchMax) = 0;
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        if (iRet as usize) == cchMax {
            *pszDest.add(cchMax) = 0;
        }
        S_OK
    }
}

/// strsafe.h `StringCchPrintfW(pszDest, cchDest, pszFormat, ...)`, written as a macro call:
/// `StringCchPrintfW!(dest, cch, fmt, args...)`. Evaluates to the `HRESULT`.
#[macro_export]
macro_rules! StringCchPrintfW {
    ($pszDest:expr, $cchDest:expr, $pszFormat:expr $(, $arg:expr)* $(,)?) => {{
        let pszDest: *mut $crate::WCHAR = $pszDest as _;
        let cchDest: usize = $cchDest as usize;
        if cchDest > $crate::STRSAFE_MAX_CCH || cchDest == 0 {
            $crate::STRSAFE_E_INVALID_PARAMETER
        } else {
            let iRet = $crate::_snwprintf(pszDest, cchDest - 1, $pszFormat as _ $(, $arg)*);
            $crate::printf_worker_finish(pszDest, cchDest, iRet)
        }
    }};
}

/// strsafe.h `StringCbPrintfW`: the byte-counted form of `StringCchPrintfW!`.
#[macro_export]
macro_rules! StringCbPrintfW {
    ($pszDest:expr, $cbDest:expr, $pszFormat:expr $(, $arg:expr)* $(,)?) => {
        $crate::StringCchPrintfW!(
            $pszDest,
            ($cbDest as usize) / core::mem::size_of::<$crate::WCHAR>(),
            $pszFormat
            $(, $arg)*
        )
    };
}

/// strsafe.h `StringCbPrintfExW(pszDest, cbDest, ppszDestEnd, pcbRemaining, dwFlags,
/// pszFormat, ...)` as a macro call. Evaluates to the `HRESULT`.
#[macro_export]
macro_rules! StringCbPrintfExW {
    ($pszDest:expr, $cbDest:expr, $ppszDestEnd:expr, $pcbRemaining:expr, $dwFlags:expr,
     $pszFormat:expr $(, $arg:expr)* $(,)?) => {{
        let pszDest: *mut $crate::WCHAR = $pszDest as _;
        let cbDest: usize = $cbDest as usize;
        let pcbRemaining: *mut usize = $pcbRemaining as _;
        let cchDest = cbDest / core::mem::size_of::<$crate::WCHAR>();
        let mut cchRemaining: usize = 0;
        let hr = if cchDest > $crate::STRSAFE_MAX_CCH {
            $crate::STRSAFE_E_INVALID_PARAMETER
        } else {
            $crate::printf_ex_worker(
                pszDest, cchDest, cbDest, $ppszDestEnd as _, &mut cchRemaining,
                $dwFlags as _, $pszFormat as _,
                |d, n, f| $crate::_snwprintf(d, n, f $(, $arg)*),
            )
        };
        if (hr >= 0 || hr == $crate::STRSAFE_E_INSUFFICIENT_BUFFER) && !pcbRemaining.is_null() {
            *pcbRemaining = cchRemaining * core::mem::size_of::<$crate::WCHAR>()
                + (cbDest % core::mem::size_of::<$crate::WCHAR>());
        }
        hr
    }};
}

// ---- the printf family: fixed-arity twins of the variadic entry points ----
// Each `_v` takes a POINTER to the caller's va_list, which is what
// `varargs-rust/rust_varargs_shim.c.template` hands over. `_vsnwprintf`'s last parameter is a
// va_list BY VALUE, so each of these dereferences once on the way through.

/// strsafe.h `StringVPrintfWorkerW` plus `StringCchPrintfW`'s bound check.
#[no_mangle]
pub unsafe extern "C" fn StringCchPrintfW_v(
    mut pszDest: *mut WCHAR,
    cchDest: usize,
    pszFormat: *const WCHAR,
    app: *mut c_void,
) -> HRESULT {
    if cchDest > STRSAFE_MAX_CCH {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut hr: HRESULT = S_OK;
    if cchDest == 0 {
        hr = STRSAFE_E_INVALID_PARAMETER;
    } else {
        let cchMax = cchDest - 1;
        let iRet = _vsnwprintf(pszDest, cchMax, pszFormat, *(app as *mut *mut c_void));
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

/// The byte-counted twin.
#[no_mangle]
pub unsafe extern "C" fn StringCbPrintfW_v(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszFormat: *const WCHAR,
    app: *mut c_void,
) -> HRESULT {
    StringCchPrintfW_v(pszDest, cbDest / core::mem::size_of::<WCHAR>(), pszFormat, app)
}

/// strsafe.h `StringCchVPrintfW` — NOT variadic in C either (it takes the `va_list`), so this
/// one needs no trampoline and callers reach it directly.
#[no_mangle]
pub unsafe extern "system" fn StringCchVPrintfW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszFormat: *const WCHAR,
    ap: *mut c_void,
) -> HRESULT {
    // `ap` arrives BY VALUE here (it is a real `va_list` parameter, not the shim's `&ap`), so
    // wrap it to reuse the `_v` body rather than duplicating the truncation logic.
    let mut by_value = ap;
    StringCchPrintfW_v(
        pszDest,
        cchDest,
        pszFormat,
        &mut by_value as *mut *mut c_void as *mut c_void,
    )
}

/// strsafe.h:623 `StringCbPrintfExW`, minus the `va_start` its trampoline does.
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn StringCbPrintfExW_v(
    pszDest: *mut WCHAR,
    cbDest: usize,
    ppszDestEnd: *mut *mut WCHAR,
    pcbRemaining: *mut usize,
    dwFlags: DWORD,
    pszFormat: *const WCHAR,
    app: *mut c_void,
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
            *(app as *mut *mut c_void),
        )
    };
    if (hr >= 0 || hr == STRSAFE_E_INSUFFICIENT_BUFFER) && !pcbRemaining.is_null() {
        *pcbRemaining =
            cchRemaining * core::mem::size_of::<WCHAR>() + (cbDest % core::mem::size_of::<WCHAR>());
    }
    hr
}
