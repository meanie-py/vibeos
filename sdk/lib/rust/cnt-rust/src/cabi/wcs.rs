//! The kernel (`_LIBCNT_`) wide-string slice of ReactOS libcntpr
//! `string/wcs.c` — the 8 functions that compile in the kernel build
//! (everything else in wcs.c is `#ifndef _LIBCNT_`, i.e. msvcrt-only:
//! `_wcsdup`/`_wcsicoll`/`_wcsset`/`_wcsupr_s`/`wcstod`/`wctomb`/
//! `wcstombs`/`_wcstombs_l` — locale/float/heap, NOT here). All 8 are
//! pure algorithm (no locale, no float, no heap). Stitched into `cabi`
//! like `math.rs`/`strtod.rs` (bespoke wide-pointer bodies, not a
//! gen_cnt family). Faithful to the C including the `_LIBCNT_` macro
//! reductions: `MSVCRT_CHECK_PMT(x)` → `(x)`, `MSVCRT_INVALID_PMT` →
//! no-op, and no `_set_errno` (it is `#ifndef _LIBCNT_`).
//! Differential-pinned in cnt-difftest.

// <errno.h>/<corecrt.h> values used by the *_s contracts.
const EINVAL: i32 = 22;
const ERANGE: i32 = 34;
const STRUNCATE: i32 = 80;
const TRUNCATE: usize = usize::MAX; // _TRUNCATE == (size_t)-1

/// `strlenW` — wide NUL scan.
#[inline]
unsafe fn wlen(s: *const u16) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// `_wcsnset(str, c, n)` — set up to `n` leading chars to `c` (stops at
/// NUL). `while ((n-- > 0) && *str) *str++ = c;` — the trailing `n`
/// value is unused, so the post-decrement-on-exit is immaterial.
#[no_mangle]
pub unsafe extern "C" fn _wcsnset(str: *mut u16, c: u16, n: usize) -> *mut u16 {
    let ret = str;
    let mut p = str;
    let mut k = n;
    while k > 0 && *p != 0 {
        *p = c;
        p = p.add(1);
        k -= 1;
    }
    ret
}

/// `_wcsrev(str)` — reverse in place. `end = str + strlen - 1;` — for an
/// empty string `strlen == 0` so the C forms `str - 1` and the
/// `end > str` guard skips the loop; we early-return to avoid the Rust
/// pointer underflow while preserving the same (no-op) result.
#[no_mangle]
pub unsafe extern "C" fn _wcsrev(str: *mut u16) -> *mut u16 {
    let ret = str;
    let len = wlen(str);
    if len == 0 {
        return ret;
    }
    let mut s = str;
    let mut e = str.add(len - 1);
    while e > s {
        let t = *e;
        *e = *s;
        *s = t;
        e = e.sub(1);
        s = s.add(1);
    }
    ret
}

/// `wcscoll(a, b)` — the C is `return strcmpW(a, b);` (FIXME: handle
/// collates). `strcmpW` returns the signed difference of the first
/// differing WCHARs (unsigned), 0 if equal.
#[no_mangle]
pub unsafe extern "C" fn wcscoll(str1: *const u16, str2: *const u16) -> i32 {
    let mut a = str1;
    let mut b = str2;
    loop {
        let (ca, cb) = (*a, *b);
        if ca != cb {
            return ca as i32 - cb as i32;
        }
        if ca == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

/// `wcspbrk(str, accept)` — first char of `str` that is in `accept`.
#[no_mangle]
pub unsafe extern "C" fn wcspbrk(str: *const u16, accept: *const u16) -> *mut u16 {
    let mut s = str;
    while *s != 0 {
        let mut p = accept;
        while *p != 0 {
            if *p == *s {
                return s as *mut u16;
            }
            p = p.add(1);
        }
        s = s.add(1);
    }
    core::ptr::null_mut()
}

/// `wcscpy_s(dst, num, src)` — bounded wide copy.
#[no_mangle]
pub unsafe extern "C" fn wcscpy_s(dst: *mut u16, num: usize, src: *const u16) -> i32 {
    if dst.is_null() || num == 0 {
        return EINVAL;
    }
    *dst = 0;
    if src.is_null() {
        return EINVAL;
    }
    let size = wlen(src) + 1;
    if size > num {
        return ERANGE;
    }
    core::ptr::copy_nonoverlapping(src, dst, size);
    0
}

/// `wcsncpy_s(dst, num, src, count)` — bounded wide copy of `<=count`.
#[no_mangle]
pub unsafe extern "C" fn wcsncpy_s(
    dst: *mut u16,
    num: usize,
    src: *const u16,
    count: usize,
) -> i32 {
    if dst.is_null() || num == 0 {
        return EINVAL;
    }
    *dst = 0;
    if src.is_null() {
        return EINVAL;
    }
    let size = core::cmp::min(wlen(src), count);
    if size >= num {
        return ERANGE;
    }
    core::ptr::copy_nonoverlapping(src, dst, size);
    *dst.add(size) = 0;
    0
}

/// `wcscat_s(dst, elem, src)` — bounded wide concat.
#[no_mangle]
pub unsafe extern "C" fn wcscat_s(dst: *mut u16, elem: usize, src: *const u16) -> i32 {
    if dst.is_null() || elem == 0 {
        return EINVAL;
    }
    if src.is_null() {
        *dst = 0;
        return EINVAL;
    }
    let end = dst.add(elem);
    // seek to end of dst (or elem if no NUL found)
    let mut ptr = dst;
    while ptr < end && *ptr != 0 {
        ptr = ptr.add(1);
    }
    let mut s = src;
    while ptr < end {
        let c = *s;
        s = s.add(1);
        *ptr = c;
        ptr = ptr.add(1);
        if c == 0 {
            return 0;
        }
    }
    // not enough space
    *dst = 0;
    ERANGE
}

/// `wcsncat_s(dst, elem, src, count)` — bounded wide concat of `<=count`,
/// with `_TRUNCATE`/`STRUNCATE`. `_LIBCNT_` macro reductions applied
/// (CHECK_PMT→cond, INVALID_PMT→no-op, no errno).
#[no_mangle]
pub unsafe extern "C" fn wcsncat_s(
    dst: *mut u16,
    elem: usize,
    src: *const u16,
    count: usize,
) -> i32 {
    if dst.is_null() || elem == 0 {
        return EINVAL;
    }
    if !(!src.is_null() || count == 0) {
        return EINVAL;
    }
    if count == 0 {
        return 0;
    }
    // find first NUL within elem
    let mut dststart = 0usize;
    while dststart < elem {
        if *dst.add(dststart) == 0 {
            break;
        }
        dststart += 1;
    }
    if dststart == elem {
        // dst[elem] is not NUL terminated
        return EINVAL;
    }

    let mut ret = 0i32;
    let srclen;
    if count == TRUNCATE {
        let mut sl = wlen(src);
        if sl >= elem - dststart {
            sl = elem - dststart - 1;
            ret = STRUNCATE;
        }
        srclen = sl;
    } else {
        srclen = core::cmp::min(wlen(src), count);
    }
    if srclen < elem - dststart {
        core::ptr::copy_nonoverlapping(src, dst.add(dststart), srclen);
        *dst.add(dststart + srclen) = 0;
        return ret;
    }
    // dst[elem] is too small
    *dst = 0;
    ERANGE
}
