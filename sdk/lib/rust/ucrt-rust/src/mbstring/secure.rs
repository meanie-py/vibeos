//! The `_s`/`_s_l` "secure" forms: the 4 files with no shared template
//! (`_mbccpy_s`/`_l`, `_mbscat_s`/`_l`, `_mbscpy_s`/`_l`, `_mbsset_s`/`_l`)
//! AND the 3 shared `.inl` templates + their 12 thin instantiations
//! (`_mbsncat_s`/`_l`, `_mbsnbcat_s`/`_l`, `_mbsncpy_s`/`_l`, `_mbsnbcpy_s`/`_l`,
//! `_mbsnset_s`/`_l`, `_mbsnbset_s`/`_l`), each ported ONCE as a generic
//! function parameterized over `bytes: bool` (`_COUNT_IN_BYTES` 1 vs 0) --
//! see `ncat_s_common`, `ncpy_s_common`, `nset_s_common` below, each with a
//! branch-correspondence table in its own doc comment for review against the
//! `.inl` it replaces.
//!
//! ## Details that are silent if wrong (for what IS here)
//!
//! * **This build has no `_DEBUG`** (see `mod.rs`), so `_FILL_STRING`/
//!   `_FILL_BYTE` are no-ops and `_RESET_STRING` is exactly "write one NUL
//!   at the front" -- every `_RESET_STRING`/`RESET` call below is written as
//!   `*dst = 0` and nothing else.
//! * **`_mbccpy_s_l` is defined for its own file** (not delegated to from
//!   `modify.rs::_mbccpy_l`; it is the other way around: `_mbccpy`/`_mbccpy_l`
//!   in `modify.rs` call INTO this file's `_mbccpy_s_l`).
//! * **`_mbccpy_s_l` never validates that `*_PCopied` starts at 0 for the
//!   caller** -- it unconditionally writes 0 there FIRST (`_ASSIGN_IF_NOT_NULL`)
//!   before any validation, so a caller passing a garbage `_PCopied` still
//!   gets a defined 0 even on an EINVAL return.
//! * **`_mbscat_s_l`/`_mbscpy_s_l` use `_ISMBBLEADPREFIX`**, a BACKWARD parity
//!   scan from a byte position to the string start (`ismbb_lead_prefix`
//!   below) -- "is the byte at this exact position a genuine (odd-parity)
//!   lead byte", not a forward one-shot classification. Both call it at
//!   TWO different points with different pointer arithmetic for "the last
//!   byte of dst's original content" vs "the last byte just written" --
//!   transcribed as two separate call sites, not refactored into one.
//! * **`_mbscat_s_l` keeps a persistent `fFoundInvalidMBC` flag that can be
//!   set by the FIRST prefix check (on dst's own tail) and independently by
//!   a SECOND check after the copy**, and only converts to `EILSEQ` at the
//!   very end if still set by then -- an early return via
//!   `_RETURN_BUFFER_TOO_SMALL`/an immediate `_RETURN_MBCS_ERROR` bypasses
//!   that flag check entirely. `_mbscpy_s_l` has no such flag: it returns
//!   `EILSEQ` immediately at its one prefix-check site instead.

use super::{Locale, EILSEQ, EINVAL, STRUNCATE, TRUNCATE};
use crate::ismbbyte::_ismbblead_l;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int};

const ERANGE: c_int = 34;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn strcat_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn strcpy_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn _strset_s(dst: *mut u8, size: usize, value: c_int) -> c_int;
    fn strncat_s(dst: *mut u8, size: usize, src: *const u8, count: usize) -> c_int;
    /// Matches the EXISTING crate-wide declaration in `nls_wrappers.rs`
    /// exactly (`*mut c_char`/`*const c_char`, not `u8`) -- redeclaring with
    /// `u8` here would be a `clashing_extern_declarations` error the moment
    /// both modules are compiled together (caught by the all-features check).
    fn strncpy_s(dst: *mut c_char, size: usize, src: *const c_char, count: usize) -> c_int;
    fn _strnset_s(dst: *mut u8, size: usize, value: c_int, count: usize) -> c_int;
}

