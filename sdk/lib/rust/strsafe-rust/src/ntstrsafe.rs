//! `sdk/include/ddk/ntstrsafe.h`, the NTSTATUS twin of strsafe.h, ported once.
//!
//! Every function here is `NTSTRSAFEAPI` (`static __inline NTSTATUS NTAPI`), so, exactly like
//! strsafe.h, no object file defines any of them: a translated module calling
//! `RtlStringCchCopyW` either links this or names a symbol nothing exports. Tree consensus had
//! already copied `extern` declarations of five of them into setuplib's prelude — they compile,
//! and fail at link.
//!
//! Transcribed from the header, NOT aliased onto the HRESULT family next door: the two differ
//! in more than the codes. ntstrsafe's Cch entry points NUL the destination when `cchDest`
//! exceeds the maximum, and its workers report `STATUS_BUFFER_OVERFLOW` (an NTSTATUS warning,
//! `NT_SUCCESS` false) where strsafe reports an HRESULT failure. Line numbers are ntstrsafe.h's.
//!
//! The printf entry points are variadic, which stable Rust can declare but not define, so they
//! are macros over msvcrt's `_snwprintf` (declared in the crate root) — the same shape as
//! [`StringCchPrintfW!`](crate::StringCchPrintfW).

use crate::WCHAR;

pub type NTSTATUS = i32;

/// ntstrsafe.h:23.
pub const NTSTRSAFE_MAX_CCH: usize = 2147483647;
/// ntstatus.h: `((NTSTATUS)0x80000005)` and `((NTSTATUS)0xC000000D)`.
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as i32;
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as i32;
const STATUS_SUCCESS: NTSTATUS = 0;

/// :3170 `RtlStringLengthWorkerW`.
pub unsafe fn RtlStringLengthWorkerW(
    mut psz: *const WCHAR,
    mut cchMax: usize,
    pcchLength: *mut usize,
) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;
    let cch_max_prev = cchMax;
    while cchMax != 0 && *psz != 0 {
        psz = psz.add(1);
        cchMax -= 1;
    }
    if cchMax == 0 {
        status = STATUS_INVALID_PARAMETER;
    }
    if !pcchLength.is_null() {
        *pcchLength = if status >= 0 {
            cch_max_prev - cchMax
        } else {
            0
        };
    }
    status
}

/// :1702 `RtlStringCopyWorkerW`, and :2001 `RtlStringCopyNWorkerW` with `cchToCopy` unbounded.
unsafe fn copy_worker<T: Copy + PartialEq + Default>(
    mut pszDest: *mut T,
    mut cchDest: usize,
    mut pszSrc: *const T,
    mut cchToCopy: usize,
) -> NTSTATUS {
    if cchDest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let nul = T::default();
    let mut status = STATUS_SUCCESS;
    while cchDest != 0 && cchToCopy != 0 && *pszSrc != nul {
        *pszDest = *pszSrc;
        pszDest = pszDest.add(1);
        pszSrc = pszSrc.add(1);
        cchDest -= 1;
        cchToCopy -= 1;
    }
    if cchDest == 0 {
        pszDest = pszDest.sub(1);
        status = STATUS_BUFFER_OVERFLOW;
    }
    *pszDest = nul;
    status
}

/// :1702.
pub unsafe fn RtlStringCopyWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    copy_worker(pszDest, cchDest, pszSrc, usize::MAX)
}

/// :2001.
pub unsafe fn RtlStringCopyNWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
    cchToCopy: usize,
) -> NTSTATUS {
    copy_worker(pszDest, cchDest, pszSrc, cchToCopy)
}

/// :2298 `RtlStringCatWorkerW`.
pub unsafe fn RtlStringCatWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    RtlStringCatNWorkerW(pszDest, cchDest, pszSrc, usize::MAX)
}

/// :2548 `RtlStringCatNWorkerW`.
pub unsafe fn RtlStringCatNWorkerW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
    cchToAppend: usize,
) -> NTSTATUS {
    let mut len = 0usize;
    let status = RtlStringLengthWorkerW(pszDest, cchDest, &mut len);
    if status < 0 {
        return status;
    }
    copy_worker(pszDest.add(len), cchDest - len, pszSrc, cchToAppend)
}

/// :127 `RtlStringCchCopyW`.
pub unsafe fn RtlStringCchCopyW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    if cchDest > NTSTRSAFE_MAX_CCH {
        if cchDest > 0 {
            *pszDest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }
    RtlStringCopyWorkerW(pszDest, cchDest, pszSrc)
}

