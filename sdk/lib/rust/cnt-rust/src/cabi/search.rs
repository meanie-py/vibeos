//! C search leaves — `bsearch` and `_lfind` (kernel libcntpr).
//!
//! Faithful ports of ReactOS `search/bsearch.c` (DJ Delorie) and
//! `search/lfind.c`. The comparator runs over opaque C memory, so — like
//! `qsort` — the body is inherently pointer-based; there is no pure-slice
//! core worth factoring out. Result-faithful to the C byte-for-byte: same
//! probe sequence, and on ties the same element pointer is returned.
//!
//! `_lsearch` is NOT here — it is `#ifndef _LIBCNT_` (msvcrt-only, and it
//! mutates the array), so the kernel never links it; CRT keeps the C.

use super::CntCmpFn;
use core::ffi::c_void;

/// `void *bsearch(const void *key, const void *base, size_t nelem,
///                size_t size, int (__cdecl *cmp)(const void*, const void*))`
///
/// Binary search over a sorted array. `lim >>= 1` is the for-loop's
/// increment in the C — it runs every iteration, *after* the `lim--`
/// taken on the move-right branch.
#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const c_void,
    base0: *const c_void,
    nelem: usize,
    size: usize,
    cmp: CntCmpFn,
) -> *mut c_void {
    let mut base = base0 as *const u8;
    let mut lim = nelem;
    while lim != 0 {
        let p = base.add((lim >> 1) * size);
        let cmpval = cmp(key, p as *const c_void);
        if cmpval == 0 {
            return p as *mut c_void;
        }
        if cmpval > 0 {
            // key > p: move right
            base = p.add(size);
            lim -= 1;
        } // else move left
        lim >>= 1;
    }
    core::ptr::null_mut()
}

/// `void *_lfind(const void *key, const void *base, unsigned int *nelp,
///               unsigned int width, int (__cdecl *compar)(...))`
///
/// Linear search; returns the first matching element or NULL. `nelp`/
/// `width` are C `unsigned int` (32-bit on every target).
#[no_mangle]
pub unsafe extern "C" fn _lfind(
    key: *const c_void,
    base: *const c_void,
    nelp: *const u32,
    width: u32,
    compar: CntCmpFn,
) -> *mut c_void {
    let mut char_base = base as *const u8;
    let n = *nelp;
    let mut i: u32 = 0;
    while i < n {
        if compar(key, char_base as *const c_void) == 0 {
            return char_base as *mut c_void;
        }
        char_base = char_base.add(width as usize);
        i += 1;
    }
    core::ptr::null_mut()
}