/// Sets `errno` and calls the invalid-parameter handler (the release-build
/// `_INVALID_PARAMETER` -> `_invalid_parameter_noinfo` path; `_ASSERT_EXPR`
/// is a debug-only no-op here), then returns `code` -- the shape of every
/// `_VALIDATE_RETURN(..., code, code)` in this file.
#[inline]
unsafe fn validate_fail(code: c_int) -> c_int {
    *_errno() = code;
    _invalid_parameter_noinfo();
    code
}

/// `_RETURN_MBCS_ERROR` / `_SET_MBCS_ERROR`: sets `errno = EILSEQ` and
/// returns it, WITHOUT calling the invalid-parameter handler.
#[inline]
unsafe fn mbcs_error() -> c_int {
    *_errno() = EILSEQ;
    EILSEQ
}

/// `_RETURN_BUFFER_TOO_SMALL`: `_VALIDATE_RETURN(0, ERANGE, ERANGE)` --
/// always taken, so always calls the handler.
#[inline]
unsafe fn buffer_too_small() -> c_int {
    validate_fail(ERANGE)
}

/// `_ISMBBLEADPREFIX(_Result, _StringStart, _BytePtr)`
/// (`corecrt_internal_securecrt.h:100-109`): scans backward from `byte_ptr`
/// while each byte classifies as `_ismbblead_l`, and answers whether the
/// COUNT of such bytes (byte_ptr back to where the scan stopped) is odd --
/// i.e. whether the byte AT `byte_ptr` is itself a genuine (correctly
/// paired) lead byte rather than a trail byte of an earlier pair. Uses
/// `wrapping_sub` throughout: the scan can walk one element before
/// `string_start`, which the loop condition then rejects WITHOUT
/// dereferencing -- exactly what the C's `Tmp >= Start &&` does, and Rust's
/// checked arithmetic would fault on where the C's plain pointer decrement
/// does not.
#[inline]
unsafe fn ismbb_lead_prefix(string_start: *const u8, byte_ptr: *const u8, l: Locale) -> bool {
    let mut tmp = byte_ptr;
    loop {
        if (tmp as usize) < (string_start as usize) {
            break;
        }
        if _ismbblead_l(*tmp as u32, l) == 0 {
            break;
        }
        tmp = tmp.wrapping_sub(1);
    }
    ((byte_ptr as usize).wrapping_sub(tmp as usize)) & 1 != 0
}