/// `RtlStringCbCopyW`: the Cch form with `cbDest / sizeof(wchar_t)`.
pub unsafe fn RtlStringCbCopyW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    RtlStringCchCopyW(pszDest, cbDest / core::mem::size_of::<WCHAR>(), pszSrc)
}

/// `RtlStringCchCopyNW`.
pub unsafe fn RtlStringCchCopyNW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
    cchToCopy: usize,
) -> NTSTATUS {
    if cchDest > NTSTRSAFE_MAX_CCH || cchToCopy > NTSTRSAFE_MAX_CCH {
        if cchDest > 0 {
            *pszDest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }
    RtlStringCopyNWorkerW(pszDest, cchDest, pszSrc, cchToCopy)
}

/// `RtlStringCbCopyNW`.
pub unsafe fn RtlStringCbCopyNW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
    cbToCopy: usize,
) -> NTSTATUS {
    let w = core::mem::size_of::<WCHAR>();
    RtlStringCchCopyNW(pszDest, cbDest / w, pszSrc, cbToCopy / w)
}

/// `RtlStringCbCopyNA`: the narrow twin, `sizeof(char)` is 1.
pub unsafe fn RtlStringCbCopyNA(
    pszDest: *mut i8,
    cbDest: usize,
    pszSrc: *const i8,
    cbToCopy: usize,
) -> NTSTATUS {
    if cbDest > NTSTRSAFE_MAX_CCH || cbToCopy > NTSTRSAFE_MAX_CCH {
        if cbDest > 0 {
            *pszDest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }
    copy_worker(pszDest, cbDest, pszSrc, cbToCopy)
}

/// `RtlStringCchCatW`. Note: unlike the Copy entry points, Cat does NOT nul an oversized
/// destination (the header returns at once).
pub unsafe fn RtlStringCchCatW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    if cchDest > NTSTRSAFE_MAX_CCH {
        return STATUS_INVALID_PARAMETER;
    }
    RtlStringCatWorkerW(pszDest, cchDest, pszSrc)
}

/// `RtlStringCbCatW`.
pub unsafe fn RtlStringCbCatW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
) -> NTSTATUS {
    RtlStringCchCatW(pszDest, cbDest / core::mem::size_of::<WCHAR>(), pszSrc)
}

/// `RtlStringCchCatNW`.
pub unsafe fn RtlStringCchCatNW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszSrc: *const WCHAR,
    cchToAppend: usize,
) -> NTSTATUS {
    if cchDest > NTSTRSAFE_MAX_CCH {
        return STATUS_INVALID_PARAMETER;
    }
    RtlStringCatNWorkerW(pszDest, cchDest, pszSrc, cchToAppend)
}

/// `RtlStringCbCatNW`.
pub unsafe fn RtlStringCbCatNW(
    pszDest: *mut WCHAR,
    cbDest: usize,
    pszSrc: *const WCHAR,
    cbToAppend: usize,
) -> NTSTATUS {
    let w = core::mem::size_of::<WCHAR>();
    RtlStringCchCatNW(pszDest, cbDest / w, pszSrc, cbToAppend / w)
}

/// :2853 `RtlpStringVPrintfWorkerW`'s tail, given `_vsnwprintf`'s return for `cchDest - 1`.
pub unsafe fn ntstrsafe_printf_finish(pszDest: *mut WCHAR, cchDest: usize, iRet: i32) -> NTSTATUS {
    let cch_max = cchDest - 1;
    if iRet < 0 || iRet as usize > cch_max {
        *pszDest.add(cch_max) = 0;
        return STATUS_BUFFER_OVERFLOW;
    }
    if iRet as usize == cch_max {
        *pszDest.add(cch_max) = 0;
    }
    STATUS_SUCCESS
}

/// :1110 `RtlStringCchPrintfW(pszDest, cchDest, pszFormat, ...)`.
#[macro_export]
macro_rules! RtlStringCchPrintfW {
    ($pszDest:expr, $cchDest:expr, $pszFormat:expr $(, $arg:expr)* $(,)?) => {{
        let pszDest: *mut $crate::WCHAR = $pszDest as _;
        let cchDest: usize = $cchDest as usize;
        if cchDest > $crate::ntstrsafe::NTSTRSAFE_MAX_CCH {
            if cchDest > 0 {
                *pszDest = 0;
            }
            $crate::ntstrsafe::STATUS_INVALID_PARAMETER
        } else if cchDest == 0 {
            // RtlStringVPrintfWorkerW (:2882) refuses an empty buffer before formatting.
            $crate::ntstrsafe::STATUS_INVALID_PARAMETER
        } else {
            let iRet = $crate::_snwprintf(pszDest, cchDest - 1, $pszFormat as _ $(, $arg)*);
            $crate::ntstrsafe::ntstrsafe_printf_finish(pszDest, cchDest, iRet)
        }
    }};
}

