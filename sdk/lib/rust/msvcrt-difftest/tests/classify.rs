#![no_builtins] // the #[no_mangle] libc-named shims must be CALLED, not folded to LLVM builtins.
//! Classify each **shipping** msvcrt Rust shim (`msvcrt-crt-rust`, the crate
//! that links into msvcrt.dll) against the GENUINE compiled msvcrt reference
//! (`dll/win32/msvcrt/string.c`, compiled by build.rs as `ref_*`).
//!
//! Output is a verdict table, not a pass/fail: the point is to learn WHICH
//! functions Rust can safely take over in msvcrt.dll and which have a
//! divergent contract (cnt-rust's cores track *libcntpr*; msvcrt is *Wine*-
//! derived, and `msvcrt-crt-rust`'s wrappers are where the gap is adapted).
//!
//!   MATCH      — identical results across the whole fuzz domain → safe to flip
//!   SIGN-ONLY  — comparison fns: same sign, different magnitude (see note)
//!   DIVERGE    — a real contract difference (counterexample printed) → DO NOT FLIP
//!
//! ⚠ TWO BLIND SPOTS. This harness can only falsify what it VARIES, and it
//! varies neither of these — both must be settled by READING the C:
//!
//!  1. **Locale.** The reference runs in the default "C" locale. msvcrt bodies
//!     that consult `get_locinfo()->lc_handle[LC_CTYPE]` (`_stricmp`,
//!     `_strnicmp`, `_memicmp`) coincide with a locale-blind ASCII Rust fold
//!     ONLY there — so a MATCH from this harness would be a lie the moment an
//!     app calls `setlocale()`. They are deliberately NOT graded here and NOT
//!     flipped; see the DEFERRED note printed after the table.
//!
//!  2. **`_WIN64`.** The reference TU is compiled for the HOST (x86_64 linux),
//!     where `_WIN64` is undefined — so every `#if defined(_WIN64)` in the C
//!     is graded on its 32-bit arm. `strncmp` is the one flipped function that
//!     has such a branch; it is handled explicitly below (both arms graded),
//!     but any NEW flip must be grepped for `_WIN64` first.

// Force the SHIPPING crate's objects into the link. WITHOUT this, rustc omits
// the unused rlib entirely and the shared names (strlen/strcmp/memcpy/…)
// silently bind to GLIBC — the harness would then grade glibc against msvcrt
// and report a meaningless "MATCH". `assert_shims_live()` below proves the
// binding at runtime.
//
// This is `msvcrt-crt-rust`, NOT `cnt-rust`: msvcrt-crt-rust is what actually
// links into msvcrt.dll. Its wrappers delegate to `cnt_rust::core_rs` but
// ADAPT it to the msvcrt contract (unsigned-char compare, clamped returns, and
// two msvcrt-contract ports), so grading cnt-rust's `cabi` shims — as waves 1-2
// did, when the wrappers were verbatim copies of them — would no longer be
// grading the artifact.
extern crate msvcrt_crt_rust;

use core::ffi::c_void;