// ── mbccpy_s_l.cpp / mbccpy_s.cpp ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_s_l(
    dst: *mut u8,
    size_in_bytes: usize,
    p_copied: *mut c_int,
    src: *const u8,
    locale: Locale,
) -> c_int {
    if !p_copied.is_null() {
        *p_copied = 0;
    }
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    if src.is_null() {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    let l = lu.locale_t();
    if _ismbblead_l(*src as u32, l) != 0 {
        if *src.add(1) == 0 {
            *dst = 0;
            if !p_copied.is_null() {
                *p_copied = 1;
            }
            return mbcs_error();
        }
        if size_in_bytes < 2 {
            *dst = 0;
            return buffer_too_small();
        }
        *dst = *src;
        *dst.add(1) = *src.add(1);
        if !p_copied.is_null() {
            *p_copied = 2;
        }
    } else {
        *dst = *src;
        if !p_copied.is_null() {
            *p_copied = 1;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_s(dst: *mut u8, size_in_bytes: usize, p_copied: *mut c_int, src: *const u8) -> c_int {
    _mbccpy_s_l(dst, size_in_bytes, p_copied, src, core::ptr::null_mut())
}

// ── mbscat_s_l.cpp / mbscat_s.cpp ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbscat_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, locale: Locale) -> c_int {
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    if src.is_null() {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strcat_s(dst, size_in_bytes, src);
    }
    let l = lu.locale_t();
    let mut found_invalid = false;
    let mut p = dst;
    let mut available = size_in_bytes;
    while available > 0 && *p != 0 {
        p = p.add(1);
        available -= 1;
    }
    if available == 0 {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    if ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
        found_invalid = true;
        p = p.sub(1);
        *p = 0;
        available += 1;
    }
    let mut s = src;
    loop {
        let b = *s;
        s = s.add(1);
        *p = b;
        p = p.add(1);
        if b == 0 {
            break;
        }
        available -= 1;
        if available == 0 {
            break;
        }
    }
    if available == 0 {
        if *s == 0 && ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
            *p.sub(1) = 0;
            return mbcs_error();
        }
        *dst = 0;
        return buffer_too_small();
    }
    if ismbb_lead_prefix(dst, p.wrapping_sub(2), l) {
        *p.sub(2) = 0;
        found_invalid = true;
    }
    if found_invalid {
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbscat_s(dst: *mut u8, size_in_bytes: usize, src: *const u8) -> c_int {
    _mbscat_s_l(dst, size_in_bytes, src, core::ptr::null_mut())
}

// ── mbscpy_s_l.cpp / mbscpy_s.cpp ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbscpy_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, locale: Locale) -> c_int {
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    if src.is_null() {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strcpy_s(dst, size_in_bytes, src);
    }
    let l = lu.locale_t();
    let mut p = dst;
    let mut s = src;
    let mut available = size_in_bytes;
    loop {
        let b = *s;
        s = s.add(1);
        *p = b;
        p = p.add(1);
        if b == 0 {
            break;
        }
        available -= 1;
        if available == 0 {
            break;
        }
    }
    if available == 0 {
        if *s == 0 && ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
            *p.sub(1) = 0;
            return mbcs_error();
        }
        *dst = 0;
        return buffer_too_small();
    }
    if (p.wrapping_sub(2) as usize) >= (dst as usize) && ismbb_lead_prefix(dst, p.wrapping_sub(2), l) {
        *p.sub(2) = 0;
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbscpy_s(dst: *mut u8, size_in_bytes: usize, src: *const u8) -> c_int {
    _mbscpy_s_l(dst, size_in_bytes, src, core::ptr::null_mut())
}

// ── mbsset_s_l.cpp / mbsset_s.cpp ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsset_s_l(dst: *mut u8, size_in_bytes: usize, value: u32, locale: Locale) -> c_int {
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strset_s(dst, size_in_bytes, value as c_int);
    }
    let l = lu.locale_t();
    let highval = (value >> 8) as u8;
    let lowval = (value & 0xff) as u8;
    // "ensure _Value is a valid mbchar": a two-byte value needs a genuine
    // lead byte on top and a nonzero low byte; a one-byte value must NOT
    // itself classify as a lead byte.
    if (highval != 0 && (lowval == 0 || _ismbblead_l(highval as u32, l) == 0))
        || (highval == 0 && _ismbblead_l(lowval as u32, l) != 0)
    {
        *dst = 0;
        return mbcs_error();
    }
    let mut mbcs_err = false;
    let mut p = dst;
    let mut available = size_in_bytes;
    if highval != 0 {
        while *p != 0 {
            available -= 1;
            if available == 0 {
                break;
            }
            if *p.add(1) == 0 {
                *p = 0;
                available += 1;
                mbcs_err = true;
                break;
            }
            *p = highval;
            p = p.add(1);
            available -= 1;
            if available == 0 {
                break;
            }
            *p = lowval;
            p = p.add(1);
        }
    } else {
        while *p != 0 {
            available -= 1;
            if available == 0 {
                break;
            }
            *p = lowval;
            p = p.add(1);
        }
    }
    if available == 0 {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    if mbcs_err {
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsset_s(dst: *mut u8, size_in_bytes: usize, value: u32) -> c_int {
    _mbsset_s_l(dst, size_in_bytes, value, core::ptr::null_mut())
}

// ============================================================================
// mbsncat_s.inl -- shared by _mbsncat_s_l (bytes=false, _COUNT=_CountInChars)
// and _mbsnbcat_s_l (bytes=true, _COUNT=_CountInBytes).
// ============================================================================

/// `mbsncat_s.inl`'s generic body, ported ONCE and parameterized by `bytes`
/// (`_COUNT_IN_BYTES`). Branch-correspondence table (read top to bottom,
/// matching the `.inl`'s own order) -- review this against
/// `sdk/lib/ucrt/mbstring/mbsncat_s.inl` directly:
///
/// | # | `.inl` condition                                   | `bytes` | Rust below                                    |
/// |---|-----------------------------------------------------|---------|------------------------------------------------|
/// | 1 | `count==0 && dst==NULL && size==0`                   | both    | early `return 0`                                |
/// | 2 | `_VALIDATE_STRING(dst,size)`                         | both    | `dst.is_null() \|\| size==0` -> `validate_fail(EINVAL)` |
/// | 3 | `count!=0` -> `_VALIDATE_POINTER_RESET_STRING(src,...)` | both | `count!=0 && src.is_null()` -> `*dst=0; validate_fail(EINVAL)` |
/// | 4 | `ismbcodepage==0` -> `strncat_s(...)`                | both    | same, calling `strncat_s`                       |
/// | 5 | scan to dst's own NUL, `available==0` -> reset+EINVAL| both   | scan loop, then same check                      |
/// | 6 | `available<size` -> back up over dst's own dangling lead | both | `ismbb_lead_prefix(dst, p-1)` check              |
/// | 7 | `count==_TRUNCATE` -> plain fill-to-NUL-or-buffer-full loop | both | `count==TRUNCATE` arm, shared for both `bytes`  |
/// | 8 | else, bytes: `while(cnt>0 && write!=0 && --avail>0){cnt--;}` | true | byte-budget loop (all 3 conditions inline, decrement at bottom, matching AND short-circuit) |
/// | 8'| else, chars: `while(cnt>0){ lead-aware copy; cnt--; }` (break-only, no trailing decrement on break) | false | char-budget loop, `cnt-=1` only reached past the `if`/`else`, matching |
/// | 9 | `cnt==0` after either loop -> `*p++=0`                | both    | same                                            |
/// |10 | `available==0`, `bytes` only: `*src==0 \|\| cnt==1` -> dangling-lead check -> immediate `_RETURN_MBCS_ERROR` | true only | same, returns before the TRUNCATE/too-small checks |
/// |11 | `available==0`, `count==_TRUNCATE`: `_SET_MBCS_ERROR` (no return) if `found_invalid`, then lead-prefix check at `dst[size-2]` (ABSOLUTE, not `p`-relative) -> `_FILL_BYTE` (no-op) + `_RETURN_TRUNCATE`, else `dst[size-1]=0` + `_RETURN_TRUNCATE` | both | same; note `errno` can be `EILSEQ` while the RETURN VALUE is `STRUNCATE` -- both independently correct, not a contradiction |
/// |12 | `available==0`, else: reset + `_RETURN_BUFFER_TOO_SMALL`| both  | same                                             |
/// |13 | `bytes` only, `available<size`: lead-prefix check at `p[-2]` -> mark invalid | true only | same                                    |
/// |14 | `found_invalid` -> `_RETURN_MBCS_ERROR`, else 0        | both    | same                                            |
#[inline]
unsafe fn ncat_s_common(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale, bytes: bool) -> c_int {
    let mut found_invalid = false;
    if count == 0 && dst.is_null() && size_in_bytes == 0 {
        return 0;
    }
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    if count != 0 && src.is_null() {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncat_s(dst, size_in_bytes, src, count);
    }
    let l = lu.locale_t();

    let mut p = dst;
    let mut available = size_in_bytes;
    while available > 0 && *p != 0 {
        p = p.add(1);
        available -= 1;
    }
    if available == 0 {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    if available < size_in_bytes && ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
        p = p.sub(1);
        *p = 0;
        available += 1;
        found_invalid = true;
    }

    let mut s = src;
    let mut cnt = count;
    if count == TRUNCATE {
        loop {
            let b = *s;
            s = s.add(1);
            *p = b;
            p = p.add(1);
            if b == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
        }
    } else if bytes {
        while cnt > 0 {
            let b = *s;
            s = s.add(1);
            *p = b;
            p = p.add(1);
            if b == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            cnt -= 1;
        }
    } else {
        while cnt > 0 {
            if _ismbblead_l(*s as u32, l) != 0 {
                if *s.add(1) == 0 {
                    *p = 0;
                    found_invalid = true;
                    break;
                }
                if available <= 2 {
                    available = 0;
                    break;
                }
                *p = *s;
                p = p.add(1);
                s = s.add(1);
                *p = *s;
                p = p.add(1);
                s = s.add(1);
                available -= 2;
            } else {
                let b = *s;
                s = s.add(1);
                *p = b;
                p = p.add(1);
                if b == 0 || {
                    available -= 1;
                    available == 0
                } {
                    break;
                }
            }
            cnt -= 1;
        }
    }
    if count != TRUNCATE && cnt == 0 {
        *p = 0;
        p = p.add(1);
    }

    if available == 0 {
        if bytes && (*s == 0 || cnt == 1) && ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
            *p.sub(1) = 0;
            return mbcs_error();
        }
        if count == TRUNCATE {
            if found_invalid {
                *_errno() = EILSEQ; // `_SET_MBCS_ERROR`: errno only, no return yet.
            }
            if size_in_bytes > 1 && ismbb_lead_prefix(dst, dst.add(size_in_bytes - 2), l) {
                *dst.add(size_in_bytes - 2) = 0;
                // `_FILL_BYTE(dst[size-1])`: no-op in this build, see mod.rs.
                return STRUNCATE;
            }
            *dst.add(size_in_bytes - 1) = 0;
            return STRUNCATE;
        }
        *dst = 0;
        return buffer_too_small();
    }
    if bytes && available < size_in_bytes && ismbb_lead_prefix(dst, p.wrapping_sub(2), l) {
        *p.sub(2) = 0;
        found_invalid = true;
    }
    if found_invalid {
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale) -> c_int {
    ncat_s_common(dst, size_in_bytes, src, count, locale, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat_s(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize) -> c_int {
    _mbsncat_s_l(dst, size_in_bytes, src, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale) -> c_int {
    ncat_s_common(dst, size_in_bytes, src, count, locale, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_s(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize) -> c_int {
    _mbsnbcat_s_l(dst, size_in_bytes, src, count, core::ptr::null_mut())
}

// ============================================================================
// mbsncpy_s.inl -- shared by _mbsncpy_s_l (bytes=false) and _mbsnbcpy_s_l
// (bytes=true). Structurally close to mbsncat_s.inl but NOT identical: the
// char-mode loop here is a `do { } while(--count>0)` (body runs at least
// once, decrement+test at the BOTTOM), where mbsncat_s.inl's char-mode loop
// is `while(count>0){ ...; count--; }` (test at TOP, decrement inside the
// body, skipped entirely on an internal `break`) -- same end effect on a
// natural exit, but do-while executes even when NOTHING has been checked yet
// (safe here only because `count==0` was already handled above), and it is
// the reason this port's char-mode arm is a bare Rust `loop` with the
// decrement/break at the BOTTOM rather than the top-tested `while cnt > 0`
// mbsncat_s_l above uses.
// ============================================================================

/// `mbsncpy_s.inl`'s generic body. Branch-correspondence table, in the
/// `.inl`'s own order -- review against `sdk/lib/ucrt/mbstring/mbsncpy_s.inl`:
///
/// | # | `.inl` condition                                     | `bytes` | Rust below |
/// |---|-------------------------------------------------------|---------|------------|
/// | 1 | `count==0 && dst==NULL && size==0`                     | both    | early `return 0` |
/// | 2 | `_VALIDATE_STRING(dst,size)`                           | both    | `dst.is_null() \|\| size==0` -> EINVAL |
/// | 3 | `count==0` -> `_RESET_STRING(dst,size); RETURN_NO_ERROR`| both   | `count==0` -> `*dst=0; return 0` (src may be NULL here, never read) |
/// | 4 | `_VALIDATE_POINTER_RESET_STRING(src,...)`              | both    | `src.is_null()` -> `*dst=0; EINVAL` |
/// | 5 | `ismbcodepage==0` -> `strncpy_s(...)`                  | both    | same |
/// | 6 | `count==_TRUNCATE`: `while((*p++=*src++)!=0 && --avail>0){}` | both | `count==TRUNCATE` arm, shared |
/// | 7 | else, bytes: `while((*p++=*src++)!=0 && --avail>0 && --cnt>0){}` (empty body, ALL 3 clauses inline) | true | byte-budget loop, matches short-circuit exactly |
/// | 7'| else, chars: `do { lead-aware copy } while(--cnt>0);` (body ALWAYS runs once; decrement+test at bottom; internal `break` skips the decrement) | false | Rust `loop{ body; cnt-=1; if cnt==0{break} }`, with internal breaks bypassing the decrement via their own `break` |
/// | 8 | `cnt==0` after either loop -> `*p++=0`                  | both    | same |
/// | 9 | `available==0`, bytes only: `*src==0 \|\| cnt==1` -> lead-prefix at `p[-1]` -> immediate `_RETURN_MBCS_ERROR` | true only | same |
/// |10 | `available==0`, `count==_TRUNCATE`: `_SET_MBCS_ERROR` if invalid (no return), lead-prefix at `dst[size-2]` (absolute) -> truncate, else `dst[size-1]=0` -> truncate | both | same |
/// |11 | `available==0`, else: reset + `_RETURN_BUFFER_TOO_SMALL`| both   | same |
/// |12 | bytes only, guard `(p-dst)>=2` (NOT `available<size` -- `mbsncpy_s.inl` uses an ABSOLUTE distance-from-`dst` guard here, unlike `mbsncat_s.inl`'s `available<size`, because `p` starts AT `dst` for a copy, not partway into existing content) -> lead-prefix at `p[-2]` -> mark invalid | true only | `(p as usize).wrapping_sub(dst as usize) >= 2` guard, transcribed literally distinct from the cat version |
/// |13 | `found_invalid` -> `_RETURN_MBCS_ERROR`, else 0          | both   | same |
#[inline]
unsafe fn ncpy_s_common(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale, bytes: bool) -> c_int {
    let mut found_invalid = false;
    if count == 0 && dst.is_null() && size_in_bytes == 0 {
        return 0;
    }
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    if count == 0 {
        *dst = 0;
        return 0;
    }
    if src.is_null() {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncpy_s(dst.cast(), size_in_bytes, src.cast(), count);
    }
    let l = lu.locale_t();

    let mut p = dst;
    let mut s = src;
    let mut available = size_in_bytes;
    let mut cnt = count;
    if count == TRUNCATE {
        loop {
            let b = *s;
            s = s.add(1);
            *p = b;
            p = p.add(1);
            if b == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
        }
    } else if bytes {
        loop {
            let b = *s;
            s = s.add(1);
            *p = b;
            p = p.add(1);
            if b == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            cnt -= 1;
            if cnt == 0 {
                break;
            }
        }
    } else {
        loop {
            if _ismbblead_l(*s as u32, l) != 0 {
                if *s.add(1) == 0 {
                    *p = 0;
                    p = p.add(1);
                    found_invalid = true;
                    break;
                }
                if available <= 2 {
                    available = 0;
                    break;
                }
                *p = *s;
                p = p.add(1);
                s = s.add(1);
                *p = *s;
                p = p.add(1);
                s = s.add(1);
                available -= 2;
            } else {
                let b = *s;
                s = s.add(1);
                *p = b;
                p = p.add(1);
                if b == 0 {
                    break;
                }
                available -= 1;
                if available == 0 {
                    break;
                }
            }
            cnt -= 1;
            if cnt == 0 {
                break;
            }
        }
    }
    if count != TRUNCATE && cnt == 0 {
        // `*p++ = 0;` -- the ADVANCE is load-bearing: the byte-counted lead-prefix
        // check below reads p[-2]. Without it _mbsnbcpy_s(d, 2, "\x82\xa0", 1) under
        // cp932 missed the dangling lead byte and returned 0 where the C says EILSEQ.
        *p = 0;
        p = p.add(1);
    }

    if available == 0 {
        if bytes && (*s == 0 || cnt == 1) && ismbb_lead_prefix(dst, p.wrapping_sub(1), l) {
            *p.sub(1) = 0;
            return mbcs_error();
        }
        if count == TRUNCATE {
            if found_invalid {
                *_errno() = EILSEQ;
            }
            if size_in_bytes > 1 && ismbb_lead_prefix(dst, dst.add(size_in_bytes - 2), l) {
                *dst.add(size_in_bytes - 2) = 0;
                return STRUNCATE;
            }
            *dst.add(size_in_bytes - 1) = 0;
            return STRUNCATE;
        }
        *dst = 0;
        return buffer_too_small();
    }
    if bytes && (p as usize).wrapping_sub(dst as usize) >= 2 && ismbb_lead_prefix(dst, p.wrapping_sub(2), l) {
        *p.sub(2) = 0;
        found_invalid = true;
    }
    if found_invalid {
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale) -> c_int {
    ncpy_s_common(dst, size_in_bytes, src, count, locale, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy_s(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize) -> c_int {
    _mbsncpy_s_l(dst, size_in_bytes, src, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_s_l(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize, locale: Locale) -> c_int {
    ncpy_s_common(dst, size_in_bytes, src, count, locale, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_s(dst: *mut u8, size_in_bytes: usize, src: *const u8, count: usize) -> c_int {
    _mbsnbcpy_s_l(dst, size_in_bytes, src, count, core::ptr::null_mut())
}

// ============================================================================
// mbsnset_s.inl -- shared by _mbsnset_s_l (bytes=false) and _mbsnbset_s_l
// (bytes=true). Structurally the SIMPLEST of the three templates: there is
// exactly ONE `#if _COUNT_IN_BYTES` branch point (converting a byte budget to
// a character budget ONCE, up front, only in the two-byte fill-value case --
// a single-byte fill value needs no conversion since 1 byte == 1 char either
// way). Everything after that point is IDENTICAL between the two
// instantiations. There is also NO `_TRUNCATE`/`STRUNCATE` handling anywhere
// in this template: `_mbsnset_s`/`_mbsnbset_s` never return `STRUNCATE`, and
// passing `_TRUNCATE` as `count` just behaves as an enormous ordinary count
// (fill until the string or buffer ends) with no dedicated code path.
// ============================================================================

/// `mbsnset_s.inl`'s generic body. Branch-correspondence table, in the
/// `.inl`'s own order -- review against `sdk/lib/ucrt/mbstring/mbsnset_s.inl`:
///
/// | # | `.inl` condition                                          | `bytes` | Rust below |
/// |---|-------------------------------------------------------------|---------|------------|
/// | 1 | `count==0 && dst==NULL && size==0`                           | both    | early `return 0` |
/// | 2 | `_VALIDATE_STRING(dst,size)`                                 | both    | `dst.is_null() \|\| size==0` -> EINVAL |
/// | 3 | `ismbcodepage==0` -> `_strnset_s(...)`                       | both    | same |
/// | 4 | split `value` into `highval`/`lowval`; reject an invalid mbchar (2-byte with a zero low byte or a non-lead high byte; OR a 1-byte value that ITSELF classifies as a lead byte) -> reset + `_RETURN_MBCS_ERROR` | both | `validate the mbchar` check, same formula as `_mbsset_s_l` |
/// | 5 | `highval!=0`, bytes only: odd `count` -> `count--; mbcs_error=1;` then `count/=2` (byte budget -> char budget, ONCE) | true only | `if (count&1)==1 { count-=1; mbcs_error=1; } count/=2;` |
/// | 6 | `highval!=0`: `while(*p!=0 && count>0 && --avail>0){ orphan-check; toggle mblead on ORIGINAL byte BEFORE overwrite; write high; if(--avail==0)break; toggle mblead again; write low; count--; }` | both (identical after step 5's conversion) | double-byte fill loop |
/// | 6'| `highval==0`: `while(*p!=0 && count>0 && --avail>0){ toggle mblead; write low; count--; }` | both | single-byte fill loop |
/// | 7 | post-loop: `count==0 && mblead && *p!=0 && --avail>0` -> `mbcs_error=1`; if `p[1]==0` truncate here (`*p=0; ++avail;`) else overwrite the orphaned trail with `' '` | both | orphan-trail fixup |
/// | 8 | post-loop: `count==0` -> skip forward over any untouched tail of the original string (`while(*p!=0 && --avail>0) ++p;`), which can itself drive `available` to 0 | both | tail-skip |
/// | 9 | `available==0` -> reset + `_RETURN_DEST_NOT_NULL_TERMINATED`      | both | same |
/// |10 | `mbcs_error` -> `_RETURN_MBCS_ERROR`, else `_RETURN_NO_ERROR`      | both | same |
#[inline]
unsafe fn nset_s_common(dst: *mut u8, size_in_bytes: usize, value: u32, mut count: usize, locale: Locale, bytes: bool) -> c_int {
    if count == 0 && dst.is_null() && size_in_bytes == 0 {
        return 0;
    }
    if dst.is_null() || size_in_bytes == 0 {
        return validate_fail(EINVAL);
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strnset_s(dst, size_in_bytes, value as c_int, count);
    }
    let l = lu.locale_t();

    let highval = (value >> 8) as u8;
    let lowval = (value & 0xff) as u8;
    if (highval != 0 && (lowval == 0 || _ismbblead_l(highval as u32, l) == 0)) || (highval == 0 && _ismbblead_l(lowval as u32, l) != 0) {
        *dst = 0;
        return mbcs_error();
    }

    let mut mbcs_err = false;
    let mut mblead = false;
    let mut p = dst;
    let mut available = size_in_bytes;

    if highval != 0 {
        if bytes {
            if count & 1 == 1 {
                count -= 1;
                mbcs_err = true;
            }
            count /= 2;
        }
        while *p != 0 {
            if count == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            if *p.add(1) == 0 {
                *p = 0;
                available += 1;
                mbcs_err = true;
                break;
            }
            mblead = if mblead { false } else { _ismbblead_l(*p as u32, l) != 0 };
            *p = highval;
            p = p.add(1);
            available -= 1;
            if available == 0 {
                break;
            }
            mblead = if mblead { false } else { _ismbblead_l(*p as u32, l) != 0 };
            *p = lowval;
            p = p.add(1);
            count -= 1;
        }
    } else {
        while *p != 0 {
            if count == 0 {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            mblead = if mblead { false } else { _ismbblead_l(*p as u32, l) != 0 };
            *p = lowval;
            p = p.add(1);
            count -= 1;
        }
    }

    if count == 0 && mblead && *p != 0 {
        available -= 1;
        if available > 0 {
            mbcs_err = true;
            if *p.add(1) == 0 {
                *p = 0;
                available += 1;
            } else {
                *p = b' ';
                p = p.add(1);
            }
        }
    }

    if count == 0 {
        while *p != 0 {
            available -= 1;
            if available == 0 {
                break;
            }
            p = p.add(1);
        }
    }

    if available == 0 {
        *dst = 0;
        return validate_fail(EINVAL);
    }
    if mbcs_err {
        return mbcs_error();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset_s_l(dst: *mut u8, size_in_bytes: usize, value: u32, count: usize, locale: Locale) -> c_int {
    nset_s_common(dst, size_in_bytes, value, count, locale, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset_s(dst: *mut u8, size_in_bytes: usize, value: u32, count: usize) -> c_int {
    _mbsnset_s_l(dst, size_in_bytes, value, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset_s_l(dst: *mut u8, size_in_bytes: usize, value: u32, count: usize, locale: Locale) -> c_int {
    nset_s_common(dst, size_in_bytes, value, count, locale, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset_s(dst: *mut u8, size_in_bytes: usize, value: u32, count: usize) -> c_int {
    _mbsnbset_s_l(dst, size_in_bytes, value, count, core::ptr::null_mut())
}
