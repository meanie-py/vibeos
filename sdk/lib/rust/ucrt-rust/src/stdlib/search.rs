//! `bsearch.cpp`/`bsearch_s.cpp`, `lfind.cpp`/`lfind_s.cpp`,
//! `lsearch.cpp`/`lsearch_s.cpp`. Each pair is one body over a comparator
//! closure; the `_s` form passes `context` first. Pointer arithmetic is done
//! on addresses with wrapping ops, as the C's `char*` math compiles.

use super::invalid_einval;
use core::ffi::{c_int, c_uint, c_void};
use core::ptr::null_mut;

/// `int (__cdecl*)(void const*, void const*)`.
pub type Compare = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;
/// `int (__cdecl*)(void*, void const*, void const*)`.
pub type CompareS = Option<unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> c_int>;

// ── bsearch.cpp ─────────────────────────────────────────────────────────────

/// The body of `bsearch`/`bsearch_s`, after validation.
#[inline(always)]
unsafe fn bsearch_body(
    key: *const c_void,
    base: *const c_void,
    mut num: usize,
    width: usize,
    mut compare: impl FnMut(*const c_void, *const c_void) -> c_int,
) -> *mut c_void {
    let mut lo = base as usize;
    let mut hi = lo.wrapping_add(num.wrapping_sub(1).wrapping_mul(width));

    // A NULL key is allowed: we never dereference it ourselves.
    while lo <= hi {
        let half = num / 2;
        if half != 0 {
            let mid = lo.wrapping_add((if num & 1 != 0 { half } else { half - 1 }).wrapping_mul(width));

            let result = compare(key, mid as *const c_void);
            if result == 0 {
                return mid as *mut c_void;
            } else if result < 0 {
                hi = mid.wrapping_sub(width);
                num = if num & 1 != 0 { half } else { half - 1 };
            } else {
                lo = mid.wrapping_add(width);
                num = half;
            }
        } else if num != 0 {
            return if compare(key, lo as *const c_void) != 0 { null_mut() } else { lo as *mut c_void };
        } else {
            break;
        }
    }

    null_mut()
}

/// `void* bsearch(key, base, num, width, compare)`.
#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const c_void,
    base: *const c_void,
    num: usize,
    width: usize,
    compare: Compare,
) -> *mut c_void {
    if !(!base.is_null() || num == 0) {
        invalid_einval();
        return null_mut();
    }
    if !(width > 0) {
        invalid_einval();
        return null_mut();
    }
    let Some(compare) = compare else {
        invalid_einval();
        return null_mut();
    };
    bsearch_body(key, base, num, width, |a, b| compare(a, b))
}

/// `void* bsearch_s(key, base, num, width, compare, context)`.
#[no_mangle]
pub unsafe extern "C" fn bsearch_s(
    key: *const c_void,
    base: *const c_void,
    num: usize,
    width: usize,
    compare: CompareS,
    context: *mut c_void,
) -> *mut c_void {
    if !(!base.is_null() || num == 0) {
        invalid_einval();
        return null_mut();
    }
    if !(width > 0) {
        invalid_einval();
        return null_mut();
    }
    let Some(compare) = compare else {
        invalid_einval();
        return null_mut();
    };
    bsearch_body(key, base, num, width, |a, b| compare(context, a, b))
}

// ── lfind.cpp ───────────────────────────────────────────────────────────────

/// The linear scan shared by `_lfind*` and `_lsearch*`: the first `p` in
/// `[first, last)` stepping `width` with `compare(key, p) == 0`.
#[inline(always)]
unsafe fn scan(
    key: *const c_void,
    first: usize,
    last: usize,
    width: usize,
    compare: &mut impl FnMut(*const c_void, *const c_void) -> c_int,
) -> Option<usize> {
    let mut p = first;
    while p != last {
        if compare(key, p as *const c_void) == 0 {
            return Some(p);
        }
        p = p.wrapping_add(width);
    }
    None
}

/// `_lfind`/`_lfind_s` validation, in the C's order: key, num,
/// `base || *num == 0`, width, compare. True when it passed.
#[inline(always)]
unsafe fn lfind_valid(key: *const c_void, base: *const c_void, num: *mut c_uint, width: usize, has_compare: bool) -> bool {
    if key.is_null() || num.is_null() || !(!base.is_null() || *num == 0) || !(width > 0) || !has_compare {
        invalid_einval();
        return false;
    }
    true
}