// ── the genuine msvcrt C reference (build.rs, from the real string.c) ──
extern "C" {
    // ── wave 4: the wide family, from the wcs.c reference TU ──
    fn ref_wcslen(s: *const u16) -> usize;
    fn ref_wcscmp(a: *const u16, b: *const u16) -> i32;
    fn ref_wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref_wcschr(s: *const u16, c: u16) -> *mut u16;
    fn ref_wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn ref_wcsstr(h: *const u16, n: *const u16) -> *mut u16;
    fn ref_wcsspn(s: *const u16, a: *const u16) -> usize;
    fn ref_wcscspn(s: *const u16, r: *const u16) -> usize;
    fn ref_wcspbrk(s: *const u16, a: *const u16) -> *mut u16;
    fn ref_wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn ref_wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    fn ref__wcsrev(s: *mut u16) -> *mut u16;
    fn ref__wcsset(s: *mut u16, c: u16) -> *mut u16;
    fn ref__wcsnset(s: *mut u16, c: u16, n: usize) -> *mut u16;

    // the SHIPPING Rust (msvcrt-crt-rust) wide wrappers
    fn wcslen(s: *const u16) -> usize;
    fn wcscmp(a: *const u16, b: *const u16) -> i32;
    fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn wcsstr(h: *const u16, n: *const u16) -> *mut u16;
    fn wcsspn(s: *const u16, a: *const u16) -> usize;
    fn wcscspn(s: *const u16, r: *const u16) -> usize;
    fn wcspbrk(s: *const u16, a: *const u16) -> *mut u16;
    fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    fn _wcsrev(s: *mut u16) -> *mut u16;
    fn _wcsset(s: *mut u16, c: u16) -> *mut u16;
    fn _wcsnset(s: *mut u16, c: u16, n: usize) -> *mut u16;

    fn ref_strlen(s: *const u8) -> usize;
    fn ref_strnlen(s: *const u8, m: usize) -> usize;
    fn ref_strcmp(a: *const u8, b: *const u8) -> i32;
    fn ref_strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_strchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_strrchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_strstr(h: *const u8, n: *const u8) -> *mut u8;
    fn ref_strspn(s: *const u8, a: *const u8) -> usize;
    fn ref_strcspn(s: *const u8, r: *const u8) -> usize;
    fn ref_strpbrk(s: *const u8, a: *const u8) -> *mut u8;
    fn ref_strcpy(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_strncpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn ref_strcat(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_strncat(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn ref__stricmp(a: *const u8, b: *const u8) -> i32;
    fn ref__strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref__strlwr(s: *mut u8) -> *mut u8;
    fn ref__strupr(s: *mut u8) -> *mut u8;
    fn ref__strrev(s: *mut u8) -> *mut u8;
    fn ref__strset(s: *mut u8, c: i32) -> *mut u8;
    fn ref__strnset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    fn ref_memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn ref_memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref_memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_memset(d: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn ref__memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref__memccpy(d: *mut c_void, s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn ref_atoi(s: *const u8) -> i32;
    fn ref_atol(s: *const u8) -> i32;
    fn ref__atoi64(s: *const u8) -> i64;
    fn ref_strtol(s: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn ref_strtoul(s: *const u8, e: *mut *mut u8, b: i32) -> u32;
    // wave 5 probe: the genuine msvcrt float parser (fpnum_parse + bnum.h bignum)
    fn ref_strtod(s: *const u8, e: *mut *mut u8) -> f64;
    fn ref_atof(s: *const u8) -> f64;
    // wave 5a
    fn ref__swab(src: *const u8, dst: *mut u8, len: i32);
    fn ref___strncnt(s: *const u8, size: usize) -> usize;
    fn _swab(src: *const u8, dst: *mut u8, len: i32);
    fn __strncnt(s: *const u8, size: usize) -> usize;

    // ── wave 5b: the `_s` secure family ──
    fn ref_strcpy_s(d: *mut u8, elem: usize, s: *const u8) -> i32;
    fn ref_strcat_s(d: *mut u8, elem: usize, s: *const u8) -> i32;
    fn ref_strncpy_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn ref_strncat_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn ref__strnset_s(s: *mut u8, size: usize, c: i32, n: usize) -> i32;
    fn strcpy_s(d: *mut u8, elem: usize, s: *const u8) -> i32;
    fn strcat_s(d: *mut u8, elem: usize, s: *const u8) -> i32;
    fn strncpy_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn strncat_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn _strnset_s(s: *mut u8, size: usize, c: i32, n: usize) -> i32;

    // the two errno/handler channels — SEPARATE storage per side, on purpose
    fn ref_errno_get() -> i32;          // refimpl.c  (the reference's)
    fn ref_errno_set(v: i32);
    fn ref_invalid_pmt_count() -> i32;
    fn ref_invalid_pmt_reset();
    fn subj_errno_get() -> i32;         // subjectimpl.c (the subject's)
    fn subj_errno_set(v: i32);
    fn subj_invalid_pmt_count() -> i32;
    fn subj_invalid_pmt_reset();

    // ── wave 6b: the radix `_s` family (4 contracts, 10 functions) ──
    fn ref__itoa_s(v: i32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn ref__ltoa_s(v: i32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn ref__i64toa_s(v: i64, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn ref__ui64toa_s(v: u64, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn ref__ultoa_s(v: u32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn ref__itow_s(v: i32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn ref__ltow_s(v: i32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn ref__i64tow_s(v: i64, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn ref__ui64tow_s(v: u64, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn ref__ultow_s(v: u32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn ref__strtoi64(s: *const u8, e: *mut *mut u8, b: i32) -> i64;

    fn _itoa_s(v: i32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn _ltoa_s(v: i32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn _i64toa_s(v: i64, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn _ui64toa_s(v: u64, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn _ultoa_s(v: u32, s: *mut u8, sz: usize, rx: i32) -> i32;
    fn _itow_s(v: i32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn _ltow_s(v: i32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn _i64tow_s(v: i64, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn _ui64tow_s(v: u64, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn _ultow_s(v: u32, s: *mut u16, sz: usize, rx: i32) -> i32;
    fn _strtoi64(s: *const u8, e: *mut *mut u8, b: i32) -> i64;

    // ── wave 6c: the wide `_s` family, tokenizers, memmove, _strtoui64 ──
    fn ref_wcscpy_s(d: *mut u16, e: usize, s: *const u16) -> i32;
    fn ref_wcsncpy_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn ref_wcscat_s(d: *mut u16, e: usize, s: *const u16) -> i32;
    fn ref_wcsncat_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn ref__wcsnset_s(s: *mut u16, sz: usize, c: u16, n: usize) -> i32;
    fn ref__wcsset_s(s: *mut u16, n: usize, c: u16) -> i32;
    fn ref_wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn ref_wcsnlen(s: *const u16, m: usize) -> usize;
    fn ref_strtok_s(s: *mut u8, d: *const u8, c: *mut *mut u8) -> *mut u8;
    fn ref_wcstok_s(s: *mut u16, d: *const u16, c: *mut *mut u16) -> *mut u16;
    fn ref__strtoui64(s: *const u8, e: *mut *mut u8, b: i32) -> u64;

    fn wcscpy_s(d: *mut u16, e: usize, s: *const u16) -> i32;
    fn wcsncpy_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn wcscat_s(d: *mut u16, e: usize, s: *const u16) -> i32;
    fn wcsncat_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn _wcsnset_s(s: *mut u16, sz: usize, c: u16, n: usize) -> i32;
    fn _wcsset_s(s: *mut u16, n: usize, c: u16) -> i32;
    fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn wcsnlen(s: *const u16, m: usize) -> usize;
    fn strtok_s(s: *mut u8, d: *const u8, c: *mut *mut u8) -> *mut u8;
    fn wcstok_s(s: *mut u16, d: *const u16, c: *mut *mut u16) -> *mut u16;
    fn _strtoui64(s: *const u8, e: *mut *mut u8, b: i32) -> u64;

    // ── wave 7: the case-insensitive family ──
    fn ref__strlwr_s(s: *mut u8, len: usize) -> i32;
    fn ref__strupr_s(s: *mut u8, len: usize) -> i32;
    fn _stricmp(a: *const u8, b: *const u8) -> i32;
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn _strlwr_s(s: *mut u8, len: usize) -> i32;
    fn _strupr_s(s: *mut u8, len: usize) -> i32;

    // ── wave 8: the WIDE case-insensitive family ──
    fn ref__wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn ref__wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref__wcslwr(s: *mut u16) -> *mut u16;
    fn ref__wcsupr(s: *mut u16) -> *mut u16;
    fn ref__wcslwr_s(s: *mut u16, n: usize) -> i32;
    fn ref__wcsupr_s(s: *mut u16, n: usize) -> i32;
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _wcslwr(s: *mut u16) -> *mut u16;
    fn _wcsupr(s: *mut u16) -> *mut u16;
    fn _wcslwr_s(s: *mut u16, n: usize) -> i32;
    fn _wcsupr_s(s: *mut u16, n: usize) -> i32;

    // ── wave 9: the wide ctype family ──
    fn ref_iswctype(wc: u32, ty: u16) -> i32;
    fn ref_iswalnum(wc: u32) -> i32;
    fn ref_iswalpha(wc: u32) -> i32;
    fn ref_iswcntrl(wc: u32) -> i32;
    fn ref_iswdigit(wc: u32) -> i32;
    fn ref_iswgraph(wc: u32) -> i32;
    fn ref_iswlower(wc: u32) -> i32;
    fn ref_iswprint(wc: u32) -> i32;
    fn ref_iswpunct(wc: u32) -> i32;
    fn ref_iswspace(wc: u32) -> i32;
    fn ref_iswupper(wc: u32) -> i32;
    fn ref_iswxdigit(wc: u32) -> i32;
    fn ref_iswblank(wc: u32) -> i32;
    fn iswctype(wc: u32, ty: u16) -> i32;
    fn iswalnum(wc: u32) -> i32;
    fn iswalpha(wc: u32) -> i32;
    fn iswcntrl(wc: u32) -> i32;
    fn iswdigit(wc: u32) -> i32;
    fn iswgraph(wc: u32) -> i32;
    fn iswlower(wc: u32) -> i32;
    fn iswprint(wc: u32) -> i32;
    fn iswpunct(wc: u32) -> i32;
    fn iswspace(wc: u32) -> i32;
    fn iswupper(wc: u32) -> i32;
    fn iswxdigit(wc: u32) -> i32;
    fn iswblank(wc: u32) -> i32;

    // ── wave 10: the narrow ctype/case family (ctype.c, reference TU #3) ──
    fn ref_isalnum(c: i32) -> i32;
    fn isalnum(c: i32) -> i32;
    fn ref_isalpha(c: i32) -> i32;
    fn isalpha(c: i32) -> i32;
    fn ref_iscntrl(c: i32) -> i32;
    fn iscntrl(c: i32) -> i32;
    fn ref_isdigit(c: i32) -> i32;
    fn isdigit(c: i32) -> i32;
    fn ref_isgraph(c: i32) -> i32;
    fn isgraph(c: i32) -> i32;
    fn ref_islower(c: i32) -> i32;
    fn islower(c: i32) -> i32;
    fn ref_isprint(c: i32) -> i32;
    fn isprint(c: i32) -> i32;
    fn ref_ispunct(c: i32) -> i32;
    fn ispunct(c: i32) -> i32;
    fn ref_isspace(c: i32) -> i32;
    fn isspace(c: i32) -> i32;
    fn ref_isupper(c: i32) -> i32;
    fn isupper(c: i32) -> i32;
    fn ref_isxdigit(c: i32) -> i32;
    fn isxdigit(c: i32) -> i32;
    fn ref_isblank(c: i32) -> i32;
    fn isblank(c: i32) -> i32;
    fn ref_isleadbyte(c: i32) -> i32;
    fn isleadbyte(c: i32) -> i32;
    fn ref__isctype(c: i32, ty: i32) -> i32;
    fn _isctype(c: i32, ty: i32) -> i32;
    fn ref_toupper(c: i32) -> i32;
    fn toupper(c: i32) -> i32;
    fn ref_tolower(c: i32) -> i32;
    fn tolower(c: i32) -> i32;
    fn ref__toupper(c: i32) -> i32;
    fn _toupper(c: i32) -> i32;
    fn ref__tolower(c: i32) -> i32;
    fn _tolower(c: i32) -> i32;
    fn ref__toupper_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn _toupper_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn ref__tolower_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn _tolower_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn ref___isascii(c: i32) -> i32;
    fn __isascii(c: i32) -> i32;
    fn ref___toascii(c: i32) -> i32;
    fn __toascii(c: i32) -> i32;
    fn ref___iscsym(c: i32) -> i32;
    fn __iscsym(c: i32) -> i32;
    fn ref___iscsymf(c: i32) -> i32;
    fn __iscsymf(c: i32) -> i32;
    fn ref__isblank_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn _isblank_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn ref__ispunct_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    fn _ispunct_l(c: i32, l: *mut core::ffi::c_void) -> i32;
    // the thread_data_t tokenizers (wave 10)
    fn ref_strtok(s: *mut u8, delim: *const u8) -> *mut u8;
    fn strtok(s: *mut u8, delim: *const u8) -> *mut u8;
    fn ref_wcstok(s: *mut u16, delim: *const u16) -> *mut u16;
    fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16;
    // wave 11: the locale-mode switch (one per side — separate locinfo storage)
    fn ref_set_nls(on: i32);
    fn subj_set_nls(on: i32);
    // wave 11: the wide case mappers (wint_t is 16-bit in msvcrt)
    fn ref_towlower(c: u16) -> u16;
    fn towlower(c: u16) -> u16;
    fn ref_towupper(c: u16) -> u16;
    fn towupper(c: u16) -> u16;
    fn ref__towlower_l(c: u16, l: *mut core::ffi::c_void) -> u16;
    fn _towlower_l(c: u16, l: *mut core::ffi::c_void) -> u16;
    fn ref__towupper_l(c: u16, l: *mut core::ffi::c_void) -> u16;
    fn _towupper_l(c: u16, l: *mut core::ffi::c_void) -> u16;
    // ── wave 12: the collation family ──
    fn ref_strcoll(a: *const u8, b: *const u8) -> i32;
    fn strcoll(a: *const u8, b: *const u8) -> i32;
    fn ref__stricoll(a: *const u8, b: *const u8) -> i32;
    fn _stricoll(a: *const u8, b: *const u8) -> i32;
    fn ref__strncoll(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _strncoll(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref__strnicoll(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _strnicoll(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_strxfrm(d: *mut u8, s: *const u8, n: usize) -> usize;
    fn strxfrm(d: *mut u8, s: *const u8, n: usize) -> usize;
    fn ref_wcscoll(a: *const u16, b: *const u16) -> i32;
    fn wcscoll(a: *const u16, b: *const u16) -> i32;
    fn ref__wcsicoll(a: *const u16, b: *const u16) -> i32;
    fn _wcsicoll(a: *const u16, b: *const u16) -> i32;
    fn ref__wcsncoll(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _wcsncoll(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref__wcsnicoll(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _wcsnicoll(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref_wcsxfrm(d: *mut u16, s: *const u16, n: usize) -> usize;
    fn wcsxfrm(d: *mut u16, s: *const u16, n: usize) -> usize;
}

/// The four observable effects of a CRT call, as one comparable unit.
///
/// WHY THIS TYPE EXISTS. A CRT function's contract is not just "what it wrote"
/// and "what it returned". For any function that can fail it is also:
///
///   3. **errno**  — the value an app tests after the call
///   4. **did the invalid-parameter handler FIRE** — an app installs one with
///      `_set_invalid_parameter_handler()`, and it may `abort()`
///
/// Effect 4 was the harness's blind spot. `subj_invalid_pmt_count()` existed
/// from wave 5 and *nothing ever read it* — the wave-5 comment claimed to grade
/// "all four observable effects" while grading three. That mattered: msvcrt's
/// radix family has THREE mutually-inconsistent error contracts, and the ONLY
/// thing separating `_ultoa_s` (sets errno, does NOT fire the handler — it
/// hand-rolls `*_errno() = EINVAL`) from `_ltoa_s` (fires it, via
/// MSVCRT_CHECK_PMT) is effect 4. Grading 3-of-4 would have called them equal
/// and shipped a silently wrong `_ultoa_s`.
///
/// Bundling them means a new row cannot *forget* to check one.
#[derive(PartialEq, Debug)]
struct Effects { ret: i64, errno: i32, pmt: i32 }

/// errno is seeded with THIS, not with 0, before every graded call.
///
/// # Why not 0 (wave 13, and it was a real blind spot for twelve waves)
///
/// The harness used to zero errno before each call. That makes a function which *itself*
/// zeroes errno **invisible** — errno was already 0, so the write changes nothing anybody
/// can observe. And msvcrt has exactly such a function:
///
/// ```c
/// static inline double strtod_helper(..., int *perr) {
///     if (perr) *perr = 0;
/// #if _MSVCR_VER == 0
///     else *_errno() = 0;          /* <-- strtod/atof CLEAR errno on entry */
/// #endif
/// ```
///
/// while its wide twin `_wcstod_l` — written out separately in wcs.c — **does not**. So
/// `strtod("abc")` clears a pre-existing errno and `wcstod(L"abc")` preserves it. A caller
/// that sets `errno = 0`, calls one, and tests for ERANGE gets different answers from the
/// two. That is a genuine, observable difference in the CRT, and the old harness could not
/// see it: sabotaging `_wcstod_l` to clear errno moved **zero rows**.
///
/// Seeding a sentinel instead costs nothing (both sides get the same seed, so any function
/// that leaves errno alone still reports the sentinel on both sides and still MATCHes) and
/// turns "did you touch errno at all?" into an observable. The value is deliberately not a
/// real errno constant, so it can never be confused with one the code under test set.
///
/// The general shape, and the third time this lane has met it: **the harness's own setup is
/// part of the oracle.** Zeroing a side channel before the call is a claim that nobody
/// clears it — and that claim was false. (§9.73: a convenience stub IS an oracle.)
const ERRNO_SEED: i32 = 0x5A5A;

impl Effects {
    /// Run `f` on the SUBJECT side with both side-channels seeded first.
    fn subject(f: impl FnOnce() -> i64) -> Effects {
        unsafe {
            subj_errno_set(ERRNO_SEED);
            subj_invalid_pmt_reset();
            let ret = f();
            Effects { ret, errno: subj_errno_get(), pmt: subj_invalid_pmt_count() }
        }
    }
    /// Run `f` on the REFERENCE side with *its* (separate) side-channels seeded.
    fn reference(f: impl FnOnce() -> i64) -> Effects {
        unsafe {
            ref_errno_set(ERRNO_SEED);
            ref_invalid_pmt_reset();
            let ret = f();
            Effects { ret, errno: ref_errno_get(), pmt: ref_invalid_pmt_count() }
        }
    }
}

impl core::fmt::Display for Effects {
    fn fmt(&self, w: &mut core::fmt::Formatter) -> core::fmt::Result {
        let e = match self.errno { 0 => "-".into(), 22 => "EINVAL".into(), 34 => "ERANGE".into(),
                                   80 => "STRUNCATE".into(), n => format!("errno{n}") };
        write!(w, "ret={} errno={} handler={}", self.ret, e, if self.pmt > 0 { "FIRED" } else { "no" })
    }
}

/// Grade a function on ALL FOUR effects at once (no buffer involved).
fn check_effects(name: &'static str, mut f: impl FnMut(&mut Rng) -> (Effects, Effects, String)) {
    let mut r = Rng(0x14057B7EF767814F);
    for _ in 0..20_000 {
        let (got, want, ctx) = f(&mut r);
        if got != want {
            record(name, V::Diverge, format!("[{ctx}]\n        cnt   = {got}\n        msvcrt= {want}"));
            return;
        }
    }
    record(name, V::Match, String::new());
}

/// Buffer AND all four effects — for the `_s` family, whose contract is both.
///
/// The FULL destination is compared, not just the string up to the NUL. That is
/// deliberate and load-bearing: on a too-small buffer msvcrt's radix `_s` writes
/// the digits BACKWARDS into the caller's buffer and only then NULs index 0, so
/// everything past `[0]` is reversed residue. A "clean" implementation that just
/// does `str[0] = 0; return ERANGE;` is string-identical and byte-different.
/// What outcomes did the REFERENCE actually produce across the fuzz?
///
/// A row that reports MATCH having only ever exercised the happy path has proved
/// nothing about the error contracts — and the error contracts are the whole
/// reason the `_s` family is hard. This is the "a fuzzer only falsifies what it
/// VARIES" rule (§9.60) turned into a measurement: if `ERANGE` never appears in
/// the coverage line, the reversed-residue arm was never once executed and its
/// green is worthless.
///
/// Enforced, not just printed: `require` names the outcomes a row MUST hit.
#[derive(Default)]
struct Cov { ok: u32, einval: u32, erange: u32, fired: u32 }
impl Cov {
    fn see(&mut self, e: &Effects) {
        match e.ret as i32 { 0 => self.ok += 1, 22 => self.einval += 1, 34 => self.erange += 1, _ => {} }
        if e.pmt > 0 { self.fired += 1; }
    }
    fn line(&self) -> String {
        format!("ok={} EINVAL={} ERANGE={} handler-fired={}", self.ok, self.einval, self.erange, self.fired)
    }
    /// Returns the outcomes that were REQUIRED but never seen.
    fn missing(&self, require: &[&'static str]) -> Vec<&'static str> {
        require.iter().copied().filter(|w| match *w {
            "ok" => self.ok == 0,
            "EINVAL" => self.einval == 0,
            "ERANGE" => self.erange == 0,
            "handler" => self.fired == 0,
            _ => false,
        }).collect()
    }
}

fn check_buf_fx(name: &'static str, require: &[&'static str],
                mut f: impl FnMut(&mut Rng) -> (Vec<u8>, Effects, Vec<u8>, Effects, String)) {
    let mut r = Rng(0x8EBC6AF09C88C6E3);
    let mut cov = Cov::default();
    for _ in 0..20_000 {
        let (gb, ge, wb, we, ctx) = f(&mut r);
        cov.see(&we);
        if gb != wb || ge != we {
            record(name, V::Diverge, format!("[{ctx}]\n        cnt   = {ge}  buf={gb:?}\n        msvcrt= {we}  buf={wb:?}"));
            return;
        }
    }
    let missing = cov.missing(require);
    if !missing.is_empty() {
        record(name, V::Diverge, format!("UNEXERCISED CONTRACT — the fuzz never reached {missing:?}.\n        \
             This MATCH is worthless: the branch was never run. Widen the generator.\n        coverage: {}", cov.line()));
        return;
    }
    record(name, V::Match, cov.line());
}

/// `check_buf_fx`'s wide twin.
fn check_wbuf_fx(name: &'static str, require: &[&'static str],
                 mut f: impl FnMut(&mut Rng) -> (Vec<u16>, Effects, Vec<u16>, Effects, String)) {
    let mut r = Rng(0x5851F42D4C957F2D);
    let mut cov = Cov::default();
    for _ in 0..20_000 {
        let (gb, ge, wb, we, ctx) = f(&mut r);
        cov.see(&we);
        if gb != wb || ge != we {
            record(name, V::Diverge, format!("[{ctx}]\n        cnt   = {ge}  buf={gb:?}\n        msvcrt= {we}  buf={wb:?}"));
            return;
        }
    }
    let missing = cov.missing(require);
    if !missing.is_empty() {
        record(name, V::Diverge, format!("UNEXERCISED CONTRACT — the fuzz never reached {missing:?}.\n        \
             This MATCH is worthless: the branch was never run. Widen the generator.\n        coverage: {}", cov.line()));
        return;
    }
    record(name, V::Match, cov.line());
}

// ── cnt-rust's #[no_mangle] shims (the code under test) ──
extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn strnlen(s: *const u8, m: usize) -> usize;
    fn strcmp(a: *const u8, b: *const u8) -> i32;
    fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn strchr(s: *const u8, c: i32) -> *mut u8;
    fn strrchr(s: *const u8, c: i32) -> *mut u8;
    fn strstr(h: *const u8, n: *const u8) -> *mut u8;
    fn strspn(s: *const u8, a: *const u8) -> usize;
    fn strcspn(s: *const u8, r: *const u8) -> usize;
    fn strpbrk(s: *const u8, a: *const u8) -> *mut u8;
    fn strcpy(d: *mut u8, s: *const u8) -> *mut u8;
    fn strncpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn strcat(d: *mut u8, s: *const u8) -> *mut u8;
    fn strncat(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn _strlwr(s: *mut u8) -> *mut u8;
    fn _strupr(s: *mut u8) -> *mut u8;
    fn _strrev(s: *mut u8) -> *mut u8;
    fn _strset(s: *mut u8, c: i32) -> *mut u8;
    fn _strnset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    fn memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn memset(d: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn _memccpy(d: *mut c_void, s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn atoi(s: *const u8) -> i32;
    fn atol(s: *const u8) -> i32;
    fn _atoi64(s: *const u8) -> i64;
    fn strtol(s: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn strtoul(s: *const u8, e: *mut *mut u8, b: i32) -> u32;
}

/// msvcrt's comparison return clamp (`if (a > b) return 1; if (a < b) return -1; return 0;`).
fn clamp(d: i32) -> i32 { if d > 0 { 1 } else if d < 0 { -1 } else { 0 } }

// ── deterministic RNG ──
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 17
    }
    fn below(&mut self, n: usize) -> usize { (self.next() as usize) % n.max(1) }
}

/// Alphabets chosen to provoke the known divergence shapes:
/// high bytes (signed-char compare), letters (case fold), digits (conversion).
fn gen_str(r: &mut Rng, maxlen: usize) -> Vec<u8> {
    let n = r.below(maxlen);
    let mut v = Vec::with_capacity(n + 1);
    for _ in 0..n {
        let b = match r.below(6) {
            0 => b'a' + (r.below(26) as u8),
            1 => b'A' + (r.below(26) as u8),
            2 => b'0' + (r.below(10) as u8),
            3 => 0x80u8.wrapping_add(r.below(0x80) as u8), // high/negative chars
            4 => b" \t+-.xX"[r.below(7)],
            _ => 1 + (r.below(254) as u8),
        };
        v.push(b);
    }
    v.push(0);
    v
}

/// The WIDE alphabet (wave 4). Deliberately weighted toward u16 values with the
/// TOP BIT SET (0x8000..=0xFFFF): those are exactly where a signed-vs-unsigned
/// element compare would diverge, which is the bug class that bit the narrow
/// family in wave 3. If cnt-rust's `Elem for u16` were ever "fixed" to a signed
/// diff, `wcscmp`/`wcsncmp` would flip sign on these and this fuzz would catch
/// it. Also includes 0x0080..=0x00FF (the bytes that broke the narrow side) and
/// lone surrogates, which are legal wchar_t values msvcrt treats as opaque.
fn gen_wstr(r: &mut Rng, maxlen: usize) -> Vec<u16> {
    let n = r.below(maxlen);
    let mut v = Vec::with_capacity(n + 1);
    for _ in 0..n {
        let w = match r.below(6) {
            0 => (b'a' + (r.below(26) as u8)) as u16,
            1 => (b'A' + (r.below(26) as u8)) as u16,
            2 => (b'0' + (r.below(10) as u8)) as u16,
            3 => 0x80u16 + (r.below(0x80) as u16), // the narrow high-byte range
            4 => 0x8000u16 + (r.below(0x8000) as u16), // TOP BIT SET
            _ => 1 + (r.below(0xFFFE) as u16),
        };
        v.push(w);
    }
    v.push(0);
    v
}

/// Float-literal strings, weighted toward the shapes that separate a naive
/// accumulator from a correctly-rounded bignum: long mantissas, extreme
/// exponents, and hard-to-round halfway cases.
fn gen_float_str(r: &mut Rng) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    match r.below(8) {
        0 => { // simple
            for _ in 0..1 + r.below(3) { v.push(b'0' + r.below(10) as u8); }
            v.push(b'.');
            for _ in 0..1 + r.below(3) { v.push(b'0' + r.below(10) as u8); }
        }
        1 => { // LONG mantissa — where naive accumulation loses bits
            for _ in 0..17 + r.below(6) { v.push(b'0' + r.below(10) as u8); }
            v.push(b'.');
            for _ in 0..17 + r.below(6) { v.push(b'0' + r.below(10) as u8); }
        }
        2 => { // extreme exponent
            for _ in 0..1 + r.below(4) { v.push(b'0' + r.below(10) as u8); }
            v.push(b'.');
            for _ in 0..1 + r.below(8) { v.push(b'0' + r.below(10) as u8); }
            v.push(if r.below(2) == 0 { b'e' } else { b'E' });
            if r.below(2) == 0 { v.push(b'-'); }
            v.extend_from_slice(format!("{}", r.below(320)).as_bytes());
        }
        3 => { // subnormal / near-zero
            v.extend_from_slice(b"0.");
            for _ in 0..r.below(20) { v.push(b'0'); }
            for _ in 0..1 + r.below(17) { v.push(b'0' + r.below(10) as u8); }
            v.extend_from_slice(b"e-3");
            v.push(b'0' + r.below(10) as u8);
        }
        4 => { // halfway-ish: ...5 at the rounding boundary
            for _ in 0..15 + r.below(4) { v.push(b'0' + r.below(10) as u8); }
            v.extend_from_slice(b"5e");
            if r.below(2) == 0 { v.push(b'-'); }
            v.extend_from_slice(format!("{}", r.below(40)).as_bytes());
        }
        5 => { // signed + leading space (the DJ Delorie *sret quirk)
            if r.below(2) == 0 { v.push(b' '); }
            v.push(if r.below(2) == 0 { b'-' } else { b'+' });
            for _ in 0..1 + r.below(8) { v.push(b'0' + r.below(10) as u8); }
            v.push(b'.');
            for _ in 0..1 + r.below(8) { v.push(b'0' + r.below(10) as u8); }
        }
        6 => { // trailing junk (tests the end pointer)
            for _ in 0..1 + r.below(6) { v.push(b'0' + r.below(10) as u8); }
            v.push(b'.');
            for _ in 0..1 + r.below(6) { v.push(b'0' + r.below(10) as u8); }
            v.extend_from_slice(b"xyz");
        }
        _ => { // integer-only
            for _ in 0..1 + r.below(19) { v.push(b'0' + r.below(10) as u8); }
        }
    }
    v.push(0);
    v
}

// ── verdict bookkeeping ──
#[derive(PartialEq)]
enum V { Match, SignOnly, Diverge }
struct Res { name: &'static str, v: V, note: String }
static mut RESULTS: Vec<Res> = Vec::new();

fn record(name: &'static str, v: V, note: String) {
    unsafe { (*core::ptr::addr_of_mut!(RESULTS)).push(Res { name, v, note }) }
}

/// Compare a pure integer-returning fn. `cmp_semantics` = treat as a
/// comparison function (sign is the contract; magnitude may differ).
fn check_int(name: &'static str, cmp_semantics: bool, mut f: impl FnMut(&mut Rng) -> (i64, i64, String)) {
    let mut r = Rng(0x9E3779B97F4A7C15);
    let mut sign_only = false;
    for _ in 0..20_000 {
        let (got, want, ctx) = f(&mut r);
        if got != want {
            if cmp_semantics && got.signum() == want.signum() {
                sign_only = true;
                continue;
            }
            record(name, V::Diverge, format!("cnt={got} msvcrt={want}  [{ctx}]"));
            return;
        }
    }
    if sign_only {
        record(name, V::SignOnly, "same sign, different magnitude (raw byte-diff vs clamped)".into());
    } else {
        record(name, V::Match, String::new());
    }
}

/// Compare a fn that mutates a destination buffer: both the returned pointer
/// OFFSET and the full destination bytes must agree.
fn check_buf(name: &'static str, mut f: impl FnMut(&mut Rng) -> (Vec<u8>, isize, Vec<u8>, isize, String)) {
    let mut r = Rng(0xD1B54A32D192ED03);
    for _ in 0..20_000 {
        let (gb, go, wb, wo, ctx) = f(&mut r);
        if gb != wb || go != wo {
            record(name, V::Diverge, format!("buf/ret differ [{ctx}]\n        cnt   ={gb:?} ret+{go}\n        msvcrt={wb:?} ret+{wo}"));
            return;
        }
    }
    record(name, V::Match, String::new());
}

/// `check_buf`'s wide twin: both the returned pointer offset (in ELEMENTS) and
/// the full u16 destination must agree.
fn check_wbuf(name: &'static str, mut f: impl FnMut(&mut Rng) -> (Vec<u16>, isize, Vec<u16>, isize, String)) {
    let mut r = Rng(0x2545F4914F6CDD1D);
    for _ in 0..20_000 {
        let (gb, go, wb, wo, ctx) = f(&mut r);
        if gb != wb || go != wo {
            record(name, V::Diverge, format!("buf/ret differ [{ctx}]\n        cnt   ={gb:04x?} ret+{go}\n        msvcrt={wb:04x?} ret+{wo}"));
            return;
        }
    }
    record(name, V::Match, String::new());
}

/// Prove the shared-name shims resolve to MSVCRT-CRT-RUST, not glibc. The
/// wrappers' `cstr_slice` NULL-guards (NULL → empty slice), so `strlen(NULL)`
/// is 0; glibc's would fault. If this ever silently "passes" against glibc the
/// whole table is worthless, so it is a hard gate, not a warning.
unsafe fn assert_shims_live() {
    let n = strlen(core::ptr::null());
    assert_eq!(n, 0, "strlen(NULL)!=0 — shared-name shims are NOT msvcrt-crt-rust's");
    // _strrev has no glibc counterpart: linking at all proves the crate is present.
    let mut b = *b"ab\0";
    let _ = _strrev(b.as_mut_ptr());
    assert_eq!(&b, b"ba\0", "_strrev misbehaved — bad link");
    // _strnset returns the ADVANCED pointer in msvcrt (cnt-rust returns the
    // start). This distinguishes the SHIPPING wrapper from cnt-rust's cabi
    // shim, i.e. it proves we are grading the artifact and not the proxy.
    let mut t = *b"abcd\0";
    let p = _strnset(t.as_mut_ptr(), b'z' as i32, 2);
    assert_eq!(p as usize - t.as_ptr() as usize, 2, "_strnset returned the START — that is cnt-rust's contract, not msvcrt's: the harness is linked against the wrong crate");

    // Wave 4: the same gate for the WIDE shims. glibc also exports `wcslen`, so
    // without this we could be grading glibc's (which would fault on NULL, not
    // return 0) and never know.
    let wn = wcslen(core::ptr::null());
    assert_eq!(wn, 0, "wcslen(NULL)!=0 — the wide shims are NOT msvcrt-crt-rust's");
    let mut wb = [b'a' as u16, b'b' as u16, 0];
    let _ = _wcsrev(wb.as_mut_ptr());
    assert_eq!(wb, [b'b' as u16, b'a' as u16, 0], "_wcsrev misbehaved — bad link");

    println!("  [self-check] msvcrt-crt-rust shims are live — narrow (strlen(NULL)==0, _strrev ok, _strnset returns str+n) and wide (wcslen(NULL)==0, _wcsrev ok)");

    assert_null_tolerant();
}

/// ⚠️ **THE BLIND SPOT THAT SHIPPED A REGRESSION (field manual §9.64).**
///
/// The 20k-case fuzz generates real NUL-terminated strings. It NEVER passes NULL.
/// So when the single-pass perf rewrite replaced `cstr_slice(p)` (which
/// NULL-guards, returning `&[]`) with a raw pointer walk, msvcrt.dll silently
/// went from *NULL behaves like ""* — its shipped, gate-green behaviour — to
/// *NULL faults*. `classify` reported 45/45 MATCH. The net suite went 15/15 ->
/// 13/15, and only an A/B control against the previous commit found it.
///
/// A fuzzer can only falsify what it VARIES (§9.60), and it was not varying
/// nullness. So NULL is now a first-class, asserted input.
///
/// NB: this is graded against the CRATE'S OWN contract, not against the C — the
/// genuine msvcrt dereferences and dies here, so there is no `ref_*` to compare
/// with. Being *more* forgiving than the C is fine; silently becoming LESS
/// forgiving than the last shipped build is what broke the boot.
unsafe fn assert_null_tolerant() {
    let s = b"abc\0";
    let e = b"\0";
    let (sp, ep) = (s.as_ptr(), e.as_ptr());

    // NULL == "" for every reader we flipped.
    assert!(strcmp(core::ptr::null(), ep) == 0, "strcmp(NULL, \"\") != 0");
    assert!(strcmp(core::ptr::null(), sp) < 0, "strcmp(NULL, \"abc\") not < 0");
    assert!(strcmp(sp, core::ptr::null()) > 0, "strcmp(\"abc\", NULL) not > 0");
    assert_eq!(strncmp(core::ptr::null(), ep, 4), 0, "strncmp(NULL, \"\") != 0");
    assert_eq!(strspn(core::ptr::null(), sp), 0, "strspn(NULL, ..) != 0");
    assert_eq!(strcspn(core::ptr::null(), sp), 0, "strcspn(NULL, ..) != 0");
    assert!(strpbrk(core::ptr::null(), sp).is_null(), "strpbrk(NULL, ..) != NULL");
    assert!(strchr(core::ptr::null(), b'a' as i32).is_null(), "strchr(NULL, 'a') != NULL");
    assert!(strrchr(core::ptr::null(), b'a' as i32).is_null(), "strrchr(NULL, 'a') != NULL");
    // a NULL SET is empty, so nothing is in it / nothing is rejected
    assert_eq!(strspn(sp, core::ptr::null()), 0, "strspn(.., NULL set) != 0");
    assert_eq!(strcspn(sp, core::ptr::null()), 3, "strcspn(.., NULL set) != len");

    // NULL source copies as "" (writes just the terminator).
    let mut d = [0xAAu8; 8];
    strcpy(d.as_mut_ptr(), core::ptr::null());
    assert_eq!(d[0], 0, "strcpy(d, NULL) did not write \"\"");
    let mut d2 = *b"xy\0\0\0\0\0\0";
    strcat(d2.as_mut_ptr(), core::ptr::null());
    assert_eq!(&d2[..3], b"xy\0", "strcat(d, NULL) perturbed d");

    // ── the wide half ──
    let ws = [b'a' as u16, b'b' as u16, b'c' as u16, 0];
    let we = [0u16];
    let (wsp, wep) = (ws.as_ptr(), we.as_ptr());
    assert!(wcscmp(core::ptr::null(), wep) == 0, "wcscmp(NULL, L\"\") != 0");
    assert!(wcscmp(core::ptr::null(), wsp) < 0, "wcscmp(NULL, L\"abc\") not < 0");
    assert_eq!(wcsncmp(core::ptr::null(), wep, 4), 0, "wcsncmp(NULL, L\"\") != 0");
    assert_eq!(wcsspn(core::ptr::null(), wsp), 0, "wcsspn(NULL, ..) != 0");
    assert_eq!(wcscspn(core::ptr::null(), wsp), 0, "wcscspn(NULL, ..) != 0");
    assert!(wcspbrk(core::ptr::null(), wsp).is_null(), "wcspbrk(NULL, ..) != NULL");
    assert!(wcschr(core::ptr::null(), b'a' as u16).is_null(), "wcschr(NULL, 'a') != NULL");
    assert!(wcsrchr(core::ptr::null(), b'a' as u16).is_null(), "wcsrchr(NULL, 'a') != NULL");
    assert!(wcsstr(core::ptr::null(), wsp).is_null(), "wcsstr(NULL, ..) != NULL");
    assert_eq!(wcsspn(wsp, core::ptr::null()), 0, "wcsspn(.., NULL set) != 0");
    assert_eq!(wcscspn(wsp, core::ptr::null()), 3, "wcscspn(.., NULL set) != len");

    let mut wd = [0xAAAAu16; 8];
    wcscpy(wd.as_mut_ptr(), core::ptr::null());
    assert_eq!(wd[0], 0, "wcscpy(d, NULL) did not write L\"\"");

    println!("  [self-check] NULL is tolerated as \"\" by every flipped reader (narrow + wide) — the §9.64 regression class");
}

fn main() {
    unsafe { assert_shims_live() }
    unsafe { run() }
    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut s, mut d) = (0, 0, 0);
    println!("\n=== msvcrt-crt-rust (SHIPPING) vs GENUINE msvcrt/string.c — flip safety ===\n");
    for r in res {
        let tag = match r.v { V::Match => { m += 1; "MATCH     ✓ safe to flip" }
                              V::SignOnly => { s += 1; "SIGN-ONLY ⚠ review" }
                              V::Diverge => { d += 1; "DIVERGE   ✗ DO NOT FLIP" } };
        println!("  {:<12} {}", r.name, tag);
        if !r.note.is_empty() { println!("        {}", r.note); }
    }
    println!("\n  {m} MATCH   {s} SIGN-ONLY   {d} DIVERGE   (of {} tested)", res.len());

    // ── THE GATE ────────────────────────────────────────────────────────────
    // Until now this table was INFORMATIONAL: it printed `DIVERGE ✗ DO NOT FLIP`
    // and exited 0. A divergence in a function that is ALREADY FLIPPED is not a
    // suggestion — it means msvcrt.dll is shipping a wrong answer. Fail.
    //
    // `[PROBE]` rows are exempt BY DESIGN. A probe is a non-`#[no_mangle]` fn that
    // asks "COULD we flip this?" before anything overrides the C — so a DIVERGE
    // there is the probe WORKING, and a permanent, executable record of why that
    // family is off-limits. (See `strtod [PROBE]`: cnt-rust's DJ Delorie
    // accumulator vs msvcrt's correctly-rounded bignum — measured 14 ULPs apart.)
    let shipped_divergences: Vec<&str> = res
        .iter()
        .filter(|r| r.v == V::Diverge && !r.name.contains("[PROBE]"))
        .map(|r| r.name)
        .collect();
    if !shipped_divergences.is_empty() {
        println!(
            "\n  ✗ FAIL — these are FLIPPED and DIVERGENT, i.e. msvcrt.dll is shipping\n    \
             a wrong answer for them: {shipped_divergences:?}\n"
        );
        std::process::exit(1);
    }

    println!("\n  PROBES (graded, deliberately NOT flipped — the C still ships):");
    println!("    strtod [PROBE] — DIVERGE is the EXPECTED, LOAD-BEARING result.");
    println!("      cnt-rust's core is ReactOS libcntpr's DJ Delorie parser: a naive");
    println!("      `r = r*10 + d` accumulator, explicitly never correctly-rounded.");
    println!("      msvcrt's is Wine's `fpnum_parse` + `bnum.h` — a base-10 BIGNUM,");
    println!("      correctly rounded. Measured 14 ULPs apart on a hard-to-round input.");
    println!("      => the \"assemble from cnt-rust\" thesis DOES NOT REACH the float");
    println!("      family. And note _ecvt/_fcvt/_gcvt are literally `_snprintf(\"%.*le\")`");
    println!("      calls, so float↔str is not a separate wave at all — it is PART OF");
    println!("      THE PRINTF ENGINE (Phase C). Do not flip strtod/atof from cnt-rust.\n");

    println!("  DEFERRED (not graded, not flipped — locale-dependent, Phase D):");
    println!("    narrow: _stricmp / _strnicmp / _memicmp");
    println!("    wide:   _wcsicmp / _wcsnicmp / _wcsicoll / _wcsnicoll / wcscoll");
    println!("            _wcslwr / _wcsupr / towlower / towupper / the isw* ctype block");
    println!("      msvcrt folds via get_locinfo()->lc_handle[LC_CTYPE] + _tolower_l;");
    println!("      cnt-rust's fold is locale-blind ASCII. They agree ONLY in the \"C\"");
    println!("      locale — which is exactly what this harness pins, so a MATCH here");
    println!("      would be a harness artifact, not a license to flip. See lane plan");
    println!("      and field manual §9.60 (\"a harness can only falsify what it VARIES\").\n");
}

unsafe fn run() {
    // ---- pure readers ----
    check_int("strlen", false, |r| { let a = gen_str(r, 40); (strlen(a.as_ptr()) as i64, ref_strlen(a.as_ptr()) as i64, String::new()) });
    check_int("strnlen", false, |r| { let a = gen_str(r, 40); let n = r.below(45); (strnlen(a.as_ptr(), n) as i64, ref_strnlen(a.as_ptr(), n) as i64, format!("n={n}")) });
    check_int("strcmp", false, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) };
        (strcmp(a.as_ptr(), b.as_ptr()) as i64, ref_strcmp(a.as_ptr(), b.as_ptr()) as i64, format!("{a:?} vs {b:?}")) });
    // strncmp ships TWO contracts (string.c `#if defined(_WIN64)`, with
    // _MSVCR_VER=0): i386 returns the raw unsigned diff, amd64/arm64 clamp to
    // -1/0/1. The host reference compiled the i386 arm (_WIN64 undefined), so:
    //   * the i386 arm is graded DIRECTLY against it (`strncmp_raw`);
    //   * the _WIN64 arm is graded against `clamp(ref)` — an EXACT model, not
    //     an approximation: both C branches run after the identical `while`
    //     loop and then compare the same two bytes, so _WIN64 == sign(#else).
    // Together these grade both shipped contracts from one reference compile.
    check_int("strncmp[i386 raw]", false, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) }; let n = r.below(22);
        (msvcrt_crt_rust::strncmp_raw(a.as_ptr(), b.as_ptr(), n) as i64, ref_strncmp(a.as_ptr(), b.as_ptr(), n) as i64, format!("n={n} {a:?} vs {b:?}")) });
    check_int("strncmp[_WIN64]", false, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) }; let n = r.below(22);
        let want = clamp(ref_strncmp(a.as_ptr(), b.as_ptr(), n));
        let got = if cfg!(target_pointer_width = "64") { strncmp(a.as_ptr(), b.as_ptr(), n) } else { clamp(strncmp(a.as_ptr(), b.as_ptr(), n)) };
        (got as i64, want as i64, format!("n={n} {a:?} vs {b:?}")) });
    // pointer-returning searchers: compare the OFFSET into the input (or -1 for NULL)
    macro_rules! off { ($p:expr, $base:expr) => { if $p.is_null() { -1i64 } else { ($p as usize - $base.as_ptr() as usize) as i64 } } }
    check_int("strchr", false, |r| { let a = gen_str(r, 30); let c = (r.below(300) as i32) - 20;
        (off!(strchr(a.as_ptr(), c), a), off!(ref_strchr(a.as_ptr(), c), a), format!("c={c} {a:?}")) });
    check_int("strrchr", false, |r| { let a = gen_str(r, 30); let c = (r.below(300) as i32) - 20;
        (off!(strrchr(a.as_ptr(), c), a), off!(ref_strrchr(a.as_ptr(), c), a), format!("c={c} {a:?}")) });
    check_int("strstr", false, |r| { let h = gen_str(r, 30); let n = if r.below(2) == 0 && h.len() > 2 { let s = r.below(h.len() - 1); let e = s + r.below(h.len() - s); let mut v = h[s..e].to_vec(); v.push(0); v } else { gen_str(r, 6) };
        (off!(strstr(h.as_ptr(), n.as_ptr()), h), off!(ref_strstr(h.as_ptr(), n.as_ptr()), h), format!("{h:?} / {n:?}")) });
    check_int("strspn", false, |r| { let a = gen_str(r, 25); let b = gen_str(r, 8);
        (strspn(a.as_ptr(), b.as_ptr()) as i64, ref_strspn(a.as_ptr(), b.as_ptr()) as i64, format!("{a:?} / {b:?}")) });
    check_int("strcspn", false, |r| { let a = gen_str(r, 25); let b = gen_str(r, 8);
        (strcspn(a.as_ptr(), b.as_ptr()) as i64, ref_strcspn(a.as_ptr(), b.as_ptr()) as i64, format!("{a:?} / {b:?}")) });
    check_int("strpbrk", false, |r| { let a = gen_str(r, 25); let b = gen_str(r, 8);
        (off!(strpbrk(a.as_ptr(), b.as_ptr()), a), off!(ref_strpbrk(a.as_ptr(), b.as_ptr()), a), format!("{a:?} / {b:?}")) });
    check_int("memcmp", false, |r| { let a = gen_str(r, 25); let mut b = a.clone(); if r.below(2) == 0 && !b.is_empty() { let i = r.below(b.len()); b[i] = b[i].wrapping_add(1); } let n = r.below(a.len() + 1);
        (memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, ref_memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, format!("n={n}")) });
    check_int("memchr", false, |r| { let a = gen_str(r, 30); let c = (r.below(300) as i32) - 20; let n = r.below(a.len() + 1);
        (if memchr(a.as_ptr().cast(), c, n).is_null() { -1 } else { memchr(a.as_ptr().cast(), c, n) as i64 - a.as_ptr() as i64 },
         if ref_memchr(a.as_ptr().cast(), c, n).is_null() { -1 } else { ref_memchr(a.as_ptr().cast(), c, n) as i64 - a.as_ptr() as i64 }, format!("c={c} n={n}")) });

    // ---- conversions ----
    //
    // ═════ WAVE 6a: the integer-parse family, RE-GRADED through all 4 effects ═════
    //
    // These were flipped in waves 1-3 and passed — but the grader could only see
    // VALUE and ENDPTR, because errno/handler observability did not exist until
    // wave 5 built it. A stronger oracle is worth nothing if you only ever point
    // it at NEW code, so they are re-entered here through `check_effects`.
    //
    // Two blind spots in the OLD rows, both deliberate now:
    //   * the base set was [0,2,8,10,16,36] — every one of them VALID, so the
    //     `MSVCRT_CHECK_PMT(base == 0 || base >= 2)` / `(base <= 36)` rejection
    //     path was never once executed in 20,000 cases.
    //   * NULL was never passed, so `CHECK_PMT(nptr != NULL)` was never executed.
    //
    // Both now are. `gen_str` already emits 0x80-0xFF and control bytes, which is
    // what exercises the `_isspace_l()` leading-whitespace boundary — the one
    // place these functions consult the locale. refimpl models the "C" locale, so
    // a MATCH here proves C-locale equivalence, which is what msvcrt.dll ships
    // with. (A non-C locale could still diverge; that is Phase D's problem and is
    // recorded as such in the plan.)
    const BASES: [i32; 9] = [0, 2, 8, 10, 16, 36, /* now the INVALID ones: */ 1, 37, -1];
    const BASES2: [i32; 9] = BASES;

    // `gen_str` is a general string fuzzer: random bytes with digits sprinkled in.
    // The chance it ever emits a 19-DIGIT RUN is ~zero — so in 20,000 cases it
    // never once drove the parse past _I64_MAX, and the entire saturate/ERANGE
    // arm of these functions went untested. That is how `_atoi64` shipped on the
    // kernel's WRAPPING parser while msvcrt's SATURATES: a value divergence that
    // a value-comparing fuzzer could not see, purely for want of an input.
    //
    // A fuzzer only falsifies what it VARIES (§9.60). So generate the boundaries
    // on purpose: long digit runs, and the exact values the C branches on.
    fn gen_num_str(r: &mut Rng) -> Vec<u8> {
        let core: Vec<u8> = match r.below(10) {
            0 => b"9223372036854775807".to_vec(),   // _I64_MAX exactly       -> no ERANGE
            1 => b"9223372036854775808".to_vec(),   // _I64_MAX + 1           -> saturate + ERANGE
            2 => b"-9223372036854775808".to_vec(),  // _I64_MIN exactly       -> no ERANGE
            3 => b"-9223372036854775809".to_vec(),  // _I64_MIN - 1           -> saturate + ERANGE
            4 => b"4294967295".to_vec(),            // ULONG_MAX              -> strtoul edge
            5 => b"4294967296".to_vec(),            // ULONG_MAX + 1          -> strtoul clamps
            6 => b"-4294967296".to_vec(),           // < -ULONG_MAX           -> strtoul returns *1*
            7 => b"2147483648".to_vec(),            // LONG_MAX + 1           -> strtol clamps
            8 => {                                  // a random LONG digit run
                let n = 15 + r.below(10);
                let mut v = Vec::new();
                if r.below(2) == 0 { v.push(b'-'); }
                for _ in 0..n { v.push(b'0' + r.below(10) as u8); }
                v
            }
            _ => {                                  // hex / octal / 0x-prefix paths
                let pre: &[u8] = [&b"0x"[..], &b"0X"[..], &b"0"[..], &b""[..]][r.below(4)];
                let n = 1 + r.below(18);
                let mut v = pre.to_vec();
                for _ in 0..n { v.push(b"0123456789abcdefABCDEF"[r.below(22)]); }
                v
            }
        };
        // wrap with the leading whitespace / sign / trailing-junk the C must skip
        let mut v = Vec::new();
        for _ in 0..r.below(3) { v.push(b" \t\n\x0b\x0c\r"[r.below(6)]); }
        v.extend_from_slice(&core);
        if r.below(3) == 0 { v.extend_from_slice(b"zz!"); }
        v.push(0);
        v
    }
    /// Half general-purpose strings, half boundary numerals.
    fn gen_parse_input(r: &mut Rng) -> Vec<u8> {
        if r.below(2) == 0 { gen_num_str(r) } else { gen_str(r, 22) }
    }

    check_effects("atoi", |r| {
        let a = gen_parse_input(r);
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| atoi(p) as i64), Effects::reference(|| ref_atoi(p) as i64),
         if null { "NULL".into() } else { format!("{a:?}") })
    });
    check_effects("atol", |r| {
        let a = gen_parse_input(r);
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| atol(p) as i64), Effects::reference(|| ref_atol(p) as i64),
         if null { "NULL".into() } else { format!("{a:?}") })
    });
    check_effects("_atoi64", |r| {
        let a = gen_parse_input(r);
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| _atoi64(p)), Effects::reference(|| ref__atoi64(p)),
         if null { "NULL".into() } else { format!("{a:?}") })
    });
    check_effects("strtol", |r| {
        let a = gen_parse_input(r);
        let b = BASES[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = Effects::subject(|| { let v = strtol(p, &mut e1, b); ((v as i64) << 8) ^ (e1 as i64 - p as i64) });
        let w = Effects::reference(|| { let v = ref_strtol(p, &mut e2, b); ((v as i64) << 8) ^ (e2 as i64 - p as i64) });
        (g, w, format!("base={b} {}", if null { "NULL".into() } else { format!("{a:?}") }))
    });
    check_effects("strtoul", |r| {
        let a = gen_parse_input(r);
        let b = BASES[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = Effects::subject(|| { let v = strtoul(p, &mut e1, b); ((v as i64) << 8) ^ (e1 as i64 - p as i64) });
        let w = Effects::reference(|| { let v = ref_strtoul(p, &mut e2, b); ((v as i64) << 8) ^ (e2 as i64 - p as i64) });
        (g, w, format!("base={b} {}", if null { "NULL".into() } else { format!("{a:?}") }))
    });

    // ---- mutators (compare destination bytes + returned pointer offset) ----
    check_buf("strcpy", |r| { let s = gen_str(r, 30);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        let p1 = strcpy(d1.as_mut_ptr(), s.as_ptr()); let p2 = ref_strcpy(d2.as_mut_ptr(), s.as_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:?}")) });
    check_buf("strncpy", |r| { let s = gen_str(r, 25); let n = r.below(40);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        let p1 = strncpy(d1.as_mut_ptr(), s.as_ptr(), n); let p2 = ref_strncpy(d2.as_mut_ptr(), s.as_ptr(), n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("n={n} {s:?}")) });
    check_buf("strcat", |r| { let a = gen_str(r, 12); let s = gen_str(r, 12);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        d1[..a.len()].copy_from_slice(&a); d2[..a.len()].copy_from_slice(&a);
        let p1 = strcat(d1.as_mut_ptr(), s.as_ptr()); let p2 = ref_strcat(d2.as_mut_ptr(), s.as_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{a:?} + {s:?}")) });
    check_buf("strncat", |r| { let a = gen_str(r, 12); let s = gen_str(r, 12); let n = r.below(16);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        d1[..a.len()].copy_from_slice(&a); d2[..a.len()].copy_from_slice(&a);
        let p1 = strncat(d1.as_mut_ptr(), s.as_ptr(), n); let p2 = ref_strncat(d2.as_mut_ptr(), s.as_ptr(), n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("n={n}")) });
    check_buf("_strlwr", |r| { let s = gen_str(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _strlwr(d1.as_mut_ptr()); let p2 = ref__strlwr(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:?}")) });
    check_buf("_strupr", |r| { let s = gen_str(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _strupr(d1.as_mut_ptr()); let p2 = ref__strupr(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:?}")) });
    check_buf("_strrev", |r| { let s = gen_str(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _strrev(d1.as_mut_ptr()); let p2 = ref__strrev(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:?}")) });
    check_buf("_strset", |r| { let s = gen_str(r, 30); let c = (r.below(300) as i32) - 20;
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _strset(d1.as_mut_ptr(), c); let p2 = ref__strset(d2.as_mut_ptr(), c);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("c={c}")) });
    check_buf("_strnset", |r| { let s = gen_str(r, 30); let c = (r.below(300) as i32) - 20; let n = r.below(34);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _strnset(d1.as_mut_ptr(), c, n); let p2 = ref__strnset(d2.as_mut_ptr(), c, n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("c={c} n={n}")) });
    check_buf("memcpy", |r| { let s = gen_str(r, 30); let n = r.below(s.len() + 1);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        let p1 = memcpy(d1.as_mut_ptr().cast(), s.as_ptr().cast(), n); let p2 = ref_memcpy(d2.as_mut_ptr().cast(), s.as_ptr().cast(), n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("n={n}")) });
    // memcpy with OVERLAPPING src/dst: cnt-rust (libcntpr) is documented
    // overlap-safe; C says UB. Grade it explicitly rather than assume.
    check_buf("memcpy/overlap", |r| { let s = gen_str(r, 20); let n = r.below(s.len() + 1);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        d1[..s.len()].copy_from_slice(&s); d2[..s.len()].copy_from_slice(&s);
        let sh = 1 + r.below(4); // dst overlaps src, shifted forward
        let p1 = memcpy(d1.as_mut_ptr().add(sh).cast(), d1.as_ptr().cast(), n);
        let p2 = ref_memcpy(d2.as_mut_ptr().add(sh).cast(), d2.as_ptr().cast(), n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("n={n} shift={sh}")) });
    check_buf("memset", |r| { let n = r.below(40); let c = (r.below(300) as i32) - 20;
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        let p1 = memset(d1.as_mut_ptr().cast(), c, n); let p2 = ref_memset(d2.as_mut_ptr().cast(), c, n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("c={c} n={n}")) });
    check_buf("_memccpy", |r| { let s = gen_str(r, 30); let c = (r.below(300) as i32) - 20; let n = r.below(s.len() + 1);
        let (mut d1, mut d2) = (vec![0xAAu8; 48], vec![0xAAu8; 48]);
        let p1 = _memccpy(d1.as_mut_ptr().cast(), s.as_ptr().cast(), c, n); let p2 = ref__memccpy(d2.as_mut_ptr().cast(), s.as_ptr().cast(), c, n);
        let (o1, o2) = (if p1.is_null() { -1 } else { p1 as isize - d1.as_ptr() as isize },
                        if p2.is_null() { -1 } else { p2 as isize - d2.as_ptr() as isize });
        (d1, o1, d2, o2, format!("c={c} n={n}")) });

    // ─────────── WAVE 4: the wide family (vs the wcs.c reference TU) ───────────
    // Offsets are in ELEMENTS (both sides use the same formula, so the compare is
    // valid either way; /2 just makes a divergence report readable).
    macro_rules! woff { ($p:expr, $base:expr) => {
        if $p.is_null() { -1i64 } else { (($p as usize - $base.as_ptr() as usize) / 2) as i64 }
    } }

    check_int("wcslen", false, |r| { let a = gen_wstr(r, 40);
        (wcslen(a.as_ptr()) as i64, ref_wcslen(a.as_ptr()) as i64, format!("{a:04x?}")) });
    check_int("wcscmp", false, |r| { let a = gen_wstr(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_wstr(r, 20) };
        (wcscmp(a.as_ptr(), b.as_ptr()) as i64, ref_wcscmp(a.as_ptr(), b.as_ptr()) as i64, format!("{a:04x?} vs {b:04x?}")) });
    check_int("wcsncmp", false, |r| { let a = gen_wstr(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_wstr(r, 20) }; let n = r.below(22);
        (wcsncmp(a.as_ptr(), b.as_ptr(), n) as i64, ref_wcsncmp(a.as_ptr(), b.as_ptr(), n) as i64, format!("n={n} {a:04x?} vs {b:04x?}")) });
    check_int("wcschr", false, |r| { let a = gen_wstr(r, 30); let c = r.below(0x10000) as u16;
        (woff!(wcschr(a.as_ptr(), c), a), woff!(ref_wcschr(a.as_ptr(), c), a), format!("c={c:04x} {a:04x?}")) });
    check_int("wcsrchr", false, |r| { let a = gen_wstr(r, 30); let c = r.below(0x10000) as u16;
        (woff!(wcsrchr(a.as_ptr(), c), a), woff!(ref_wcsrchr(a.as_ptr(), c), a), format!("c={c:04x} {a:04x?}")) });
    check_int("wcsstr", false, |r| { let h = gen_wstr(r, 30);
        let n = if r.below(2) == 0 && h.len() > 2 { let s = r.below(h.len() - 1); let e = s + r.below(h.len() - s); let mut v = h[s..e].to_vec(); v.push(0); v } else { gen_wstr(r, 6) };
        (woff!(wcsstr(h.as_ptr(), n.as_ptr()), h), woff!(ref_wcsstr(h.as_ptr(), n.as_ptr()), h), format!("{h:04x?} / {n:04x?}")) });
    check_int("wcsspn", false, |r| { let a = gen_wstr(r, 25); let b = gen_wstr(r, 8);
        (wcsspn(a.as_ptr(), b.as_ptr()) as i64, ref_wcsspn(a.as_ptr(), b.as_ptr()) as i64, format!("{a:04x?} / {b:04x?}")) });
    check_int("wcscspn", false, |r| { let a = gen_wstr(r, 25); let b = gen_wstr(r, 8);
        (wcscspn(a.as_ptr(), b.as_ptr()) as i64, ref_wcscspn(a.as_ptr(), b.as_ptr()) as i64, format!("{a:04x?} / {b:04x?}")) });
    check_int("wcspbrk", false, |r| { let a = gen_wstr(r, 25); let b = gen_wstr(r, 8);
        (woff!(wcspbrk(a.as_ptr(), b.as_ptr()), a), woff!(ref_wcspbrk(a.as_ptr(), b.as_ptr()), a), format!("{a:04x?} / {b:04x?}")) });

    check_wbuf("wcscpy", |r| { let s = gen_wstr(r, 30);
        let (mut d1, mut d2) = (vec![0xAAAAu16; 48], vec![0xAAAAu16; 48]);
        let p1 = wcscpy(d1.as_mut_ptr(), s.as_ptr()); let p2 = ref_wcscpy(d2.as_mut_ptr(), s.as_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:04x?}")) });
    check_wbuf("wcscat", |r| { let a = gen_wstr(r, 12); let s = gen_wstr(r, 12);
        let (mut d1, mut d2) = (vec![0xAAAAu16; 48], vec![0xAAAAu16; 48]);
        d1[..a.len()].copy_from_slice(&a); d2[..a.len()].copy_from_slice(&a);
        let p1 = wcscat(d1.as_mut_ptr(), s.as_ptr()); let p2 = ref_wcscat(d2.as_mut_ptr(), s.as_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{a:04x?} + {s:04x?}")) });
    check_wbuf("_wcsrev", |r| { let s = gen_wstr(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _wcsrev(d1.as_mut_ptr()); let p2 = ref__wcsrev(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("{s:04x?}")) });
    check_wbuf("_wcsset", |r| { let s = gen_wstr(r, 30); let c = r.below(0x10000) as u16;
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _wcsset(d1.as_mut_ptr(), c); let p2 = ref__wcsset(d2.as_mut_ptr(), c);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("c={c:04x}")) });
    // NB: `_wcsnset` returns the START (msvcrt: `wchar_t *ret = str;`), unlike its
    // narrow twin `_strnset`, which returns the ADVANCED pointer. check_wbuf
    // compares the returned offset, so this asymmetry is graded, not assumed.
    // ───────── WAVE 5b: the `_s` secure family ─────────
    //
    // Grades ALL FOUR observable effects, not just the destination buffer:
    //   1. the bytes        2. the RETURN CODE (0/EINVAL/ERANGE/STRUNCATE)
    //   3. ERRNO            4. did the invalid-parameter handler FIRE
    //
    // errno is stored SEPARATELY per side on purpose (subjectimpl.c vs refimpl.c),
    // so "did the SUBJECT set errno?" is answerable at all. An errno contract you
    // cannot observe independently is a contract you are not testing.
    //
    // `check_buf` compares (bytes, ret-offset); we fold ret+errno into that offset
    // so a divergence in EITHER lights up, and print both raw values in the note.
    //
    // Inputs deliberately span: elem too-small / exact / roomy, count == 0, and
    // count == _TRUNCATE (usize::MAX) — the three branches that carry the quirks.
    check_buf("strcpy_s", |r| {
        let src = gen_str(r, 20);
        let elem = r.below(24);
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        subj_errno_set(0); ref_errno_set(0);
        let r1 = strcpy_s(d1.as_mut_ptr(), elem, src.as_ptr());
        let e1 = subj_errno_get();
        let r2 = ref_strcpy_s(d2.as_mut_ptr(), elem, src.as_ptr());
        let e2 = ref_errno_get();
        (d1, (r1 as isize) * 1000 + e1 as isize, d2, (r2 as isize) * 1000 + e2 as isize,
         format!("elem={elem} src={src:?} ret {r1}/{r2} errno {e1}/{e2}"))
    });
    check_buf("strcat_s", |r| {
        let src = gen_str(r, 14);
        let pre = gen_str(r, 10);
        let elem = r.below(28);
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        d1[..pre.len()].copy_from_slice(&pre); d2[..pre.len()].copy_from_slice(&pre);
        subj_errno_set(0); ref_errno_set(0);
        let r1 = strcat_s(d1.as_mut_ptr(), elem, src.as_ptr());
        let e1 = subj_errno_get();
        let r2 = ref_strcat_s(d2.as_mut_ptr(), elem, src.as_ptr());
        let e2 = ref_errno_get();
        (d1, (r1 as isize) * 1000 + e1 as isize, d2, (r2 as isize) * 1000 + e2 as isize,
         format!("elem={elem} pre={pre:?} src={src:?} ret {r1}/{r2} errno {e1}/{e2}"))
    });
    check_buf("strncpy_s", |r| {
        let src = gen_str(r, 20);
        let elem = r.below(24);
        let count = match r.below(5) { 0 => usize::MAX, 1 => 0, _ => r.below(24) };
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        subj_errno_set(0); ref_errno_set(0);
        let r1 = strncpy_s(d1.as_mut_ptr(), elem, src.as_ptr(), count);
        let e1 = subj_errno_get();
        let r2 = ref_strncpy_s(d2.as_mut_ptr(), elem, src.as_ptr(), count);
        let e2 = ref_errno_get();
        let cs = if count == usize::MAX { "_TRUNCATE".to_string() } else { count.to_string() };
        (d1, (r1 as isize) * 1000 + e1 as isize, d2, (r2 as isize) * 1000 + e2 as isize,
         format!("elem={elem} count={cs} src={src:?} ret {r1}/{r2} errno {e1}/{e2}"))
    });
    check_buf("strncat_s", |r| {
        let src = gen_str(r, 14);
        let pre = gen_str(r, 10);
        let elem = r.below(28);
        let count = match r.below(5) { 0 => usize::MAX, 1 => 0, _ => r.below(20) };
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        d1[..pre.len()].copy_from_slice(&pre); d2[..pre.len()].copy_from_slice(&pre);
        subj_errno_set(0); ref_errno_set(0);
        let r1 = strncat_s(d1.as_mut_ptr(), elem, src.as_ptr(), count);
        let e1 = subj_errno_get();
        let r2 = ref_strncat_s(d2.as_mut_ptr(), elem, src.as_ptr(), count);
        let e2 = ref_errno_get();
        let cs = if count == usize::MAX { "_TRUNCATE".to_string() } else { count.to_string() };
        (d1, (r1 as isize) * 1000 + e1 as isize, d2, (r2 as isize) * 1000 + e2 as isize,
         format!("elem={elem} count={cs} pre={pre:?} src={src:?} ret {r1}/{r2} errno {e1}/{e2}"))
    });
    check_buf("_strnset_s", |r| {
        let pre = gen_str(r, 20);
        let size = r.below(26);
        let count = r.below(26);
        let c = (r.below(300) as i32) - 20;
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        d1[..pre.len()].copy_from_slice(&pre); d2[..pre.len()].copy_from_slice(&pre);
        subj_errno_set(0); ref_errno_set(0);
        let r1 = _strnset_s(d1.as_mut_ptr(), size, c, count);
        let e1 = subj_errno_get();
        let r2 = ref__strnset_s(d2.as_mut_ptr(), size, c, count);
        let e2 = ref_errno_get();
        (d1, (r1 as isize) * 1000 + e1 as isize, d2, (r2 as isize) * 1000 + e2 as isize,
         format!("size={size} count={count} c={c} pre={pre:?} ret {r1}/{r2} errno {e1}/{e2}"))
    });

    // ───────── WAVE 5a: the plumbing-free leftovers ─────────
    check_int("__strncnt", false, |r| { let a = gen_str(r, 40); let n = r.below(45);
        (__strncnt(a.as_ptr(), n) as i64, ref___strncnt(a.as_ptr(), n) as i64, format!("n={n} {a:?}")) });
    check_buf("_swab", |r| {
        // odd `len` on purpose: msvcrt drops the trailing byte (len >>= 1).
        let s = gen_str(r, 24);
        let len = r.below(s.len() + 3) as i32 - 1;   // also exercises len <= 1 (no-op)
        let (mut d1, mut d2) = (vec![0xAAu8; 40], vec![0xAAu8; 40]);
        _swab(s.as_ptr(), d1.as_mut_ptr(), len);
        ref__swab(s.as_ptr(), d2.as_mut_ptr(), len);
        (d1, 0, d2, 0, format!("len={len}"))
    });

    // ───────── WAVE 5 PROBE: cnt-rust's float parser vs msvcrt's bignum ─────────
    // NOT a flip. `strtod_probe` is a plain `pub unsafe fn` (no #[no_mangle]), so
    // msvcrt.dll still uses its C strtod. This only asks: COULD we flip it?
    //
    // Graded on the BIT PATTERN, not `==`: two doubles that differ by one ULP are
    // a real divergence for a CRT, and `-0.0 == 0.0` would hide a sign bug.
    check_int("strtod [PROBE]", false, |r| {
        let s = gen_float_str(r);
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let got = msvcrt_crt_rust::strtod_probe(s.as_ptr(), &mut e1);
        let want = ref_strtod(s.as_ptr(), &mut e2);
        let (gb, wb) = (got.to_bits() as i64, want.to_bits() as i64);
        // also compare the end pointer (the `*sret` contract)
        let (go, wo) = (e1 as usize - s.as_ptr() as usize, e2 as usize - s.as_ptr() as usize);
        let ctx = format!("{:?} -> rust={got:?} (0x{:016x}) msvcrt={want:?} (0x{:016x}) end {go} vs {wo}",
                          String::from_utf8_lossy(&s[..s.len()-1]), got.to_bits(), want.to_bits());
        if go != wo { return (go as i64, wo as i64, format!("END-PTR differs: {ctx}")); }
        (gb, wb, ctx)
    });

    check_wbuf("_wcsnset", |r| { let s = gen_wstr(r, 30); let c = r.below(0x10000) as u16; let n = r.below(34);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _wcsnset(d1.as_mut_ptr(), c, n); let p2 = ref__wcsnset(d2.as_mut_ptr(), c, n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("c={c:04x} n={n}")) });

    // ═════ WAVE 6b: the radix `_s` family ═════════════════════════════════════
    //
    // Ten functions, FOUR contracts (see msvcrt-crt-rust's wave-6b banner). The
    // fuzz has to actually REACH each contract branch, so `size` is drawn small
    // enough to be too small most of the time, `radix` includes out-of-range
    // values, and NULL is drawn deliberately:
    //
    //   size  : 0..12  vs values needing up to 65 digits (base 2) -> ERANGE is COMMON
    //   radix : 2..36 valid, plus 0/1/37/-1 invalid               -> EINVAL front door
    //   str   : NULL 1-in-32                                      -> EINVAL front door
    //
    // and the FULL buffer is compared, which is what catches the reversed residue.
    fn radix_of(r: &mut Rng) -> i32 {
        match r.below(8) {
            0 => 0, 1 => 1, 2 => 37, 3 => -1,      // the invalid ones
            _ => 2 + r.below(35) as i32,           // 2..=36
        }
    }
    // Skew hard to the extremes: i32::MIN/i64::MIN are where the
    // `value < 0 && radix == 10` sign rule and the unsigned reinterpret collide.
    fn v32(r: &mut Rng) -> i32 { match r.below(6) { 0 => i32::MIN, 1 => i32::MAX, 2 => 0, 3 => -1, _ => r.next() as i32 } }
    fn v64(r: &mut Rng) -> i64 { match r.below(6) { 0 => i64::MIN, 1 => i64::MAX, 2 => 0, 3 => -1, _ => r.next() as i64 } }
    fn vu64(r: &mut Rng) -> u64 { match r.below(5) { 0 => 0, 1 => u64::MAX, 2 => u32::MAX as u64, _ => r.next() } }
    fn vu32(r: &mut Rng) -> u32 { match r.below(5) { 0 => 0, 1 => u32::MAX, 2 => 1, _ => r.next() as u32 } }

    macro_rules! radix_s_n {
        ($name:literal, $subj:ident, $refr:ident, $gen:ident, $req:expr) => {
            check_buf_fx($name, $req, |r| {
                let v = $gen(r);
                let rx = radix_of(r);
                let sz = r.below(13);
                let null = r.below(32) == 0;
                let (mut d1, mut d2) = (vec![0xAAu8; 72], vec![0xAAu8; 72]);
                let (p1, p2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) }
                               else { (d1.as_mut_ptr(), d2.as_mut_ptr()) };
                let g = Effects::subject(|| $subj(v, p1, sz, rx) as i64);
                let w = Effects::reference(|| $refr(v, p2, sz, rx) as i64);
                (d1, g, d2, w, format!("v={v} radix={rx} size={sz}{}", if null { " str=NULL" } else { "" }))
            });
        };
    }
    macro_rules! radix_s_w {
        ($name:literal, $subj:ident, $refr:ident, $gen:ident, $req:expr) => {
            check_wbuf_fx($name, $req, |r| {
                let v = $gen(r);
                let rx = radix_of(r);
                let sz = r.below(13);
                let null = r.below(32) == 0;
                let (mut d1, mut d2) = (vec![0xAAAAu16; 72], vec![0xAAAAu16; 72]);
                let (p1, p2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) }
                               else { (d1.as_mut_ptr(), d2.as_mut_ptr()) };
                let g = Effects::subject(|| $subj(v, p1, sz, rx) as i64);
                let w = Effects::reference(|| $refr(v, p2, sz, rx) as i64);
                (d1, g, d2, w, format!("v={v} radix={rx} size={sz}{}", if null { " str=NULL" } else { "" }))
            });
        };
    }

    radix_s_n!("_itoa_s",    _itoa_s,    ref__itoa_s,    v32, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_n!("_ltoa_s",    _ltoa_s,    ref__ltoa_s,    v32, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_n!("_i64toa_s",  _i64toa_s,  ref__i64toa_s,  v64, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_n!("_ui64toa_s", _ui64toa_s, ref__ui64toa_s, vu64, &["ok", "EINVAL", "handler"]);
    radix_s_n!("_ultoa_s",   _ultoa_s,   ref__ultoa_s,   vu32, &["ok", "EINVAL", "ERANGE"]);
    radix_s_w!("_itow_s",    _itow_s,    ref__itow_s,    v32, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_w!("_ltow_s",    _ltow_s,    ref__ltow_s,    v32, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_w!("_i64tow_s",  _i64tow_s,  ref__i64tow_s,  v64, &["ok", "EINVAL", "ERANGE", "handler"]);
    radix_s_w!("_ui64tow_s", _ui64tow_s, ref__ui64tow_s, vu64, &["ok", "EINVAL", "handler"]);
    radix_s_w!("_ultow_s",   _ultow_s,   ref__ultow_s,   vu32, &["ok", "EINVAL", "ERANGE"]);

    // _strtoi64 — the engine strtol/_atoi64 sit on, now exported directly.
    check_effects("_strtoi64", |r| {
        let a = gen_parse_input(r);
        let b = BASES2[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = Effects::subject(|| { let v = _strtoi64(p, &mut e1, b); v ^ (e1 as i64 - p as i64) });
        let w = Effects::reference(|| { let v = ref__strtoi64(p, &mut e2, b); v ^ (e2 as i64 - p as i64) });
        (g, w, format!("base={b} {}", if null { "NULL".into() } else { format!("{a:?}") }))
    });


    // ═════ WAVE 6c: the wide `_s` family, tokenizers, memmove, _strtoui64 ═════
    //
    // The four wide `_s` writers each have a DIFFERENT handler contract (see the
    // msvcrt-crt-rust banner), so each row's `require` list encodes it — and a
    // wrong list fails loudly as UNEXERCISED rather than passing quietly:
    //
    //   wcscpy_s   handler on every error          -> require handler
    //   wcsncpy_s  handler on the front door only  -> require ERANGE *and* handler
    //   wcscat_s   handler NEVER                   -> require ok/EINVAL/ERANGE, NOT handler
    //   wcsncat_s  handler on everything           -> require handler
    check_wbuf_fx("wcscpy_s", &["ok", "EINVAL", "ERANGE", "handler"], |r| {
        let src = gen_wstr(r, 20);
        let elem = r.below(24);
        let null_src = r.below(32) == 0;
        let sp = if null_src { core::ptr::null() } else { src.as_ptr() };
        let (mut d1, mut d2) = (vec![0xAAAAu16; 40], vec![0xAAAAu16; 40]);
        let g = Effects::subject(|| wcscpy_s(d1.as_mut_ptr(), elem, sp) as i64);
        let w = Effects::reference(|| ref_wcscpy_s(d2.as_mut_ptr(), elem, sp) as i64);
        (d1, g, d2, w, format!("elem={elem} srclen={}{}", src.len()-1, if null_src { " src=NULL" } else { "" }))
    });
    check_wbuf_fx("wcsncpy_s", &["ok", "EINVAL", "ERANGE"], |r| {
        let src = gen_wstr(r, 20);
        let elem = r.below(24);
        let count = match r.below(5) { 0 => usize::MAX, 1 => 0, _ => r.below(24) };
        let (mut d1, mut d2) = (vec![0xAAAAu16; 40], vec![0xAAAAu16; 40]);
        let g = Effects::subject(|| wcsncpy_s(d1.as_mut_ptr(), elem, src.as_ptr(), count) as i64);
        let w = Effects::reference(|| ref_wcsncpy_s(d2.as_mut_ptr(), elem, src.as_ptr(), count) as i64);
        let cs = if count == usize::MAX { "_TRUNCATE".into() } else { count.to_string() };
        (d1, g, d2, w, format!("elem={elem} count={cs} srclen={}", src.len()-1))
    });
    check_wbuf_fx("wcscat_s", &["ok", "EINVAL", "ERANGE"], |r| {
        let src = gen_wstr(r, 14);
        let pre = gen_wstr(r, 10);
        let elem = r.below(28);
        let (mut d1, mut d2) = (vec![0xAAAAu16; 40], vec![0xAAAAu16; 40]);
        d1[..pre.len()].copy_from_slice(&pre); d2[..pre.len()].copy_from_slice(&pre);
        let g = Effects::subject(|| wcscat_s(d1.as_mut_ptr(), elem, src.as_ptr()) as i64);
        let w = Effects::reference(|| ref_wcscat_s(d2.as_mut_ptr(), elem, src.as_ptr()) as i64);
        (d1, g, d2, w, format!("elem={elem} prelen={} srclen={}", pre.len()-1, src.len()-1))
    });
    check_wbuf_fx("wcsncat_s", &["ok", "EINVAL", "ERANGE", "handler"], |r| {
        let src = gen_wstr(r, 14);
        let pre = gen_wstr(r, 10);
        let elem = r.below(28);
        let count = match r.below(5) { 0 => usize::MAX, 1 => 0, _ => r.below(20) };
        let (mut d1, mut d2) = (vec![0xAAAAu16; 40], vec![0xAAAAu16; 40]);
        d1[..pre.len()].copy_from_slice(&pre); d2[..pre.len()].copy_from_slice(&pre);
        let g = Effects::subject(|| wcsncat_s(d1.as_mut_ptr(), elem, src.as_ptr(), count) as i64);
        let w = Effects::reference(|| ref_wcsncat_s(d2.as_mut_ptr(), elem, src.as_ptr(), count) as i64);
        let cs = if count == usize::MAX { "_TRUNCATE".into() } else { count.to_string() };
        (d1, g, d2, w, format!("elem={elem} count={cs} prelen={} srclen={}", pre.len()-1, src.len()-1))
    });
    check_wbuf_fx("_wcsnset_s", &["ok", "EINVAL", "handler"], |r| {
        let s = gen_wstr(r, 24);
        let size = r.below(28);
        let count = r.below(28);
        let c = 0x41 + r.below(26) as u16;
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcsnset_s(d1.as_mut_ptr(), size, c, count) as i64);
        let w = Effects::reference(|| ref__wcsnset_s(d2.as_mut_ptr(), size, c, count) as i64);
        (d1, g, d2, w, format!("size={size} count={count} c={c:04x} len={}", s.len()-1))
    });
    check_wbuf_fx("_wcsset_s", &["ok", "EINVAL", "handler"], |r| {
        let s = gen_wstr(r, 24);
        let n = r.below(28);
        let c = 0x41 + r.below(26) as u16;
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcsset_s(d1.as_mut_ptr(), n, c) as i64);
        let w = Effects::reference(|| ref__wcsset_s(d2.as_mut_ptr(), n, c) as i64);
        (d1, g, d2, w, format!("n={n} c={c:04x} len={}", s.len()-1))
    });

    check_wbuf("wcsncpy", |r| {
        let s = gen_wstr(r, 24); let n = r.below(30);
        let (mut d1, mut d2) = (vec![0xAAAAu16; 40], vec![0xAAAAu16; 40]);
        let p1 = wcsncpy(d1.as_mut_ptr(), s.as_ptr(), n);
        let p2 = ref_wcsncpy(d2.as_mut_ptr(), s.as_ptr(), n);
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("n={n} srclen={}", s.len()-1))
    });
    check_int("wcsnlen", false, |r| {
        let s = gen_wstr(r, 30); let m = r.below(34);
        (wcsnlen(s.as_ptr(), m) as i64, ref_wcsnlen(s.as_ptr(), m) as i64, format!("m={m}"))
    });

    // Tokenizers: drive a FULL tokenisation loop, not one call — the contract is
    // the sequence of tokens and the final ctx, not any single return.
    // strtok / wcstok — the THREAD-DATA tokenizers (wave 10).
    //
    // Same drive-both-sides-with-one-sequence shape as strtok_s, except the cursor is
    // not a caller-supplied ctx: each side keeps it in its OWN `thread_data_t`
    // (refimpl's is namespaced to msvcrt_ref_get_thread_data; subjectimpl has its own),
    // reached through msvcrt-sys's `offset_of!`. ref_integrity proves the two structs
    // agree on where `strtok_next` lives, which is what makes this comparison mean
    // anything at all.
    //
    // NOTE the contract these rows are really defending: `strtok` is NOT
    // `strtok_s(.., &thread->strtok_next)`. It has no CHECK_PMT guards, so a
    // `strtok(NULL, d)` on a fresh cursor returns NULL SILENTLY where strtok_s would
    // fire the invalid-parameter handler. The Effects comparison (which grades the
    // handler) is what would catch a lazy delegation.
    check_buf("strtok", |r| {
        let mut s1 = gen_str(r, 26); let delim = gen_str(r, 4);
        let mut s2 = s1.clone();
        let (mut out1, mut out2) = (Vec::new(), Vec::new());
        let mut p1 = strtok(s1.as_mut_ptr(), delim.as_ptr());
        let mut p2 = ref_strtok(s2.as_mut_ptr(), delim.as_ptr());
        let mut guard = 0;
        while (!p1.is_null() || !p2.is_null()) && guard < 40 {
            out1.push(if p1.is_null() { -1 } else { p1 as isize - s1.as_ptr() as isize } as i64);
            out2.push(if p2.is_null() { -1 } else { p2 as isize - s2.as_ptr() as isize } as i64);
            p1 = strtok(core::ptr::null_mut(), delim.as_ptr());
            p2 = ref_strtok(core::ptr::null_mut(), delim.as_ptr());
            guard += 1;
        }
        let b1: Vec<u8> = s1.iter().copied().chain(out1.iter().map(|v| *v as u8)).collect();
        let b2: Vec<u8> = s2.iter().copied().chain(out2.iter().map(|v| *v as u8)).collect();
        (b1, out1.len() as isize, b2, out2.len() as isize, format!("delim={delim:?} tokens={out1:?}/{out2:?}"))
    });
    check_wbuf("wcstok", |r| {
        let mut s1 = gen_wstr(r, 26); let delim = gen_wstr(r, 4);
        let mut s2 = s1.clone();
        let (mut out1, mut out2) = (Vec::new(), Vec::new());
        let mut p1 = wcstok(s1.as_mut_ptr(), delim.as_ptr());
        let mut p2 = ref_wcstok(s2.as_mut_ptr(), delim.as_ptr());
        let mut guard = 0;
        while (!p1.is_null() || !p2.is_null()) && guard < 40 {
            out1.push(if p1.is_null() { -1i64 } else { ((p1 as isize - s1.as_ptr() as isize) / 2) as i64 });
            out2.push(if p2.is_null() { -1i64 } else { ((p2 as isize - s2.as_ptr() as isize) / 2) as i64 });
            p1 = wcstok(core::ptr::null_mut(), delim.as_ptr());
            p2 = ref_wcstok(core::ptr::null_mut(), delim.as_ptr());
            guard += 1;
        }
        let b1: Vec<u16> = s1.iter().copied().chain(out1.iter().map(|v| *v as u16)).collect();
        let b2: Vec<u16> = s2.iter().copied().chain(out2.iter().map(|v| *v as u16)).collect();
        (b1, out1.len() as isize, b2, out2.len() as isize, format!("tokens={out1:?}/{out2:?}"))
    });
    check_buf("strtok_s", |r| {
        let mut s1 = gen_str(r, 26); let delim = gen_str(r, 4);
        let mut s2 = s1.clone();
        let (mut c1, mut c2) = (core::ptr::null_mut::<u8>(), core::ptr::null_mut::<u8>());
        let (mut out1, mut out2) = (Vec::new(), Vec::new());
        let mut p1 = strtok_s(s1.as_mut_ptr(), delim.as_ptr(), &mut c1);
        let mut p2 = ref_strtok_s(s2.as_mut_ptr(), delim.as_ptr(), &mut c2);
        let mut guard = 0;
        while (!p1.is_null() || !p2.is_null()) && guard < 40 {
            out1.push(if p1.is_null() { -1 } else { p1 as isize - s1.as_ptr() as isize } as i64);
            out2.push(if p2.is_null() { -1 } else { p2 as isize - s2.as_ptr() as isize } as i64);
            p1 = strtok_s(core::ptr::null_mut(), delim.as_ptr(), &mut c1);
            p2 = ref_strtok_s(core::ptr::null_mut(), delim.as_ptr(), &mut c2);
            guard += 1;
        }
        // fold the token-offset sequence into the compared buffer
        let b1: Vec<u8> = s1.iter().copied().chain(out1.iter().map(|v| *v as u8)).collect();
        let b2: Vec<u8> = s2.iter().copied().chain(out2.iter().map(|v| *v as u8)).collect();
        (b1, out1.len() as isize, b2, out2.len() as isize, format!("delim={delim:?} tokens={out1:?}/{out2:?}"))
    });
    check_wbuf("wcstok_s", |r| {
        let mut s1 = gen_wstr(r, 26); let delim = gen_wstr(r, 4);
        let mut s2 = s1.clone();
        let (mut c1, mut c2) = (core::ptr::null_mut::<u16>(), core::ptr::null_mut::<u16>());
        let (mut out1, mut out2) = (Vec::new(), Vec::new());
        let mut p1 = wcstok_s(s1.as_mut_ptr(), delim.as_ptr(), &mut c1);
        let mut p2 = ref_wcstok_s(s2.as_mut_ptr(), delim.as_ptr(), &mut c2);
        let mut guard = 0;
        while (!p1.is_null() || !p2.is_null()) && guard < 40 {
            out1.push(if p1.is_null() { -1i64 } else { ((p1 as isize - s1.as_ptr() as isize) / 2) as i64 });
            out2.push(if p2.is_null() { -1i64 } else { ((p2 as isize - s2.as_ptr() as isize) / 2) as i64 });
            p1 = wcstok_s(core::ptr::null_mut(), delim.as_ptr(), &mut c1);
            p2 = ref_wcstok_s(core::ptr::null_mut(), delim.as_ptr(), &mut c2);
            guard += 1;
        }
        let b1: Vec<u16> = s1.iter().copied().chain(out1.iter().map(|v| *v as u16)).collect();
        let b2: Vec<u16> = s2.iter().copied().chain(out2.iter().map(|v| *v as u16)).collect();
        (b1, out1.len() as isize, b2, out2.len() as isize, format!("tokens={out1:?}/{out2:?}"))
    });

    check_effects("_strtoui64", |r| {
        let a = gen_parse_input(r);
        let b = BASES2[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = Effects::subject(|| { let v = _strtoui64(p, &mut e1, b); (v as i64) ^ (e1 as i64 - p as i64) });
        let w = Effects::reference(|| { let v = ref__strtoui64(p, &mut e2, b); (v as i64) ^ (e2 as i64 - p as i64) });
        (g, w, format!("base={b} {}", if null { "NULL".into() } else { format!("{a:?}") }))
    });


    // ═════ WAVE 7: the case-insensitive family ════════════════════════════════
    //
    // gen_str emits 0x80-0xFF, which is the whole point here: `char` is SIGNED,
    // and the two comparison functions treat the high half DIFFERENTLY.
    //   _strnicmp folds on the sign-extended int, returns (unsigned char) c1-c2
    //   _memicmp  folds via _tolower_l (non-ASCII passes THROUGH), returns c1-c2 RAW
    // If they were the same comparison with a different bound, one of these rows
    // would be redundant. They are not, so both are graded, and NULL is fuzzed
    // (each has a front door, and they disagree about it too).
    check_effects("_stricmp", |r| {
        let (a, b) = (gen_str(r, 20), gen_str(r, 20));
        let na = r.below(32) == 0;
        let (pa, pb) = (if na { core::ptr::null() } else { a.as_ptr() }, b.as_ptr());
        (Effects::subject(|| _stricmp(pa, pb) as i64),
         Effects::reference(|| ref__stricmp(pa, pb) as i64),
         format!("{}{:?} vs {:?}", if na { "NULL " } else { "" }, a, b))
    });
    check_effects("_strnicmp", |r| {
        let (a, b) = (gen_str(r, 20), gen_str(r, 20));
        let n = r.below(24);
        let na = r.below(32) == 0;
        let (pa, pb) = (if na { core::ptr::null() } else { a.as_ptr() }, b.as_ptr());
        (Effects::subject(|| _strnicmp(pa, pb, n) as i64),
         Effects::reference(|| ref__strnicmp(pa, pb, n) as i64),
         format!("n={n} {}{:?} vs {:?}", if na { "NULL " } else { "" }, a, b))
    });
    check_effects("_memicmp", |r| {
        let (a, b) = (gen_str(r, 20), gen_str(r, 20));
        let n = r.below(a.len().min(b.len()).max(1));
        let na = r.below(32) == 0;
        let pa = if na { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| _memicmp(pa.cast(), b.as_ptr().cast(), n) as i64),
         Effects::reference(|| ref__memicmp(pa.cast(), b.as_ptr().cast(), n) as i64),
         format!("n={n} {}{:?} vs {:?}", if na { "NULL " } else { "" }, a, b))
    });
    // _strlwr_s / _strupr_s: hand-rolled errno, handler must NEVER fire.
    check_buf_fx("_strlwr_s", &["ok", "EINVAL"], |r| {
        let s = gen_str(r, 24);
        let len = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAA); d2.resize(40, 0xAA);
        let g = Effects::subject(|| _strlwr_s(d1.as_mut_ptr(), len) as i64);
        let w = Effects::reference(|| ref__strlwr_s(d2.as_mut_ptr(), len) as i64);
        (d1, g, d2, w, format!("len={len} slen={}", s.len()-1))
    });
    check_buf_fx("_strupr_s", &["ok", "EINVAL"], |r| {
        let s = gen_str(r, 24);
        let len = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAA); d2.resize(40, 0xAA);
        let g = Effects::subject(|| _strupr_s(d1.as_mut_ptr(), len) as i64);
        let w = Effects::reference(|| ref__strupr_s(d2.as_mut_ptr(), len) as i64);
        (d1, g, d2, w, format!("len={len} slen={}", s.len()-1))
    });


    // ═════ WAVE 8: the WIDE case-insensitive family ═══════════════════════════
    //
    // Deliberately NOT assumed to mirror wave 7. Three things differ and all three
    // are graded here:
    //   * the comparers return the RAW u16 diff (wcscmp CLAMPS — wave 4)
    //   * the folders MUTATE BEFORE FAILING, so the full buffer is compared
    //   * their front door zeroes *str where the narrow one does not
    check_effects("_wcsicmp", |r| {
        let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
        let na = r.below(32) == 0;
        let pa = if na { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| _wcsicmp(pa, b.as_ptr()) as i64),
         Effects::reference(|| ref__wcsicmp(pa, b.as_ptr()) as i64),
         format!("{}alen={} blen={}", if na { "NULL " } else { "" }, a.len()-1, b.len()-1))
    });
    check_effects("_wcsnicmp", |r| {
        let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
        let n = r.below(24);
        let na = r.below(32) == 0;
        let pa = if na { core::ptr::null() } else { a.as_ptr() };
        (Effects::subject(|| _wcsnicmp(pa, b.as_ptr(), n) as i64),
         Effects::reference(|| ref__wcsnicmp(pa, b.as_ptr(), n) as i64),
         format!("n={n} {}alen={} blen={}", if na { "NULL " } else { "" }, a.len()-1, b.len()-1))
    });
    // The bounded folders: `n` is drawn small enough to MISS the terminator often,
    // which is the arm that folds-then-fails. handler must never fire.
    check_wbuf_fx("_wcslwr_s", &["ok", "EINVAL"], |r| {
        let s = gen_wstr(r, 24);
        let n = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcslwr_s(d1.as_mut_ptr(), n) as i64);
        let w = Effects::reference(|| ref__wcslwr_s(d2.as_mut_ptr(), n) as i64);
        (d1, g, d2, w, format!("n={n} slen={}", s.len()-1))
    });
    check_wbuf_fx("_wcsupr_s", &["ok", "EINVAL"], |r| {
        let s = gen_wstr(r, 24);
        let n = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcsupr_s(d1.as_mut_ptr(), n) as i64);
        let w = Effects::reference(|| ref__wcsupr_s(d2.as_mut_ptr(), n) as i64);
        (d1, g, d2, w, format!("n={n} slen={}", s.len()-1))
    });
    // ══════════ WAVE 11 — the SAME ROWS, WITH A REAL LOCALE PLANTED ══════════
    //
    // THIS IS THE POINT OF WAVE 11. Every row above ran with `lc_handle[LC_CTYPE] == 0`,
    // and msvcrt BRANCHES on that:
    //
    //     if(!locinfo->lc_handle[LC_CTYPE]) { ...inline ASCII fold... }   // what we tested
    //     else                              { ..._tolower_l / LCMapStringW... }  // what we didn't
    //
    // So waves 7-8 could ship a path-1-only implementation and be reported MATCH forever.
    // They did. `_stricmp`, `_strnicmp`, `_wcsicmp`, `_wcsnicmp`, `_strlwr_s`, `_strupr_s`
    // and the wide folders were all locale-INDEPENDENT bodies for locale-DEPENDENT
    // functions — correct in the "C" locale, wrong the moment setlocale is called. That is
    // §9.60, and it was shipped, not merely documented.
    //
    // Planting a real lc_handle makes path 2 live on BOTH sides. Both then reach the SAME
    // shared NLS stub (refimpl.c's — un-namespaced as of wave 11, because kernel32 is the
    // environment, not code under test), so what this grades is the Rust's LOGIC around the
    // NLS call: does it call the same function, with the same arguments, and read the result
    // the same way. Windows' real NLS *data* is out of scope, correctly — that is kernel32's
    // job, and it is the same bounded scope _toupper_l's __crtLCMapStringA arm already has.
    //
    // A green from an unexercised branch is not a green.

    // ══ WAVE 12 — the COLLATION family, graded in BOTH locale modes ══════════
    //
    // These exist as flips only because wave 11 exists. Every one branches on
    // `lc_handle[LC_COLLATE]`, so with the harness pinned to the "C" locale the
    // CompareStringA/LCMapStringA arm was unreachable and a path-1-only body would have
    // been reported MATCH forever — §9.60, exactly. The `[NLS]` copies below are the ones
    // that carry the weight.
    macro_rules! coll_rows {
        ($suffix:literal) => {
            check_effects(concat!("strcoll", $suffix), |r| {
                let (a, b) = (gen_str(r, 20), gen_str(r, 20));
                (Effects::subject(|| strcoll(a.as_ptr(), b.as_ptr()) as i64),
                 Effects::reference(|| ref_strcoll(a.as_ptr(), b.as_ptr()) as i64),
                 format!("{a:?} vs {b:?}"))
            });
            check_effects(concat!("_stricoll", $suffix), |r| {
                let (a, b) = (gen_str(r, 20), gen_str(r, 20));
                (Effects::subject(|| _stricoll(a.as_ptr(), b.as_ptr()) as i64),
                 Effects::reference(|| ref__stricoll(a.as_ptr(), b.as_ptr()) as i64),
                 format!("{a:?} vs {b:?}"))
            });
            // `_strncoll_l` path 1 is literally `return strncmp(str1, str2, count);` — so it
            // INHERITS strncmp's two contracts (string.c `#if defined(_WIN64)`): i386
            // returns the raw unsigned byte-diff, amd64/arm64 clamp to -1/0/1. The host
            // reference compiled the i386 arm (_WIN64 undefined), so the amd64 subject must
            // be graded against `clamp(ref)` — exactly as the `strncmp[_WIN64]` row does.
            //
            // classify's own header says "any NEW flip must be grepped for _WIN64 first",
            // and this row is what happens when you don't: it came back DIVERGE with
            // cnt=1 msvcrt=47.
            //
            // In the NLS arm the reference returns CompareStringA()-CSTR_EQUAL, already
            // -1/0/1, so the clamp is a no-op there. One row covers both modes.
            check_effects(concat!("_strncoll", $suffix), |r| {
                let (a, b) = (gen_str(r, 20), gen_str(r, 20));
                let n = r.below(24);
                (Effects::subject(|| _strncoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 Effects::reference(|| {
                     let w = ref__strncoll(a.as_ptr(), b.as_ptr(), n) as i64;
                     if cfg!(target_pointer_width = "64") { w.signum() } else { w }
                 }),
                 format!("n={n} {a:?} vs {b:?}"))
            });
            check_effects(concat!("_strnicoll", $suffix), |r| {
                let (a, b) = (gen_str(r, 20), gen_str(r, 20));
                let n = r.below(24);
                (Effects::subject(|| _strnicoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 Effects::reference(|| ref__strnicoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 format!("n={n} {a:?} vs {b:?}"))
            });
            check_buf_fx(concat!("strxfrm", $suffix), &["ok"], |r| {
                let src = gen_str(r, 20);
                let len = r.below(40);
                let (mut d1, mut d2) = (vec![0xAAu8; 64], vec![0xAAu8; 64]);
                let g = Effects::subject(|| strxfrm(d1.as_mut_ptr(), src.as_ptr(), len) as i64);
                let w = Effects::reference(|| ref_strxfrm(d2.as_mut_ptr(), src.as_ptr(), len) as i64);
                (d1, g, d2, w, format!("len={len} src={src:?}"))
            });
            check_effects(concat!("wcscoll", $suffix), |r| {
                let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
                (Effects::subject(|| wcscoll(a.as_ptr(), b.as_ptr()) as i64),
                 Effects::reference(|| ref_wcscoll(a.as_ptr(), b.as_ptr()) as i64),
                 format!("alen={} blen={}", a.len()-1, b.len()-1))
            });
            check_effects(concat!("_wcsicoll", $suffix), |r| {
                let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
                (Effects::subject(|| _wcsicoll(a.as_ptr(), b.as_ptr()) as i64),
                 Effects::reference(|| ref__wcsicoll(a.as_ptr(), b.as_ptr()) as i64),
                 format!("alen={} blen={}", a.len()-1, b.len()-1))
            });
            check_effects(concat!("_wcsncoll", $suffix), |r| {
                let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
                let n = r.below(24);
                (Effects::subject(|| _wcsncoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 Effects::reference(|| ref__wcsncoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 format!("n={n}"))
            });
            check_effects(concat!("_wcsnicoll", $suffix), |r| {
                let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
                let n = r.below(24);
                (Effects::subject(|| _wcsnicoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 Effects::reference(|| ref__wcsnicoll(a.as_ptr(), b.as_ptr(), n) as i64),
                 format!("n={n}"))
            });
            check_wbuf_fx(concat!("wcsxfrm", $suffix), &["ok"], |r| {
                let src = gen_wstr(r, 20);
                let len = r.below(40);
                let (mut d1, mut d2) = (vec![0xAAAAu16; 64], vec![0xAAAAu16; 64]);
                let g = Effects::subject(|| wcsxfrm(d1.as_mut_ptr(), src.as_ptr(), len) as i64);
                let w = Effects::reference(|| ref_wcsxfrm(d2.as_mut_ptr(), src.as_ptr(), len) as i64);
                (d1, g, d2, w, format!("len={len} slen={}", src.len()-1))
            });
        };
    }
    coll_rows!("");                       // the "C" locale: path 1 (strcmp / ASCII fold)

    unsafe { ref_set_nls(1); subj_set_nls(1); }
    coll_rows!(" [NLS]");                 // a REAL locale: path 2 (CompareString / LCMapString)

    check_effects("_stricmp [NLS]", |r| {
        let (a, b) = (gen_str(r, 20), gen_str(r, 20));
        (Effects::subject(|| _stricmp(a.as_ptr(), b.as_ptr()) as i64),
         Effects::reference(|| ref__stricmp(a.as_ptr(), b.as_ptr()) as i64),
         format!("{a:?} vs {b:?}"))
    });
    check_effects("_strnicmp [NLS]", |r| {
        let (a, b) = (gen_str(r, 20), gen_str(r, 20));
        let n = r.below(24);
        (Effects::subject(|| _strnicmp(a.as_ptr(), b.as_ptr(), n) as i64),
         Effects::reference(|| ref__strnicmp(a.as_ptr(), b.as_ptr(), n) as i64),
         format!("n={n} {a:?} vs {b:?}"))
    });
    check_buf_fx("_strlwr_s [NLS]", &["ok", "EINVAL"], |r| {
        let s = gen_str(r, 24);
        let len = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAA); d2.resize(40, 0xAA);
        let g = Effects::subject(|| _strlwr_s(d1.as_mut_ptr(), len) as i64);
        let w = Effects::reference(|| ref__strlwr_s(d2.as_mut_ptr(), len) as i64);
        (d1, g, d2, w, format!("len={len} slen={}", s.len()-1))
    });
    check_buf_fx("_strupr_s [NLS]", &["ok", "EINVAL"], |r| {
        let s = gen_str(r, 24);
        let len = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAA); d2.resize(40, 0xAA);
        let g = Effects::subject(|| _strupr_s(d1.as_mut_ptr(), len) as i64);
        let w = Effects::reference(|| ref__strupr_s(d2.as_mut_ptr(), len) as i64);
        (d1, g, d2, w, format!("len={len} slen={}", s.len()-1))
    });
    check_effects("_wcsicmp [NLS]", |r| {
        let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
        (Effects::subject(|| _wcsicmp(a.as_ptr(), b.as_ptr()) as i64),
         Effects::reference(|| ref__wcsicmp(a.as_ptr(), b.as_ptr()) as i64),
         format!("alen={} blen={}", a.len()-1, b.len()-1))
    });
    check_effects("_wcsnicmp [NLS]", |r| {
        let (a, b) = (gen_wstr(r, 20), gen_wstr(r, 20));
        let n = r.below(24);
        (Effects::subject(|| _wcsnicmp(a.as_ptr(), b.as_ptr(), n) as i64),
         Effects::reference(|| ref__wcsnicmp(a.as_ptr(), b.as_ptr(), n) as i64),
         format!("n={n} alen={} blen={}", a.len()-1, b.len()-1))
    });
    check_wbuf_fx("_wcslwr_s [NLS]", &["ok", "EINVAL"], |r| {
        let s = gen_wstr(r, 24);
        let n = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcslwr_s(d1.as_mut_ptr(), n) as i64);
        let w = Effects::reference(|| ref__wcslwr_s(d2.as_mut_ptr(), n) as i64);
        (d1, g, d2, w, format!("n={n} slen={}", s.len()-1))
    });
    check_wbuf_fx("_wcsupr_s [NLS]", &["ok", "EINVAL"], |r| {
        let s = gen_wstr(r, 24);
        let n = r.below(28);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        d1.resize(40, 0xAAAA); d2.resize(40, 0xAAAA);
        let g = Effects::subject(|| _wcsupr_s(d1.as_mut_ptr(), n) as i64);
        let w = Effects::reference(|| ref__wcsupr_s(d2.as_mut_ptr(), n) as i64);
        (d1, g, d2, w, format!("n={n} slen={}", s.len()-1))
    });

    // The wide case mappers themselves, EXHAUSTIVE over the whole 16-bit wint_t domain,
    // in BOTH locale modes. (`wint_t` is 16-bit in msvcrt, so WEOF is 0xFFFF — not the
    // host's 32-bit 0xFFFFFFFF.)
    macro_rules! wcase_row {
        ($name:literal, $subj:expr, $refr:expr) => {{
            let mut bad = None;
            for c in 0u32..=0xFFFF {
                let c = c as u16;
                let (g, w) = ($subj(c), $refr(c));
                if g != w {
                    bad = Some(format!("c=0x{c:04x} cnt={g} msvcrt={w}"));
                    break;
                }
            }
            match bad {
                Some(b) => record($name, V::Diverge, b),
                None => record($name, V::Match, "exhaustive over 0x0000..=0xFFFF".into()),
            }
        }};
    }
    wcase_row!("towlower [NLS]", towlower, ref_towlower);
    wcase_row!("towupper [NLS]", towupper, ref_towupper);

    unsafe { ref_set_nls(0); subj_set_nls(0); }   // back to the "C" locale

    wcase_row!("towlower", towlower, ref_towlower);
    wcase_row!("towupper", towupper, ref_towupper);
    wcase_row!("_towlower_l", |c| _towlower_l(c, core::ptr::null_mut()),
                              |c| ref__towlower_l(c, core::ptr::null_mut()));
    wcase_row!("_towupper_l", |c| _towupper_l(c, core::ptr::null_mut()),
                              |c| ref__towupper_l(c, core::ptr::null_mut()));

    check_wbuf("_wcslwr", |r| {
        let s = gen_wstr(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _wcslwr(d1.as_mut_ptr()); let p2 = ref__wcslwr(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("len={}", s.len()-1))
    });
    check_wbuf("_wcsupr", |r| {
        let s = gen_wstr(r, 30);
        let (mut d1, mut d2) = (s.clone(), s.clone());
        let p1 = _wcsupr(d1.as_mut_ptr()); let p2 = ref__wcsupr(d2.as_mut_ptr());
        let (o1, o2) = (p1 as isize - d1.as_ptr() as isize, p2 as isize - d2.as_ptr() as isize);
        (d1, o1, d2, o2, format!("len={}", s.len()-1))
    });


    // ═════ WAVE 9: the wide ctype family ══════════════════════════════════════
    //
    // EXHAUSTIVE over the u16 domain, not fuzzed. The domain is only 65,536 wide
    // and it has three regimes the C treats differently:
    //     wc == WEOF (0xFFFF) -> 0, unconditionally
    //     wc <  256           -> MSVCRT__pwctype[wc] & mask   (table)
    //     wc >= 256           -> GetStringTypeW(...)  & mask   (NLS)
    // A random fuzz would hit all three, but exhaustion PROVES all three, and for
    // 65,536 cases x 13 predicates there is no reason to settle for less.
    //
    // WAVE 10 made this honest. Both sides now read msvcrt's REAL wide ctype table
    // (`MSVCRT__wctype`, which ctype.c owns and which is deliberately NOT namespaced —
    // it is shared ENVIRONMENT, like the locale itself). Until then the subject read
    // subjectimpl.c's hand-written 256-entry model and the reference read refimpl.c's,
    // and a differential test can never catch data it feeds to BOTH sides: a mistyped
    // byte makes both wrong the same way and still reports MATCH. The models are gone.
    macro_rules! ctype_row {
        ($name:literal, $subj:ident, $refr:ident) => {
            {
                let mut bad = None;
                for wc in 0u32..=0xFFFF {
                    let (g, w) = ($subj(wc), $refr(wc));
                    // msvcrt returns the raw masked bits, not a normalised 0/1 —
                    // so compare the VALUES, not their truthiness.
                    if g != w {
                        bad = Some(format!("wc=0x{wc:04x} cnt={g} msvcrt={w}"));
                        break;
                    }
                }
                match bad {
                    Some(b) => record($name, V::Diverge, b),
                    None => record($name, V::Match, "exhaustive over 0x0000..=0xFFFF".into()),
                }
            }
        };
    }
    ctype_row!("iswalnum",  iswalnum,  ref_iswalnum);
    ctype_row!("iswalpha",  iswalpha,  ref_iswalpha);
    ctype_row!("iswcntrl",  iswcntrl,  ref_iswcntrl);
    ctype_row!("iswdigit",  iswdigit,  ref_iswdigit);
    ctype_row!("iswgraph",  iswgraph,  ref_iswgraph);
    ctype_row!("iswlower",  iswlower,  ref_iswlower);
    ctype_row!("iswprint",  iswprint,  ref_iswprint);
    ctype_row!("iswpunct",  iswpunct,  ref_iswpunct);
    ctype_row!("iswspace",  iswspace,  ref_iswspace);
    ctype_row!("iswupper",  iswupper,  ref_iswupper);
    ctype_row!("iswxdigit", iswxdigit, ref_iswxdigit);
    ctype_row!("iswblank",  iswblank,  ref_iswblank);

    // ══ WAVE 10 — the NARROW ctype/case family (the locale-table door) ══════════
    //
    // EXHAUSTIVE over -300..=600, not fuzzed. That range is chosen to straddle every
    // regime msvcrt treats differently, and one of them is the whole point of the wave:
    //
    //     c <  -1        -> a sign-extended high-bit `char`. `(unsigned)c < 256` is
    //                       FALSE, so `_tolower_l` MISSES the table and takes its NLS
    //                       arm, which SETS errno = EILSEQ. msvcrt's own callers reach
    //                       this constantly, because string.c hands `_tolower_l` a raw
    //                       signed char with no cast (while the `_isspace_l` two lines
    //                       above it DOES cast). This arm is why five shipped parsers
    //                       and _memicmp were divergent the moment the oracle got honest.
    //     c == -1        -> the EOF slot. pctype is `MSVCRT__ctype + 1`, so pctype[-1]
    //                       is a REAL addressable entry — index it unsigned and you read
    //                       at 0xFFFFFFFF instead.
    //     0 ..= 255      -> the table proper (pctype / pclmap / pcumap).
    //     256 ..= 600    -> the `c >= 256` NLS arm again, from the other side.
    //
    // And these grade ALL FOUR observable effects (return, errno, handler), not just the
    // return — a `to*` row that compared only the value would be blind to the EILSEQ,
    // which is exactly the effect that was hiding six shipped bugs.
    macro_rules! nctype_row {
        ($name:literal, $subj:expr, $refr:expr) => {{
            let mut bad = None;
            for c in -300i32..=600 {
                let g = Effects::subject(|| $subj(c) as i64);
                let w = Effects::reference(|| $refr(c) as i64);
                if g != w {
                    bad = Some(format!("c={c} cnt={g:?} msvcrt={w:?}"));
                    break;
                }
            }
            match bad {
                Some(b) => record($name, V::Diverge, b),
                None => record($name, V::Match, "exhaustive over -300..=600 (all 4 effects)".into()),
            }
        }};
    }

    nctype_row!("isalnum",    isalnum,    ref_isalnum);
    nctype_row!("isalpha",    isalpha,    ref_isalpha);
    nctype_row!("iscntrl",    iscntrl,    ref_iscntrl);
    nctype_row!("isdigit",    isdigit,    ref_isdigit);
    nctype_row!("isgraph",    isgraph,    ref_isgraph);
    nctype_row!("islower",    islower,    ref_islower);
    nctype_row!("isprint",    isprint,    ref_isprint);
    nctype_row!("ispunct",    ispunct,    ref_ispunct);
    nctype_row!("isspace",    isspace,    ref_isspace);
    nctype_row!("isupper",    isupper,    ref_isupper);
    nctype_row!("isxdigit",   isxdigit,   ref_isxdigit);
    nctype_row!("isblank",    isblank,    ref_isblank);
    // …and its `_l` sibling, which returns the MASK where the plain form's `||`
    // normalises to 0/1. Same file, three lines apart, different contracts.
    nctype_row!("_isblank_l", |c| _isblank_l(c, core::ptr::null_mut()),
                              |c| ref__isblank_l(c, core::ptr::null_mut()));
    // `_ispunct_l` is (MSVCR80.@) — compiled and exported by the object, absent from
    // msvcrt.spec. It was C until wave 25 and the reference TU owned its plain name, so
    // this row could not exist; renaming it in build.rs is what made it gradeable.
    nctype_row!("_ispunct_l", |c| _ispunct_l(c, core::ptr::null_mut()),
                              |c| ref__ispunct_l(c, core::ptr::null_mut()));
    nctype_row!("isleadbyte", isleadbyte, ref_isleadbyte);
    nctype_row!("__isascii",  __isascii,  ref___isascii);
    nctype_row!("__toascii",  __toascii,  ref___toascii);
    nctype_row!("__iscsym",   __iscsym,   ref___iscsym);
    nctype_row!("__iscsymf",  __iscsymf,  ref___iscsymf);

    // The case-mapping family. `toupper`/`tolower` short-circuit on the global
    // `initial_locale`; `_toupper`/`_tolower` are msvcrt's `/* sic */` — UNCONDITIONAL
    // `c -/+ 0x20`, no range check, so `_toupper('0')` is 0x10. A "reasonable"
    // implementation of those two would be a bug, and this row is what says so.
    nctype_row!("toupper",    toupper,    ref_toupper);
    nctype_row!("tolower",    tolower,    ref_tolower);
    nctype_row!("_toupper",   _toupper,   ref__toupper);
    nctype_row!("_tolower",   _tolower,   ref__tolower);
    nctype_row!("_toupper_l", |c| _toupper_l(c, core::ptr::null_mut()),
                              |c| ref__toupper_l(c, core::ptr::null_mut()));
    nctype_row!("_tolower_l", |c| _tolower_l(c, core::ptr::null_mut()),
                              |c| ref__tolower_l(c, core::ptr::null_mut()));

    // _isctype over every mask, exhaustively — the primitive the whole is* family
    // funnels through.
    {
        const MASKS: [i32; 10] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x8000];
        let mut bad = None;
        'outer: for &ty in &MASKS {
            for c in -300i32..=600 {
                let g = Effects::subject(|| _isctype(c, ty) as i64);
                let w = Effects::reference(|| ref__isctype(c, ty) as i64);
                if g != w {
                    bad = Some(format!("c={c} mask=0x{ty:x} cnt={g:?} msvcrt={w:?}"));
                    break 'outer;
                }
            }
        }
        match bad {
            Some(b) => record("_isctype", V::Diverge, b),
            None => record("_isctype", V::Match, "exhaustive: 10 masks x -300..=600".into()),
        }
    }
    // iswctype with EVERY mask bit, over the whole domain.
    {
        let mut bad = None;
        'outer: for ty in [0x1u16, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x103, 0x8000, 0xFFFF] {
            for wc in 0u32..=0xFFFF {
                let (g, w) = (iswctype(wc, ty), ref_iswctype(wc, ty));
                if g != w {
                    bad = Some(format!("wc=0x{wc:04x} mask=0x{ty:04x} cnt={g} msvcrt={w}"));
                    break 'outer;
                }
            }
        }
        match bad {
            Some(b) => record("iswctype", V::Diverge, b),
            None => record("iswctype", V::Match, "exhaustive: 12 masks x 0x0000..=0xFFFF".into()),
        }
    }

    // ══ WAVE 13 — the float PARSER (msvcrt's own bignum, ported) ══════════════
    //
    // The wave-5 probe (above) is still here and still DIVERGEs, and that is the point:
    // it records that cnt-rust's DJ Delorie parser is a DIFFERENT ALGORITHM and can never
    // be flipped. What changed is that `strtod` no longer takes that route — the Rust in
    // msvcrt-crt-rust/src/fpnum.rs IS msvcrt's bignum, transcribed from bnum.h +
    // string.c. So the probe should stay red while the rows below go green. If the probe
    // ever turns green, something has quietly re-pointed it at the new engine and it has
    // stopped being a control.
    //
    // The oracle here is the strongest the lane has: an exact 64-bit BIT PATTERN. Not
    // `==` — `-0.0 == 0.0` would hide a sign bug and a 1-ULP slip is a real CRT defect.
    fp_rows();

}

// ── wave 13's float instrumentation ───────────────────────────────────────────

/// Everything a float door can observe: the value's BITS, where it stopped, and both
/// side-channels. Compared as a unit — a right value with a wrong endptr is a bug, and so
/// is a right value with a missing ERANGE.
#[derive(PartialEq, Default)]
struct Fp {
    bits: u64,
    end: i64,
    errno: i32,
    pmt: i32,
    ret: i32,
}

impl core::fmt::Display for Fp {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "0x{:016x} ({:?})  end={}  errno={}  pmt={}  ret={}",
            self.bits,
            f64::from_bits(self.bits),
            self.end,
            self.errno,
            self.pmt,
            self.ret
        )
    }
}

/// Wave 13's corpus.
///
/// `gen_float_str` (wave 5) was built to embarrass a NAIVE parser — long mantissas, hard
/// exponents. That is not enough here, because the thing under test IS the bignum, and a
/// bignum has whole regions of behaviour a short input can never reach:
///
///   * the **circular buffer running out of limbs** (128 limbs x 9 digits = 1152 digits),
///     whose overflow arm is a sticky-OR that NO shorter input executes;
///   * the limb-boundary `bnum_lshift(29)` / `bnum_rshift(9)` normalisation loops;
///   * the `INT_MAX`-saturating exponent arithmetic;
///   * the rollback/`unget` paths on malformed tails;
///   * and the `_MSVCR_VER == 0` contract — `inf`/`nan`/`0x` must NOT parse.
///
/// A fuzzer only falsifies what it VARIES (§9.60), so each of those is a deliberate arm.
fn gen_fp(r: &mut Rng) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    match r.below(24) {
        // ── EXACT HALFWAY TIES (12% of the corpus). See gen_tie: these are the only
        //    inputs that reach FP_ROUND_EVEN at all, and the only ones that can falsify
        //    the sticky-scan transcription. Random digits are NEVER ties. ──
        19 | 20 => return gen_tie(r),
        // ── the BNUM-LEVEL tie: 27 significant digits, then a '5', then a zero run, then
        //    a lone non-zero digit. The bignum keeps 3 limbs = 27 decimal digits of
        //    mantissa, so THIS is the cut its round mode is about. ──
        21 => {
            for _ in 0..27 { v.push(b'1' + r.below(9) as u8); }
            v.push(b'5');
            for _ in 0..r.below(30) { v.push(b'0'); }
            if r.below(2) == 0 { v.push(b'1' + r.below(9) as u8); }
            if r.below(2) == 0 {
                v.push(b'e');
                if r.below(2) == 0 { v.push(b'-'); }
                v.extend_from_slice(format!("{}", r.below(60)).as_bytes());
            }
        }

        // ── fixed torture literals: the exact IEEE boundaries, where 1 ULP shows ──
        0 => v.extend_from_slice(b"1.7976931348623157e308"), // DBL_MAX
        1 => v.extend_from_slice(b"1.7976931348623159e308"), // just over -> inf + ERANGE
        2 => v.extend_from_slice(b"2.2250738585072014e-308"), // smallest normal
        3 => v.extend_from_slice(b"2.2250738585072011e-308"), // the famous "PHP hang" denormal
        4 => v.extend_from_slice(b"4.9406564584124654e-324"), // smallest subnormal
        5 => v.extend_from_slice(b"2.4703282292062327e-324"), // half of it — rounds to 0 or to min
        6 => v.extend_from_slice(b"0.0000000000000000087519423665610e-32"), // wave 5's 14-ULP input
        7 => v.extend_from_slice(b"9007199254740993"), // 2^53+1: first unrepresentable integer
        8 => v.extend_from_slice(b"1e400"),           // overflow -> ERANGE
        9 => v.extend_from_slice(b"1e-400"),          // underflow -> ERANGE

        // ── exponent arithmetic that saturates at INT_MAX inside the parser ──
        10 => {
            v.extend_from_slice(b"1e");
            if r.below(2) == 0 {
                v.push(b'-');
            }
            v.extend_from_slice(format!("{}", r.next() % 4_000_000_000u64).as_bytes());
        }

        // ── THE CIRCULAR BUFFER. >1152 digits exhausts the 128-limb ring, and the
        //    "buffer full" arm folds the rest into a sticky bit. Unreachable below ~1152. ──
        11 => {
            for _ in 0..1100 + r.below(400) {
                v.push(b'0' + r.below(10) as u8);
            }
            if r.below(2) == 0 {
                v.push(b'.');
                for _ in 0..r.below(300) {
                    v.push(b'0' + r.below(10) as u8);
                }
            }
        }

        // ── halfway cases: an exact "5" tail is where FP_ROUND_EVEN decides ──
        12 => {
            for _ in 0..15 + r.below(6) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(b'.');
            for _ in 0..r.below(20) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(b'5');
            if r.below(2) == 0 {
                for _ in 0..r.below(6) {
                    v.push(b'0'); // trailing zeros: still EXACTLY half -> ties-to-even
                }
            } else {
                v.push(b'1' + r.below(9) as u8); // ...or a hair over half -> round up
            }
        }

        // ── the _MSVCR_VER == 0 CONTRACT. These must NOT parse in *this* CRT. ──
        13 => v.extend_from_slice(match r.below(6) {
            0 => &b"inf"[..],
            1 => &b"INFINITY"[..],
            2 => &b"nan"[..],
            3 => &b"NaN"[..],
            4 => &b"0x10"[..],
            _ => &b"0X1p3"[..],
        }),

        // ── the 'd'/'D' Fortran exponent msvcrt has never dropped ──
        14 => {
            for _ in 0..1 + r.below(8) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(if r.below(2) == 0 { b'd' } else { b'D' });
            if r.below(2) == 0 {
                v.push(b'-');
            }
            v.extend_from_slice(format!("{}", r.below(320)).as_bytes());
        }

        // ── the rollback/unget paths: malformed, partially-formed, empty ──
        15 => v.extend_from_slice(match r.below(9) {
            0 => &b""[..],
            1 => &b"."[..],
            2 => &b"-"[..],
            3 => &b"+"[..],
            4 => &b"-."[..],
            5 => &b"1e"[..],
            6 => &b"1e+"[..],
            7 => &b".e5"[..],
            _ => &b"-.e-"[..],
        }),

        // ── THE SIGNED-CHAR / WEOF TRAP. A 0xFF byte sign-extends to 0xFFFF == WEOF, so
        //    the parser sees end-of-input at a byte it has ALREADY stepped over. ──
        16 => {
            for _ in 0..1 + r.below(4) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(0xFF);
            for _ in 0..r.below(3) {
                v.push(b'0' + r.below(10) as u8);
            }
        }

        // ── the other high bytes: 0x80..0xFE widen to 0xFF80..0xFFFE, which are NOT WEOF ──
        17 => {
            for _ in 0..1 + r.below(4) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(0x80 + r.below(0x7F) as u8);
            for _ in 0..r.below(3) {
                v.push(b'0' + r.below(10) as u8);
            }
        }

        // ── leading whitespace (the skip loop) then the parser's own sign ──
        18 => {
            for _ in 0..r.below(4) {
                v.push([b' ', b'\t', b'\n', b'\r', 0x0b, 0x0c][r.below(6)]);
            }
            if r.below(2) == 0 {
                v.push(if r.below(2) == 0 { b'-' } else { b'+' });
            }
            for _ in 0..1 + r.below(20) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(b'.');
            for _ in 0..r.below(25) {
                v.push(b'0' + r.below(10) as u8);
            }
        }

        // ── the bulk: general random floats across the whole exponent range ──
        _ => {
            if r.below(3) == 0 {
                v.push(if r.below(2) == 0 { b'-' } else { b'+' });
            }
            for _ in 0..r.below(30) {
                v.push(b'0' + r.below(10) as u8);
            }
            if r.below(4) != 0 {
                v.push(b'.');
                for _ in 0..r.below(30) {
                    v.push(b'0' + r.below(10) as u8);
                }
            }
            if r.below(2) == 0 {
                v.push(if r.below(2) == 0 { b'e' } else { b'E' });
                match r.below(3) {
                    0 => v.push(b'-'),
                    1 => v.push(b'+'),
                    _ => {}
                }
                v.extend_from_slice(format!("{}", r.below(700)).as_bytes());
            }
            if r.below(6) == 0 {
                v.extend_from_slice(b"xyz"); // trailing junk -> exercises the endptr
            }
        }
    }
    v.push(0);
    v
}

/// Multiply a decimal digit-string (most significant first) by a small integer, in place.
/// Schoolbook, exact, no float anywhere near it.
fn dec_mul(d: &mut Vec<u8>, k: u32) {
    let mut carry = 0u32;
    for x in d.iter_mut().rev() {
        let t = (*x as u32) * k + carry;
        *x = (t % 10) as u8;
        carry = t / 10;
    }
    while carry > 0 {
        d.insert(0, (carry % 10) as u8);
        carry /= 10;
    }
}

/// **Exact halfway ties** — the inputs a correctly-rounded parser exists for, and the ones
/// a random-digit fuzzer will never, ever produce.
///
/// A tie is a decimal that lands exactly midway between two adjacent doubles: take a 53-bit
/// mantissa `m`, and the value `(2m+1) * 2^(k-53)` is precisely the midpoint of the two
/// doubles with mantissas `m` and `m+1`. Every such value has a FINITE exact decimal
/// expansion (because `2^-n == 5^n / 10^n`), so we can write it down exactly with schoolbook
/// arithmetic — no float involved, no rounding introduced by the generator itself.
///
/// This matters for two reasons, and I only found the second one by trying to break the
/// first:
///
///   1. **`FP_ROUND_EVEN` is only reachable from a tie.** Without this generator the entire
///      ties-to-even arm of `fpnum_double` — the whole reason msvcrt carries a bignum —
///      is dead code as far as the harness is concerned. 20,000 rows of random digits and
///      not one of them is a tie.
///
///   2. **It is the only way to falsify Wine's sticky-scan bug.** `fpnum_parse_bnum`'s
///      round decision reads limb `e-4` and then scans the limbs below it — and Wine's scan
///      re-reads the LOWEST limb every iteration instead of the i'th (see fpnum.rs). The
///      two behaviours diverge only when limb `e-4` is EXACTLY `LIMB_MAX/2`, which is to say
///      only on a tie, and only when the limbs below it are a MIX of zero and non-zero.
///      So: emit the tie, then optionally push a lone non-zero digit far out to the right,
///      past enough zeros to leave an intervening limb empty. `nudge` is that offset.
///
/// Negative-controlled: with `b.at(b.b)` "corrected" to `b.at(i)` in fpnum.rs, these rows
/// go DIVERGE. Before this generator existed, they did not — the fix was invisible, and the
/// transcription was therefore unverified. A guard that cannot be shown to fail is worthless.
fn gen_tie(r: &mut Rng) -> Vec<u8> {
    // A 53-bit mantissa, skewed to the extremes where the exponent logic is liveliest.
    let m: u64 = match r.below(4) {
        0 => 1u64 << 52,                       // the smallest normal mantissa
        1 => (1u64 << 53) - 1,                 // the largest
        2 => (1u64 << 52) | (r.next() & 0xF),  // just above the boundary
        _ => (1u64 << 52) | (r.next() & ((1 << 52) - 1)),
    };
    let odd = 2 * m + 1; // the midpoint's numerator: always odd, always exact

    // 2^k, for k on both sides of zero — negative k is where the FINITE-but-long decimal
    // expansions live (2^-60 is 60 digits of exactness), which is what stresses the limbs.
    let k: i32 = (r.below(140) as i32) - 120;

    let mut digits: Vec<u8> = odd
        .to_string()
        .bytes()
        .map(|c| c - b'0')
        .collect();

    let mut point_from_right: i32 = 0;
    if k >= 0 {
        for _ in 0..k {
            dec_mul(&mut digits, 2);
        }
    } else {
        for _ in 0..(-k) {
            dec_mul(&mut digits, 5); // 2^-1 == 5/10: multiply by 5, move the point one left
        }
        point_from_right = -k;
    }

    // Render as an exact decimal string.
    let mut s: Vec<u8> = Vec::new();
    if r.below(4) == 0 {
        s.push(b'-');
    }
    let n = digits.len() as i32;
    if point_from_right == 0 {
        s.extend(digits.iter().map(|&d| d + b'0'));
    } else if point_from_right >= n {
        s.extend_from_slice(b"0.");
        for _ in 0..(point_from_right - n) {
            s.push(b'0');
        }
        s.extend(digits.iter().map(|&d| d + b'0'));
    } else {
        let split = (n - point_from_right) as usize;
        s.extend(digits[..split].iter().map(|&d| d + b'0'));
        s.push(b'.');
        s.extend(digits[split..].iter().map(|&d| d + b'0'));
    }

    // The nudge: a lone non-zero digit pushed out past `pad` zeros. `pad == 0` leaves the
    // value an EXACT tie (ties-to-even decides it); a large `pad` leaves limb e-4 sitting
    // at exactly 500000000 while a limb further down is non-zero — the one state in which
    // Wine's sticky-scan bug is observable.
    match r.below(3) {
        0 => {} // leave it an exact tie
        _ => {
            if point_from_right == 0 {
                s.push(b'.');
            }
            let pad = r.below(45);
            for _ in 0..pad {
                s.push(b'0');
            }
            s.push(b'1' + r.below(9) as u8);
        }
    }

    s.push(0);
    s
}

/// The wide corpus is the narrow one widened — which is not a lazy shortcut but the
/// interesting case: byte 0xFF widens to `L'\u{00FF}'` (ÿ), which is **NOT** WEOF, so the
/// wide parser does NOT stop there while the narrow one does. Same input text, genuinely
/// different contract, and both sides of the harness see it. A literal 0xFFFF is injected
/// separately to reach the wide side's own WEOF.
fn gen_fp_w(r: &mut Rng) -> Vec<u16> {
    let b = gen_fp(r);
    let mut v: Vec<u16> = b.iter().map(|&c| c as u16).collect();
    if r.below(16) == 0 && v.len() > 2 {
        let at = 1 + r.below(v.len() - 2);
        v[at] = 0xFFFF;
    }
    v
}

/// Compare a float door on all its observables, and REFUSE to report MATCH on a row that
/// never exercised the contract it exists to test.
///
/// The coverage gate is the §9.60 rule made mechanical: a `strtod` row that only ever saw
/// well-formed in-range inputs has proved nothing about ERANGE or about rejection, and its
/// green would be worthless.
///
/// `require` is per-row and NOT boilerplate, because the family splits on *how it reports
/// errors*:
///
///   * `strtod`/`wcstod`/`atof` set **errno** — so they must be seen to produce ERANGE.
///   * `_atoflt_l`/`_atodbl`/`__STRINGTOLD_L` take the `perr` arm of `strtod_helper` and
///     never touch errno at all: the range error comes back **out-of-band** as an
///     `_OVERFLOW`/`_UNDERFLOW` **return code**. Demanding an ERANGE of those rows would
///     be demanding the wrong contract — and a blanket `require` list would have done
///     exactly that, failing them for behaving correctly.
///
/// Which is itself the lesson: the coverage assertion has to know what the function
/// promises, or it is just another thing that is confidently wrong.
fn check_fp(
    name: &'static str,
    require: &[&'static str],
    mut f: impl FnMut(&mut Rng) -> (Fp, Fp, String),
) {
    let mut r = Rng(0xD1B54A32D192ED03);
    let (mut saw_erange, mut saw_value, mut saw_reject, mut saw_ret) = (0u32, 0u32, 0u32, 0u32);

    for _ in 0..20_000 {
        let (got, want, ctx) = f(&mut r);

        // Coverage is measured on the REFERENCE — what msvcrt actually did — so a subject
        // that never sets errno cannot flatter its own coverage.
        if want.errno == 34 {
            saw_erange += 1;
        }
        if want.bits != 0 && want.bits != 0x8000_0000_0000_0000 {
            saw_value += 1;
        }
        if want.end == 0 {
            saw_reject += 1;
        }
        if want.ret != 0 {
            saw_ret += 1;
        }

        if got != want {
            record(
                name,
                V::Diverge,
                format!("[{ctx}]\n        rust  = {got}\n        msvcrt= {want}"),
            );
            return;
        }
    }

    let missing: Vec<&str> = require
        .iter()
        .copied()
        .filter(|w| match *w {
            "ERANGE" => saw_erange == 0,
            "value" => saw_value == 0,
            "reject" => saw_reject == 0,
            "ret" => saw_ret == 0,
            _ => false,
        })
        .collect();
    if !missing.is_empty() {
        record(
            name,
            V::Diverge,
            format!(
                "MATCHED, but the row never exercised: {}. \
                 A green from an unexercised contract is not a green (§9.60).",
                missing.join(", ")
            ),
        );
        return;
    }

    record(
        name,
        V::Match,
        format!("ERANGE x{saw_erange}, values x{saw_value}, rejects x{saw_reject}, err-ret x{saw_ret}"),
    );
}

fn fp_rows() {
    use core::ffi::c_void;

    #[repr(C)]
    struct CrtFloat {
        f: f32,
    }
    #[repr(C)]
    struct CrtDouble {
        x: f64,
    }

    extern "C" {
        // ── the reference (msvcrt's genuine string.c / wcs.c, compiled for the host) ──
        fn ref__strtod_l(s: *const u8, e: *mut *mut u8, l: *mut c_void) -> f64;
        fn ref__atof_l(s: *const u8, l: *mut c_void) -> f64;
        fn ref__atoflt_l(v: *mut CrtFloat, s: *mut u8, l: *mut c_void) -> i32;
        fn ref__atodbl(v: *mut CrtDouble, s: *mut u8) -> i32;
        fn ref__atodbl_l(v: *mut CrtDouble, s: *mut u8, l: *mut c_void) -> i32;
        fn ref_wcstod(s: *const u16, e: *mut *mut u16) -> f64;
        fn ref__wcstod_l(s: *const u16, e: *mut *mut u16, l: *mut c_void) -> f64;
        fn ref__wtof(s: *const u16) -> f64;
        fn ref__wtof_l(s: *const u16, l: *mut c_void) -> f64;
        fn ref__atoldbl(v: *mut u8, s: *mut u8) -> i32;
        fn ref___STRINGTOLD_L(
            v: *mut u8,
            e: *mut *mut u8,
            s: *const u8,
            f: i32,
            l: *mut c_void,
        ) -> i32;

        // ── the subject (msvcrt-crt-rust's #[no_mangle] bodies — the code that SHIPS) ──
        fn strtod(s: *const u8, e: *mut *mut u8) -> f64;
        fn _strtod_l(s: *const u8, e: *mut *mut u8, l: *mut c_void) -> f64;
        fn atof(s: *const u8) -> f64;
        fn _atof_l(s: *const u8, l: *mut c_void) -> f64;
        fn _atoflt_l(v: *mut CrtFloat, s: *mut u8, l: *mut c_void) -> i32;
        fn _atodbl(v: *mut CrtDouble, s: *mut u8) -> i32;
        fn _atodbl_l(v: *mut CrtDouble, s: *mut u8, l: *mut c_void) -> i32;
        fn wcstod(s: *const u16, e: *mut *mut u16) -> f64;
        fn _wcstod_l(s: *const u16, e: *mut *mut u16, l: *mut c_void) -> f64;
        fn _wtof(s: *const u16) -> f64;
        fn _wtof_l(s: *const u16, l: *mut c_void) -> f64;
        fn _atoldbl(v: *mut u8, s: *mut u8) -> i32;
        fn __STRINGTOLD_L(
            v: *mut u8,
            e: *mut *mut u8,
            s: *const u8,
            f: i32,
            l: *mut c_void,
        ) -> i32;
    }

    /// endptr as a displacement from the start of the string. `-1` for NULL.
    macro_rules! eoff {
        ($p:expr, $base:expr) => {
            if $p.is_null() {
                -1i64
            } else {
                ($p as usize as i64) - ($base.as_ptr() as usize as i64)
            }
        };
    }

    unsafe {
        // ── strtod / _strtod_l ────────────────────────────────────────────────
        check_fp("strtod", &["ERANGE", "value", "reject"], |r| {
            let s = gen_fp(r);
            let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
            let g = Effects::subject(|| strtod(s.as_ptr(), &mut e1).to_bits() as i64);
            let w = Effects::reference(|| ref_strtod(s.as_ptr(), &mut e2).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: eoff!(e1, s), errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: eoff!(e2, s), errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        check_fp("_strtod_l", &["ERANGE", "value", "reject"], |r| {
            let s = gen_fp(r);
            let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
            let n = core::ptr::null_mut();
            let g = Effects::subject(|| _strtod_l(s.as_ptr(), &mut e1, n).to_bits() as i64);
            let w = Effects::reference(|| ref__strtod_l(s.as_ptr(), &mut e2, n).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: eoff!(e1, s), errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: eoff!(e2, s), errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        // atof/_atof_l pass endptr = NULL, which is a DIFFERENT arm of strtod_helper
        // (the `#if _MSVCR_VER == 0` errno reset only happens on the perr==NULL path).
        check_fp("atof", &["ERANGE", "value"], |r| {
            let s = gen_fp(r);
            let g = Effects::subject(|| atof(s.as_ptr()).to_bits() as i64);
            let w = Effects::reference(|| ref_atof(s.as_ptr()).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: 0, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: 0, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        check_fp("_atof_l", &["ERANGE", "value"], |r| {
            let s = gen_fp(r);
            let n = core::ptr::null_mut();
            let g = Effects::subject(|| _atof_l(s.as_ptr(), n).to_bits() as i64);
            let w = Effects::reference(|| ref__atof_l(s.as_ptr(), n).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: 0, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: 0, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        // ── the _CRT_FLOAT / _CRT_DOUBLE doors ────────────────────────────────
        //
        // These take the `perr` arm of strtod_helper — errno is NOT touched; the ERANGE
        // comes back out-of-band and is folded into an _OVERFLOW/_UNDERFLOW return. So
        // the `errno` column here should stay 0 on BOTH sides, and the `ret` column is
        // where the contract lives. A row that graded only the value would miss all of it.
        check_fp("_atoflt_l", &["value", "ret"], |r| {
            let mut s = gen_fp(r);
            let n = core::ptr::null_mut();
            let (mut v1, mut v2) = (CrtFloat { f: 0.0 }, CrtFloat { f: 0.0 });
            let g = Effects::subject(|| _atoflt_l(&mut v1, s.as_mut_ptr(), n) as i64);
            let w = Effects::reference(|| ref__atoflt_l(&mut v2, s.as_mut_ptr(), n) as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: v1.f.to_bits() as u64, end: 0, errno: g.errno, pmt: g.pmt, ret: g.ret as i32 },
                Fp { bits: v2.f.to_bits() as u64, end: 0, errno: w.errno, pmt: w.pmt, ret: w.ret as i32 },
                ctx,
            )
        });

        check_fp("_atodbl", &["value", "ret"], |r| {
            let mut s = gen_fp(r);
            let (mut v1, mut v2) = (CrtDouble { x: 0.0 }, CrtDouble { x: 0.0 });
            let g = Effects::subject(|| _atodbl(&mut v1, s.as_mut_ptr()) as i64);
            let w = Effects::reference(|| ref__atodbl(&mut v2, s.as_mut_ptr()) as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: v1.x.to_bits(), end: 0, errno: g.errno, pmt: g.pmt, ret: g.ret as i32 },
                Fp { bits: v2.x.to_bits(), end: 0, errno: w.errno, pmt: w.pmt, ret: w.ret as i32 },
                ctx,
            )
        });

        check_fp("_atodbl_l", &["value", "ret"], |r| {
            let mut s = gen_fp(r);
            let n = core::ptr::null_mut();
            let (mut v1, mut v2) = (CrtDouble { x: 0.0 }, CrtDouble { x: 0.0 });
            let g = Effects::subject(|| _atodbl_l(&mut v1, s.as_mut_ptr(), n) as i64);
            let w = Effects::reference(|| ref__atodbl_l(&mut v2, s.as_mut_ptr(), n) as i64);
            let ctx = format!("{:?}", String::from_utf8_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: v1.x.to_bits(), end: 0, errno: g.errno, pmt: g.pmt, ret: g.ret as i32 },
                Fp { bits: v2.x.to_bits(), end: 0, errno: w.errno, pmt: w.pmt, ret: w.ret as i32 },
                ctx,
            )
        });

        // ── the wide family ───────────────────────────────────────────────────
        //
        // `_wcstod_l` is NOT `strtod_helper` with a wide cursor. The C wrote it out
        // separately and it is DIFFERENT: it does not zero errno on entry. Transcribe the
        // file you are replacing, not the one next to it — and grade it separately, which
        // is the only reason we would ever find out.
        check_fp("wcstod", &["ERANGE", "value", "reject"], |r| {
            let s = gen_fp_w(r);
            let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
            let g = Effects::subject(|| wcstod(s.as_ptr(), &mut e1).to_bits() as i64);
            let w = Effects::reference(|| ref_wcstod(s.as_ptr(), &mut e2).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf16_lossy(&s[..s.len() - 1]));
            // endptr displacement is in WCHARS on this side
            let (o1, o2) = (
                if e1.is_null() { -1 } else { (e1 as usize as i64 - s.as_ptr() as usize as i64) / 2 },
                if e2.is_null() { -1 } else { (e2 as usize as i64 - s.as_ptr() as usize as i64) / 2 },
            );
            (
                Fp { bits: g.ret as u64, end: o1, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: o2, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        check_fp("_wcstod_l", &["ERANGE", "value", "reject"], |r| {
            let s = gen_fp_w(r);
            let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
            let n = core::ptr::null_mut();
            let g = Effects::subject(|| _wcstod_l(s.as_ptr(), &mut e1, n).to_bits() as i64);
            let w = Effects::reference(|| ref__wcstod_l(s.as_ptr(), &mut e2, n).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf16_lossy(&s[..s.len() - 1]));
            let (o1, o2) = (
                if e1.is_null() { -1 } else { (e1 as usize as i64 - s.as_ptr() as usize as i64) / 2 },
                if e2.is_null() { -1 } else { (e2 as usize as i64 - s.as_ptr() as usize as i64) / 2 },
            );
            (
                Fp { bits: g.ret as u64, end: o1, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: o2, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        check_fp("_wtof", &["ERANGE", "value"], |r| {
            let s = gen_fp_w(r);
            let g = Effects::subject(|| _wtof(s.as_ptr()).to_bits() as i64);
            let w = Effects::reference(|| ref__wtof(s.as_ptr()).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf16_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: 0, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: 0, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        check_fp("_wtof_l", &["ERANGE", "value"], |r| {
            let s = gen_fp_w(r);
            let n = core::ptr::null_mut();
            let g = Effects::subject(|| _wtof_l(s.as_ptr(), n).to_bits() as i64);
            let w = Effects::reference(|| ref__wtof_l(s.as_ptr(), n).to_bits() as i64);
            let ctx = format!("{:?}", String::from_utf16_lossy(&s[..s.len() - 1]));
            (
                Fp { bits: g.ret as u64, end: 0, errno: g.errno, pmt: g.pmt, ret: 0 },
                Fp { bits: w.ret as u64, end: 0, errno: w.errno, pmt: w.pmt, ret: 0 },
                ctx,
            )
        });

        // ── the 80-bit family ─────────────────────────────────────────────────
        //
        // The out-param is a **10-byte** MSVCRT__LDOUBLE, while the parser computes into a
        // 12-byte WINE_BROKEN_LDOUBLE and copies only 10 out. ReactOS's own comment on
        // that struct is "The wine version is wrong and causes buffer overruns in the
        // test!", so this row hands BOTH sides a 16-byte buffer pre-filled with a sentinel
        // and compares ALL 16 — an overrun of even one byte past the tenth is a DIVERGE,
        // not a silent corruption of whatever the caller kept next to it.
        {
            let mut r = Rng(0x2545F4914F6CDD1D);
            let mut diverged: Option<String> = None;
            let (mut saw_ok, mut saw_err) = (0u32, 0u32);
            for _ in 0..20_000 {
                let mut s = gen_fp(&mut r);
                let (mut b1, mut b2) = ([0xAAu8; 16], [0xAAu8; 16]);
                let g = Effects::subject(|| _atoldbl(b1.as_mut_ptr(), s.as_mut_ptr()) as i64);
                let w = Effects::reference(|| ref__atoldbl(b2.as_mut_ptr(), s.as_mut_ptr()) as i64);
                if w.ret == 0 { saw_ok += 1; } else { saw_err += 1; }
                if b1 != b2 || g != w {
                    diverged = Some(format!(
                        "[{:?}]\n        rust  = {b1:02x?} ret={} errno={} pmt={}\n        msvcrt= {b2:02x?} ret={} errno={} pmt={}",
                        String::from_utf8_lossy(&s[..s.len() - 1]),
                        g.ret, g.errno, g.pmt, w.ret, w.errno, w.pmt));
                    break;
                }
            }
            match diverged {
                Some(d) => record("_atoldbl", V::Diverge, d),
                None if saw_ok == 0 || saw_err == 0 => record(
                    "_atoldbl", V::Diverge,
                    format!("MATCHED, but only saw ok={saw_ok} err={saw_err} — one arm was never exercised (§9.60)")),
                None => record("_atoldbl", V::Match,
                    format!("16-byte buffer compared (10 written + 6 sentinel: overrun-proof); ok x{saw_ok}, err x{saw_err}")),
            }
        }

        check_fp("__STRINGTOLD_L", &["value", "ret", "reject"], |r| {
            let mut s = gen_fp(r);
            let n = core::ptr::null_mut();
            let (mut b1, mut b2) = ([0xAAu8; 16], [0xAAu8; 16]);
            let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
            let g = Effects::subject(|| {
                __STRINGTOLD_L(b1.as_mut_ptr(), &mut e1, s.as_mut_ptr(), 0, n) as i64
            });
            let w = Effects::reference(|| {
                ref___STRINGTOLD_L(b2.as_mut_ptr(), &mut e2, s.as_mut_ptr(), 0, n) as i64
            });
            let ctx = format!(
                "{:?} bytes rust={b1:02x?} msvcrt={b2:02x?}",
                String::from_utf8_lossy(&s[..s.len() - 1])
            );
            // Fold the 16-byte out-param into the `bits` column so one comparison covers
            // the value, the return code, the endptr AND the overrun sentinel.
            let h = |b: &[u8; 16]| -> u64 {
                let mut h = 0xcbf29ce484222325u64;
                for &x in b {
                    h ^= x as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
                h
            };
            (
                Fp { bits: h(&b1), end: eoff!(e1, s), errno: g.errno, pmt: g.pmt, ret: g.ret as i32 },
                Fp { bits: h(&b2), end: eoff!(e2, s), errno: w.errno, pmt: w.pmt, ret: w.ret as i32 },
                ctx,
            )
        });
    }
}