/// `RtlStringCbPrintfW`: the Cch form with `cbDest / sizeof(wchar_t)`.
#[macro_export]
macro_rules! RtlStringCbPrintfW {
    ($pszDest:expr, $cbDest:expr, $pszFormat:expr $(, $arg:expr)* $(,)?) => {
        $crate::RtlStringCchPrintfW!(
            $pszDest,
            ($cbDest as usize) / core::mem::size_of::<$crate::WCHAR>(),
            $pszFormat
            $(, $arg)*
        )
    };
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn copy_truncates_and_reports_overflow() {
        let src = w("abcdef");
        let mut buf = [0xFFFFu16; 4];
        let st = unsafe { RtlStringCchCopyW(buf.as_mut_ptr(), 4, src.as_ptr()) };
        assert_eq!(st, STATUS_BUFFER_OVERFLOW);
        assert_eq!(&buf, &[b'a' as u16, b'b' as u16, b'c' as u16, 0]);
        let mut big = [0u16; 8];
        assert_eq!(
            unsafe { RtlStringCchCopyW(big.as_mut_ptr(), 8, src.as_ptr()) },
            0
        );
        assert_eq!(&big[..7], &src[..]);
        assert_eq!(
            unsafe { RtlStringCchCopyW(big.as_mut_ptr(), 0, src.as_ptr()) },
            STATUS_INVALID_PARAMETER
        );
    }

    #[test]
    fn oversized_copy_nuls_but_oversized_cat_does_not() {
        let src = w("x");
        let mut buf = [7u16; 2];
        assert_eq!(
            unsafe { RtlStringCchCopyW(buf.as_mut_ptr(), NTSTRSAFE_MAX_CCH + 1, src.as_ptr()) },
            STATUS_INVALID_PARAMETER
        );
        assert_eq!(buf[0], 0);
        let mut buf = [7u16; 2];
        assert_eq!(
            unsafe { RtlStringCchCatW(buf.as_mut_ptr(), NTSTRSAFE_MAX_CCH + 1, src.as_ptr()) },
            STATUS_INVALID_PARAMETER
        );
        assert_eq!(buf[0], 7);
    }

    #[test]
    fn cat_appends_within_bounds_and_cb_divides_by_two() {
        let mut buf = [0u16; 8];
        let a = w("ab");
        let b = w("cdefgh");
        unsafe {
            assert_eq!(RtlStringCbCopyW(buf.as_mut_ptr(), 16, a.as_ptr()), 0);
            assert_eq!(
                RtlStringCbCatW(buf.as_mut_ptr(), 16, b.as_ptr()),
                STATUS_BUFFER_OVERFLOW
            );
        }
        assert_eq!(&buf, &w("abcdefg")[..]);
        let mut buf = [0u16; 8];
        unsafe {
            RtlStringCchCopyW(buf.as_mut_ptr(), 8, a.as_ptr());
            assert_eq!(RtlStringCbCatNW(buf.as_mut_ptr(), 16, b.as_ptr(), 4), 0);
        }
        assert_eq!(&buf[..5], &w("abcd")[..]);
    }

    #[test]
    fn length_worker_fails_without_a_terminator_inside_the_bound() {
        let s = [b'a' as u16; 3];
        let mut n = 99usize;
        assert_eq!(
            unsafe { RtlStringLengthWorkerW(s.as_ptr(), 3, &mut n) },
            STATUS_INVALID_PARAMETER
        );
        assert_eq!(n, 0);
    }

    #[test]
    fn printf_finish_matches_the_worker() {
        let mut buf = [1u16; 4];
        assert_eq!(
            unsafe { ntstrsafe_printf_finish(buf.as_mut_ptr(), 4, -1) },
            STATUS_BUFFER_OVERFLOW
        );
        assert_eq!(buf[3], 0);
        let mut buf = [1u16; 4];
        assert_eq!(
            unsafe { ntstrsafe_printf_finish(buf.as_mut_ptr(), 4, 3) },
            0
        );
        assert_eq!(buf[3], 0);
        let mut buf = [1u16; 4];
        assert_eq!(
            unsafe { ntstrsafe_printf_finish(buf.as_mut_ptr(), 4, 2) },
            0
        );
        assert_eq!(buf[3], 1);
    }
}
