//! `qsort.cpp` and `qsort_s.cpp`: Microsoft's non-recursive median-of-three
//! quicksort with an explicit stack and an insertion (max-selection) sort
//! below `CUTOFF`. Ported statement for statement, because the ORDER of
//! comparator calls is observable. Element pointers are kept as addresses and
//! compared unsigned, as the C's `char*` comparisons compile; the one place
//! the C compares `ptrdiff_t`s (which half to recurse into) is signed here too.

use super::invalid_einval;
use super::search::{Compare, CompareS};
use core::ffi::{c_int, c_void};

/// Arrays of this many elements or fewer use `shortsort`.
const CUTOFF: usize = 8;

/// `STKSIZ`: `8 * sizeof(void*) - 2` -- 30 on 32-bit, 62 on 64-bit.
const STKSIZ: usize = 8 * core::mem::size_of::<*mut c_void>() - 2;

/// `swap`: exchanges two `width`-byte elements unless they are the same one.
/// Distinct elements of one array never overlap, so `swap_nonoverlapping` is
/// the C's byte-at-a-time loop without a hand-written copy loop.
#[inline(always)]
unsafe fn swap(a: usize, b: usize, width: usize) {
    if a != b {
        core::ptr::swap_nonoverlapping(a as *mut u8, b as *mut u8, width);
    }
}

/// `shortsort`/`shortsort_s`: repeatedly move the maximum of `[lo, hi]` to
/// `hi`. Compares `(p, max)`; on a tie the EARLIER element stays `max`.
#[inline(always)]
unsafe fn shortsort(lo: usize, mut hi: usize, width: usize, comp: &mut impl FnMut(usize, usize) -> c_int) {
    while hi > lo {
        let mut max = lo;
        let mut p = lo + width;
        while p <= hi {
            if comp(p, max) > 0 {
                max = p;
            }
            p += width;
        }

        swap(max, hi, width);

        hi -= width;
    }
}

/// The body of `qsort`/`qsort_s`, after validation.
unsafe fn qsort_body(base: *mut c_void, num: usize, width: usize, comp: &mut impl FnMut(usize, usize) -> c_int) {
    // A stack for saving the sub-arrays yet to be processed:
    let mut lostk = [0usize; STKSIZ];
    let mut histk = [0usize; STKSIZ];
    let mut stkptr: c_int = 0;

    if num < 2 {
        return;
    }

    // `hi` is the last element, not one past it.
    let mut lo = base as usize;
    let mut hi = (base as usize).wrapping_add(width.wrapping_mul(num - 1));

    // `recurse:` -- each `continue 'recurse` is the C's `goto recurse`.
    'recurse: loop {
        let size = (hi - lo) / width + 1;

        if size <= CUTOFF {
            shortsort(lo, hi, width, comp);
        } else {
            // Median of the first, middle and last elements.
            let mut mid = lo + (size / 2) * width;

            if comp(lo, mid) > 0 {
                swap(lo, mid, width);
            }
            if comp(lo, hi) > 0 {
                swap(lo, hi, width);
            }
            if comp(mid, hi) > 0 {
                swap(mid, hi, width);
            }

            let mut loguy = lo;
            let mut higuy = hi;

            loop {
                // The doubled loops never call comp(mid, mid).
                if mid > loguy {
                    loop {
                        loguy += width;
                        if !(loguy < mid && comp(loguy, mid) <= 0) {
                            break;
                        }
                    }
                }
                if mid <= loguy {
                    loop {
                        loguy += width;
                        if !(loguy <= hi && comp(loguy, mid) <= 0) {
                            break;
                        }
                    }
                }

                loop {
                    higuy -= width;
                    if !(higuy > mid && comp(higuy, mid) > 0) {
                        break;
                    }
                }

                if higuy < loguy {
                    break;
                }

                swap(loguy, higuy, width);

                // If the partition element was moved, follow it.
                if mid == higuy {
                    mid = loguy;
                }
            }

            // Find adjacent elements equal to the partition element.
            higuy += width;
            if mid < higuy {
                loop {
                    higuy -= width;
                    if !(higuy > mid && comp(higuy, mid) == 0) {
                        break;
                    }
                }
            }
            if mid >= higuy {
                loop {
                    higuy -= width;
                    if !(higuy > lo && comp(higuy, mid) == 0) {
                        break;
                    }
                }
            }

            // Recurse into the smaller side first. `ptrdiff_t` comparison:
            // `hi - loguy` is negative when `loguy == hi + width`.
            if (higuy.wrapping_sub(lo) as isize) >= (hi.wrapping_sub(loguy) as isize) {
                if lo < higuy {
                    lostk[stkptr as usize] = lo;
                    histk[stkptr as usize] = higuy;
                    stkptr += 1;
                }

                if loguy < hi {
                    lo = loguy;
                    continue 'recurse;
                }
            } else {
                if loguy < hi {
                    lostk[stkptr as usize] = loguy;
                    histk[stkptr as usize] = hi;
                    stkptr += 1;
                }

                if lo < higuy {
                    hi = higuy;
                    continue 'recurse;
                }
            }
        }

        // Pop a pending sub-array, if any.
        stkptr -= 1;
        if stkptr >= 0 {
            lo = lostk[stkptr as usize];
            hi = histk[stkptr as usize];
            continue 'recurse;
        } else {
            return;
        }
    }
}

/// `_VALIDATE_RETURN_VOID` x3, in the C's order: `base || num == 0`,
/// `width > 0`, `comp`. True when it passed.
#[inline(always)]
unsafe fn qsort_valid(base: *mut c_void, num: usize, width: usize, has_comp: bool) -> bool {
    if !(!base.is_null() || num == 0) || !(width > 0) || !has_comp {
        invalid_einval();
        return false;
    }
    true
}

/// `void qsort(base, num, width, comp)`.
#[no_mangle]
pub unsafe extern "C" fn qsort(base: *mut c_void, num: usize, width: usize, comp: Compare) {
    if !qsort_valid(base, num, width, comp.is_some()) {
        return;
    }
    let Some(comp) = comp else { return };
    qsort_body(base, num, width, &mut |a, b| comp(a as *const c_void, b as *const c_void));
}

/// `void qsort_s(base, num, width, comp, context)`.
#[no_mangle]
pub unsafe extern "C" fn qsort_s(base: *mut c_void, num: usize, width: usize, comp: CompareS, context: *mut c_void) {
    if !qsort_valid(base, num, width, comp.is_some()) {
        return;
    }
    let Some(comp) = comp else { return };
    qsort_body(base, num, width, &mut |a, b| comp(context, a as *const c_void, b as *const c_void));
}