/// `void* _lfind(key, base, unsigned* num, unsigned width, compare)`. The
/// end is `*num * width` in `unsigned int` (wraps at 32 bits).
#[no_mangle]
pub unsafe extern "C" fn _lfind(
    key: *const c_void,
    base: *const c_void,
    num: *mut c_uint,
    width: c_uint,
    compare: Compare,
) -> *mut c_void {
    if !lfind_valid(key, base, num, width as usize, compare.is_some()) {
        return null_mut();
    }
    let Some(compare) = compare else { return null_mut() };
    let first = base as usize;
    let last = first.wrapping_add((*num).wrapping_mul(width) as usize);
    scan(key, first, last, width as usize, &mut |a, b| compare(a, b)).map_or(null_mut(), |p| p as *mut c_void)
}

/// `void* _lfind_s(key, base, unsigned* num, size_t width, compare, context)`.
/// The end is `*num * width` in `size_t`.
#[no_mangle]
pub unsafe extern "C" fn _lfind_s(
    key: *const c_void,
    base: *const c_void,
    num: *mut c_uint,
    width: usize,
    compare: CompareS,
    context: *mut c_void,
) -> *mut c_void {
    if !lfind_valid(key, base, num, width, compare.is_some()) {
        return null_mut();
    }
    let Some(compare) = compare else { return null_mut() };
    let first = base as usize;
    let last = first.wrapping_add((*num as usize).wrapping_mul(width));
    scan(key, first, last, width, &mut |a, b| compare(context, a, b)).map_or(null_mut(), |p| p as *mut c_void)
}

// ── lsearch.cpp ─────────────────────────────────────────────────────────────

/// `_lsearch`/`_lsearch_s` validation, in the C's order: key, num, base
/// (NOT relaxed for `*num == 0`, unlike `_lfind`), width, compare.
#[inline(always)]
unsafe fn lsearch_valid(key: *const c_void, base: *mut c_void, num: *mut c_uint, width: usize, has_compare: bool) -> bool {
    if key.is_null() || num.is_null() || base.is_null() || !(width > 0) || !has_compare {
        invalid_einval();
        return false;
    }
    true
}

/// Not found: append `key` at `last` (`memcpy`, `width` bytes) and bump `*num`.
#[inline(always)]
unsafe fn lsearch_tail(
    key: *const c_void,
    num: *mut c_uint,
    first: usize,
    last: usize,
    width: usize,
    compare: &mut impl FnMut(*const c_void, *const c_void) -> c_int,
) -> *mut c_void {
    if let Some(found) = scan(key, first, last, width, compare) {
        return found as *mut c_void;
    }
    core::ptr::copy_nonoverlapping(key.cast::<u8>(), last as *mut u8, width);
    *num = (*num).wrapping_add(1);
    last as *mut c_void
}

/// `void* _lsearch(key, base, unsigned* num, unsigned width, compare)`.
#[no_mangle]
pub unsafe extern "C" fn _lsearch(
    key: *const c_void,
    base: *mut c_void,
    num: *mut c_uint,
    width: c_uint,
    compare: Compare,
) -> *mut c_void {
    if !lsearch_valid(key, base, num, width as usize, compare.is_some()) {
        return null_mut();
    }
    let Some(compare) = compare else { return null_mut() };
    let first = base as usize;
    let last = first.wrapping_add((*num).wrapping_mul(width) as usize);
    lsearch_tail(key, num, first, last, width as usize, &mut |a, b| compare(a, b))
}

/// `void* _lsearch_s(key, base, unsigned* num, size_t width, compare, context)`.
#[no_mangle]
pub unsafe extern "C" fn _lsearch_s(
    key: *const c_void,
    base: *mut c_void,
    num: *mut c_uint,
    width: usize,
    compare: CompareS,
    context: *mut c_void,
) -> *mut c_void {
    if !lsearch_valid(key, base, num, width, compare.is_some()) {
        return null_mut();
    }
    let Some(compare) = compare else { return null_mut() };
    let first = base as usize;
    let last = first.wrapping_add((*num as usize).wrapping_mul(width));
    lsearch_tail(key, num, first, last, width, &mut |a, b| compare(context, a, b))
}
