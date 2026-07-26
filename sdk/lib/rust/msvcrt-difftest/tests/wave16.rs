#![no_builtins] // the #[no_mangle] libc-named shims must be CALLED, not folded to LLVM builtins.
//! Wave 16 — grade the LAST 50 exports of `dll/win32/msvcrt/string.c` and `wcs.c`
//! (`msvcrt-crt-rust/src/string_l.rs`, `src/wcs_rest.rs`) against the GENUINE compiled C
//! that they replace (`ref_*`, built by build.rs from the real .c files).
//!
//! Same oracle and the same idiom as `classify.rs`: four observable effects per call
//! (return, errno, invalid-parameter handler, and the full destination image), a `require`
//! list per row that fails LOUDLY as UNEXERCISED when a named outcome class never happened,
//! and a fuzz corpus written to REACH the misbehaviours this port preserves on purpose.
//!
//! ## What this file grades that a naive fuzz would not
//!
//! Wave 16's whole point is that these bodies reproduce msvcrt's bugs. A corpus that never
//! reaches one has proved nothing about it, so each is a first-class, `require`d outcome:
//!
//!   * `_strtoul_l` / `_wcstoul_l` clamp UNDERFLOW to **1** — not 0, not ULONG_MAX. Reached
//!     by `"-4294967296"` and friends (`clamp1` tag, required).
//!   * `_wtoi64_l` has **no overflow check** and wraps where `_wcstoi64_l` saturates.
//!     Reached by 20+ digit runs (`wrapped` tag, required).
//!   * `wctob`'s `< 0xff` is strict, so **U+00FF reports EOF**. The row is EXHAUSTIVE over
//!     the whole 16-bit `wint_t` domain, so 0x00FF is not a case we might miss — but see
//!     the row itself: that particular quirk turns out to be UNFALSIFIABLE by construction,
//!     and the harness says so instead of taking credit for it.
//!   * `wcsrtombs_s_l` touches `mbstr[-1]` when `conv == size == 0`. Every buffer here has
//!     an 8-byte GUARD PREFIX that is part of the compared image, so that touch is graded
//!     rather than merely tolerated (`oob-1` tag, required).
//!   * the `_s` writers each have their own error contract (`_strlwr_s_l` sets errno by hand
//!     and fires NO handler; `_wctomb_s_l` fires one for ERANGE but not for EILSEQ;
//!     `wcsrtombs_s_l` fires one only on its final `else`). Encoded per row in `require`.
//!
//! ## Buffers are compared WHOLE, with a guard prefix
//!
//! Every destination is `[0xAA guard][payload][0xAA guard]` and the ENTIRE vector is
//! compared, not the string up to the NUL. A stray byte past the reported length — or
//! before the pointer, which `wcsrtombs_s_l` genuinely writes — is exactly what that costs
//! nothing to catch and is invisible to a `strcmp`-shaped oracle.
//!
//! ## The three locale axes, and why one binary
//!
//! Every `_l` entry point here is graded with a **NULL locale**, which is the interesting
//! case: with NULL the body resolves the AMBIENT locale, so switching the ambient one
//! actually moves both sides through different code.
//!
//!   1. the "C" locale — `lc_handle[LC_CTYPE] == 0`, the inline-ASCII path 1;
//!   2. a real NLS locale (`ref_set_nls`/`subj_set_nls`) — path 2, through `_towlower_l`
//!      and the Latin-1 tables, where a path-1-only implementation is finally observable;
//!   3. a non-zero `lc_codepage` (1252 / CP_UTF8 / CP932 + `mb_cur_max`), planted on BOTH
//!      sides, because *every* `WideCharToMultiByte` arm of the wchar->multibyte family is
//!      unreachable at codepage 0 and would otherwise ship ungraded.
//!
//! Axis 3 is why this is its own test binary: it mutates the shared locinfo, and leaving a
//! codepage behind would silently re-aim another suite's rows. It is restored at the end
//! anyway, but process isolation is the belt.
//!
//! ## One bounded, DECLARED blind spot: `I10_OUTPUT`'s `sprintf`
//!
//! `I10_OUTPUT` formats through `sprintf`. The reference's call is renamed to `ref_sprintf`
//! (msvcrt's own printf engine, from the wcs.c TU); the subject's binds to GLIBC's, because
//! `msvcrt-crt-rust` does not define `sprintf`. So that one row's substrate differs between
//! the two sides through no fault of the Rust.
//!
//! That is not waved away — it is MEASURED. `sprintf %.*le [SUBSTRATE]` fuzzes the exact
//! call `I10_OUTPUT` makes against both engines and reports where they part company; the
//! `I10_OUTPUT` row then grades only the inputs the substrate probe proves are common
//! ground, and the excluded region is printed as a number, not hidden. If the probe ever
//! goes clean the exclusion evaporates on its own.

// Force the SHIPPING crate's objects into the link. Without this, rustc drops the unused
// rlib and the shared names (wcstol, wcstoul, wctomb, wctob, wcrtomb, wcsrtombs, wcsncat)
// silently bind to GLIBC — the harness would then grade glibc against msvcrt and report a
// meaningless MATCH. `assert_shims_live()` proves the binding at run time.
extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_int, c_void};

/// `_locale_t`. Every row passes NULL: that is the case that resolves the AMBIENT locale,
/// which is what the three locale modes below actually move.
type LocT = *mut c_void;

/// `_CRT_FLOAT` — `typedef struct { float f; } _CRT_FLOAT;` (msvcrt.h:294).
#[repr(C)]
#[derive(Clone, Copy)]
struct CrtFloat {
    f: f32,
}

/// `typedef struct { UCHAR ld[10]; } MSVCRT__LDOUBLE;` (msvcrt.h:301, the `__REACTOS__`
/// arm). TEN bytes, by value — wine's `{ULONG x80[3]}` is the wrong size and is what the
/// C's own preamble copies INTO.
#[repr(C)]
#[derive(Clone, Copy)]
struct Ld80 {
    ld: [u8; 10],
}

/// `struct _I10_OUTPUT_DATA` (string.c:2665). 26 bytes; the whole thing is compared, since
/// `I10_OUTPUT` deliberately writes trailing digits PAST the NUL it just planted.
#[repr(C)]
#[derive(Clone, Copy)]
struct I10Data {
    pos: i16,
    sign: c_char,
    len: u8,
    str: [c_char; 22],
}

impl I10Data {
    fn poison() -> I10Data {
        I10Data { pos: 0x5A5A, sign: 0x5Au8 as c_char, len: 0x5A, str: [0x5Au8 as c_char; 22] }
    }
    fn img(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(26);
        v.extend_from_slice(&self.pos.to_le_bytes());
        v.push(self.sign as u8);
        v.push(self.len);
        for b in self.str.iter() {
            v.push(*b as u8);
        }
        v
    }
}

// ── the GENUINE compiled msvcrt C (build.rs: string.c + wcs.c, every public symbol
//    renamed to ref_*) ────────────────────────────────────────────────────────────────
extern "C" {
    // string.c — the `_l` integer parsers
    fn ref__strtoi64_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> i64;
    fn ref__strtoui64_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> u64;
    fn ref__strtol_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> i32;
    fn ref__strtoul_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> u32;
    fn ref__atoi_l(s: *const u8, l: LocT) -> c_int;
    fn ref__atol_l(s: *const u8, l: LocT) -> i32;
    fn ref__atoi64_l(s: *const u8, l: LocT) -> i64;
    // string.c — the `_l` comparers and case folders
    fn ref__strnicmp_l(a: *const u8, b: *const u8, n: usize, l: LocT) -> c_int;
    fn ref__stricmp_l(a: *const u8, b: *const u8, l: LocT) -> c_int;
    fn ref__memicmp_l(a: *const c_void, b: *const c_void, n: usize, l: LocT) -> c_int;
    fn ref__strlwr_s_l(s: *mut u8, n: usize, l: LocT) -> c_int;
    fn ref__strupr_s_l(s: *mut u8, n: usize, l: LocT) -> c_int;
    fn ref__strlwr_l(s: *mut u8, l: LocT) -> *mut u8;
    fn ref__strupr_l(s: *mut u8, l: LocT) -> *mut u8;
    // string.c — the three stragglers
    fn ref__strdup(s: *const u8) -> *mut u8;
    fn ref__atoflt(v: *mut CrtFloat, s: *mut u8) -> c_int;
    fn ref_I10_OUTPUT(ld: Ld80, prec: c_int, flag: c_int, data: *mut I10Data) -> c_int;

    // wcs.c — the wide integer engines and their clamps
    fn ref__wcstoi64_l(nptr: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> i64;
    fn ref__wcstoui64_l(nptr: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> u64;
    fn ref__wcstoi64(nptr: *const u16, end: *mut *mut u16, base: c_int) -> i64;
    fn ref__wcstoui64(nptr: *const u16, end: *mut *mut u16, base: c_int) -> u64;
    fn ref__wcstol_l(s: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> i32;
    fn ref__wcstoul_l(s: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> u32;
    fn ref_wcstol(s: *const u16, end: *mut *mut u16, base: c_int) -> i32;
    fn ref_wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> u32;
    fn ref__wtoi_l(s: *const u16, l: LocT) -> c_int;
    fn ref__wtol_l(s: *const u16, l: LocT) -> i32;
    fn ref__wtoi64_l(s: *const u16, l: LocT) -> i64;
    fn ref__wtoi(s: *const u16) -> c_int;
    fn ref__wtol(s: *const u16) -> i32;
    fn ref__wtoi64(s: *const u16) -> i64;
    // wcs.c — the wide case folders / comparers / concat
    fn ref_wcsncat(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn ref__wcsupr_s_l(s: *mut u16, n: usize, l: LocT) -> c_int;
    fn ref__wcsupr_l(s: *mut u16, l: LocT) -> *mut u16;
    fn ref__wcslwr_s_l(s: *mut u16, n: usize, l: LocT) -> c_int;
    fn ref__wcslwr_l(s: *mut u16, l: LocT) -> *mut u16;
    fn ref__wcsicmp_l(a: *const u16, b: *const u16, l: LocT) -> c_int;
    fn ref__wcsnicmp_l(a: *const u16, b: *const u16, n: usize, l: LocT) -> c_int;
    // wcs.c — wide -> multibyte
    fn ref_wctob(w: u16) -> c_int;
    fn ref_wctomb(dst: *mut u8, ch: u16) -> c_int;
    fn ref__wctomb_l(dst: *mut u8, ch: u16, l: LocT) -> c_int;
    fn ref__wctomb_s_l(len: *mut c_int, mb: *mut u8, size: usize, wch: u16, l: LocT) -> c_int;
    fn ref_wctomb_s(len: *mut c_int, mb: *mut u8, size: usize, wch: u16) -> c_int;
    fn ref_wcrtomb(dst: *mut u8, ch: u16, st: *mut c_int) -> usize;
    fn ref_wcrtomb_s(len: *mut usize, mb: *mut u8, size: usize, wch: u16, st: *mut c_int) -> c_int;
    fn ref_wcsrtombs(mb: *mut u8, w: *mut *const u16, count: usize, st: *mut c_int) -> usize;
    fn ref_wcsrtombs_s(
        ret: *mut usize,
        mb: *mut u8,
        size: usize,
        w: *mut *const u16,
        count: usize,
        st: *mut c_int,
    ) -> c_int;
    fn ref_wcstombs_s(ret: *mut usize, mb: *mut u8, size: usize, w: *const u16, count: usize) -> c_int;
    fn ref__wcstombs_s_l(
        ret: *mut usize,
        mb: *mut u8,
        size: usize,
        w: *const u16,
        count: usize,
        l: LocT,
    ) -> c_int;

    /// msvcrt's OWN printf engine (wcs.c), for the `I10_OUTPUT` substrate probe.
    /// Declaring a variadic extern is stable Rust; only *defining* one is not.
    fn ref_sprintf(buf: *mut u8, fmt: *const u8, ...) -> c_int;
}

// ── the SHIPPING Rust (msvcrt-crt-rust): the code under test ──────────────────────────
extern "C" {
    fn _strtoi64_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> i64;
    fn _strtoui64_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> u64;
    fn _strtol_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> i32;
    fn _strtoul_l(nptr: *const u8, end: *mut *mut u8, base: c_int, l: LocT) -> u32;
    fn _atoi_l(s: *const u8, l: LocT) -> c_int;
    fn _atol_l(s: *const u8, l: LocT) -> i32;
    fn _atoi64_l(s: *const u8, l: LocT) -> i64;
    fn _strnicmp_l(a: *const u8, b: *const u8, n: usize, l: LocT) -> c_int;
    fn _stricmp_l(a: *const u8, b: *const u8, l: LocT) -> c_int;
    fn _memicmp_l(a: *const c_void, b: *const c_void, n: usize, l: LocT) -> c_int;
    fn _strlwr_s_l(s: *mut u8, n: usize, l: LocT) -> c_int;
    fn _strupr_s_l(s: *mut u8, n: usize, l: LocT) -> c_int;
    fn _strlwr_l(s: *mut u8, l: LocT) -> *mut u8;
    fn _strupr_l(s: *mut u8, l: LocT) -> *mut u8;
    fn _strdup(s: *const u8) -> *mut u8;
    fn _atoflt(v: *mut CrtFloat, s: *mut u8) -> c_int;
    fn I10_OUTPUT(ld: Ld80, prec: c_int, flag: c_int, data: *mut I10Data) -> c_int;

    fn _wcstoi64_l(nptr: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> i64;
    fn _wcstoui64_l(nptr: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> u64;
    fn _wcstoi64(nptr: *const u16, end: *mut *mut u16, base: c_int) -> i64;
    fn _wcstoui64(nptr: *const u16, end: *mut *mut u16, base: c_int) -> u64;
    fn _wcstol_l(s: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> i32;
    fn _wcstoul_l(s: *const u16, end: *mut *mut u16, base: c_int, l: LocT) -> u32;
    fn wcstol(s: *const u16, end: *mut *mut u16, base: c_int) -> i32;
    fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> u32;
    fn _wtoi_l(s: *const u16, l: LocT) -> c_int;
    fn _wtol_l(s: *const u16, l: LocT) -> i32;
    fn _wtoi64_l(s: *const u16, l: LocT) -> i64;
    fn _wtoi(s: *const u16) -> c_int;
    fn _wtol(s: *const u16) -> i32;
    fn _wtoi64(s: *const u16) -> i64;
    fn wcsncat(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn _wcsupr_s_l(s: *mut u16, n: usize, l: LocT) -> c_int;
    fn _wcsupr_l(s: *mut u16, l: LocT) -> *mut u16;
    fn _wcslwr_s_l(s: *mut u16, n: usize, l: LocT) -> c_int;
    fn _wcslwr_l(s: *mut u16, l: LocT) -> *mut u16;
    fn _wcsicmp_l(a: *const u16, b: *const u16, l: LocT) -> c_int;
    fn _wcsnicmp_l(a: *const u16, b: *const u16, n: usize, l: LocT) -> c_int;
    fn wctob(w: u16) -> c_int;
    fn wctomb(dst: *mut u8, ch: u16) -> c_int;
    fn _wctomb_l(dst: *mut u8, ch: u16, l: LocT) -> c_int;
    fn _wctomb_s_l(len: *mut c_int, mb: *mut u8, size: usize, wch: u16, l: LocT) -> c_int;
    fn wctomb_s(len: *mut c_int, mb: *mut u8, size: usize, wch: u16) -> c_int;
    fn wcrtomb(dst: *mut u8, ch: u16, st: *mut c_int) -> usize;
    fn wcrtomb_s(len: *mut usize, mb: *mut u8, size: usize, wch: u16, st: *mut c_int) -> c_int;
    fn wcsrtombs(mb: *mut u8, w: *mut *const u16, count: usize, st: *mut c_int) -> usize;
    fn wcsrtombs_s(
        ret: *mut usize,
        mb: *mut u8,
        size: usize,
        w: *mut *const u16,
        count: usize,
        st: *mut c_int,
    ) -> c_int;
    fn wcstombs_s(ret: *mut usize, mb: *mut u8, size: usize, w: *const u16, count: usize) -> c_int;
    fn _wcstombs_s_l(
        ret: *mut usize,
        mb: *mut u8,
        size: usize,
        w: *const u16,
        count: usize,
        l: LocT,
    ) -> c_int;
}

// ── the two side channels (SEPARATE storage per side, on purpose) and the locale hooks ──
extern "C" {
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
    fn ref_set_nls(on: c_int);
    fn subj_set_nls(on: c_int);

    /// The REFERENCE's locinfo (refstub/msvcrt.h renames `get_locinfo` into the
    /// anti-circularity namespace) and the SUBJECT's (subjectimpl.c, plain name).
    fn msvcrt_ref_get_locinfo() -> *mut msvcrt_sys::threadlocaleinfostruct;
    fn get_locinfo() -> *mut msvcrt_sys::threadlocaleinfostruct;
    /// kernel32, SHARED (deliberately un-namespaced: it is environment, not code under
    /// test). Declared here only for the reachability probe below.
    fn WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        srclen: c_int,
        dst: *mut u8,
        dstlen: c_int,
        defchar: *const u8,
        used: *mut c_int,
    ) -> c_int;
    fn GetLastError() -> u32;

    fn free(p: *mut c_void);
    /// glibc's, i.e. the one the SUBJECT's `I10_OUTPUT` binds to. Declared here so the
    /// substrate probe can compare it against `ref_sprintf` directly.
    fn sprintf(buf: *mut u8, fmt: *const u8, ...) -> c_int;
}

// ── subject-side plumbing that refimpl.c keeps namespaced ─────────────────────────────
//
// These three are `msvcrt_ref_*` on the reference side (refstub/msvcrt.h), precisely so the
// reference can never bind to the code under test. The SUBJECT therefore needs its own —
// exactly the split subjectimpl.c already makes for `get_locinfo` / `_errno` / the pmt
// counter. wcs_rest.rs is the first flipped code to import them, so they land here rather
// than being invented in the C.

/// `_locale_tstruct` — `{ pthreadlocinfo locinfo; pthreadmbcinfo mbcinfo; }`.
#[repr(C)]
pub struct LocaleT {
    locinfo: *mut msvcrt_sys::threadlocaleinfostruct,
    mbcinfo: *mut c_void,
}

/// locale.c's `get_current_locale_noalloc` — fills the caller's on-stack `_locale_tstruct`
/// from thread state and takes a reference on it. Under the modelled locale there is
/// nothing to refcount, so "fill in the one locinfo this side models" is exact; it is
/// byte-for-byte what refimpl.c does for the reference, over the SUBJECT's locinfo.
#[no_mangle]
pub unsafe extern "C" fn get_current_locale_noalloc(locale: *mut LocaleT) -> *mut LocaleT {
    (*locale).locinfo = get_locinfo();
    (*locale).mbcinfo = core::ptr::null_mut();
    locale
}

/// locale.c's `free_locale_noalloc`. The C calls it unconditionally, even when the caller
/// PASSED a locale and `tmp` is still the zeroed struct — so it must tolerate that.
#[no_mangle]
pub unsafe extern "C" fn free_locale_noalloc(_locale: *mut LocaleT) {}

// (`GetLastError` for the subject lives in refimpl.c, next to the reference's, so that BOTH
// read the one `ref_last_error` the shared `WideCharToMultiByte` writes. It is harness
// plumbing every suite needs, not this row's, so it does not belong in a test file.)

// ══════════════════════════════════════════════════════════════════════════════════════
// The oracle
// ══════════════════════════════════════════════════════════════════════════════════════

/// errno is seeded with THIS, not 0, before every graded call — so a function that itself
/// CLEARS errno is observable. (classify.rs learned this the hard way: zeroing first makes
/// a zeroing write invisible, and msvcrt has such functions.)
const ERRNO_SEED: c_int = 0x5A5A;

/// The three scalar effects of a CRT call, as one comparable unit: what it returned, what
/// it left in errno, and whether the invalid-parameter handler fired. The fourth — what it
/// wrote — is `Obs::img`.
#[derive(PartialEq, Debug)]
struct Effects {
    ret: i64,
    errno: c_int,
    pmt: c_int,
}

impl Effects {
    fn subject(f: impl FnOnce() -> i64) -> Effects {
        unsafe {
            subj_errno_set(ERRNO_SEED);
            subj_invalid_pmt_reset();
            let ret = f();
            Effects { ret, errno: subj_errno_get(), pmt: subj_invalid_pmt_count() }
        }
    }
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
        let e = match self.errno {
            0 => "0".to_string(),
            ERRNO_SEED => "untouched".to_string(),
            22 => "EINVAL".to_string(),
            34 => "ERANGE".to_string(),
            42 => "EILSEQ".to_string(),
            80 => "STRUNCATE".to_string(),
            n => format!("errno{n}"),
        };
        write!(
            w,
            "ret={} ({:#x}) errno={} handler={}",
            self.ret,
            self.ret,
            e,
            if self.pmt > 0 { format!("FIRED x{}", self.pmt) } else { "no".into() }
        )
    }
}

/// One side's complete observable footprint: the scalar effects plus every byte the call
/// could have written (destination buffer WITH its guards, out-params, out-structs).
struct Obs {
    fx: Effects,
    img: Vec<u8>,
}

impl Obs {
    fn new(fx: Effects) -> Obs {
        Obs { fx, img: Vec::new() }
    }
    fn with(fx: Effects, img: Vec<u8>) -> Obs {
        Obs { fx, img }
    }
}

#[derive(PartialEq)]
enum V {
    Match,
    Diverge,
    Unexercised,
}

struct Res {
    name: String,
    v: V,
    note: String,
}

static mut RESULTS: Vec<Res> = Vec::new();
static mut CASES: u64 = 0;

fn record(name: String, v: V, note: String) {
    unsafe { (*core::ptr::addr_of_mut!(RESULTS)).push(Res { name, v, note }) }
}

/// Grade one function.
///
/// `require` names the outcome classes this row MUST reach. They are counted from the
/// REFERENCE's answer, so the requirement is a claim about msvcrt's contract, not about
/// our implementation — and a row that reports MATCH having never once run the branch it
/// exists to test fails as UNEXERCISED instead of passing quietly.
fn grade(
    name: &str,
    require: &[&'static str],
    iters: usize,
    mut f: impl FnMut(&mut Rng) -> (Obs, Obs, String, Vec<&'static str>),
) {
    // Seeded from the row name so every row gets its own stream and a row's corpus does
    // not shift when a neighbouring row's iteration count changes.
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for b in name.bytes() {
        seed = seed.wrapping_mul(0x100_0000_01B3).wrapping_add(b as u64);
    }
    let mut r = Rng(seed | 1);
    let mut seen: Vec<&'static str> = Vec::new();
    for _ in 0..iters {
        let (got, want, ctx, tags) = f(&mut r);
        unsafe { CASES += 1 };
        for t in tags {
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
        if got.fx != want.fx || got.img != want.img {
            let mut note = format!("[{ctx}]\n        rust  = {}", got.fx);
            if got.img != want.img {
                note.push_str(&format!("\n                img={}", hex(&got.img)));
            }
            note.push_str(&format!("\n        msvcrt= {}", want.fx));
            if got.img != want.img {
                note.push_str(&format!("\n                img={}", hex(&want.img)));
                if let Some(i) = first_diff(&got.img, &want.img) {
                    note.push_str(&format!("\n        first byte differing: index {i} (0 = the guard byte 8 BEFORE the pointer)"));
                }
            }
            record(name.to_string(), V::Diverge, note);
            return;
        }
    }
    let missing: Vec<&&str> = require.iter().filter(|w| !seen.contains(w)).collect();
    if !missing.is_empty() {
        record(
            name.to_string(),
            V::Unexercised,
            format!(
                "the fuzz never reached {missing:?} — this MATCH is worthless, the branch \
                 never ran.\n        reached: {seen:?}"
            ),
        );
        return;
    }
    record(name.to_string(), V::Match, format!("{iters} cases; reached {seen:?}"));
}

fn hex(v: &[u8]) -> String {
    let mut s = String::new();
    for (i, b) in v.iter().enumerate() {
        if i == GUARD || (GUARD > 0 && i > 0 && i % 32 == 0) {
            s.push('|');
        }
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    for i in 0..a.len().min(b.len()) {
        if a[i] != b[i] {
            return Some(i);
        }
    }
    if a.len() != b.len() {
        Some(a.len().min(b.len()))
    } else {
        None
    }
}

// ── buffers with guards ───────────────────────────────────────────────────────────────

/// Bytes of untouchable poison on EACH side of every destination. `wcsrtombs_s_l` writes
/// `mbstr[-1]` for real, so the leading guard is not paranoia — it is the only place that
/// write is visible, and the image comparison grades it.
const GUARD: usize = 8;

struct Buf(Vec<u8>);
impl Buf {
    fn new(n: usize) -> Buf {
        Buf(vec![0xAA; GUARD + n + GUARD])
    }
    fn seeded(src: &[u8], n: usize) -> Buf {
        let mut b = Buf::new(n);
        b.0[GUARD..GUARD + src.len()].copy_from_slice(src);
        b
    }
    fn p(&mut self) -> *mut u8 {
        unsafe { self.0.as_mut_ptr().add(GUARD) }
    }
    fn img(self) -> Vec<u8> {
        self.0
    }
}

struct WBuf(Vec<u16>);
impl WBuf {
    fn new(n: usize) -> WBuf {
        WBuf(vec![0xAAAA; GUARD + n + GUARD])
    }
    fn seeded(src: &[u16], n: usize) -> WBuf {
        let mut b = WBuf::new(n);
        b.0[GUARD..GUARD + src.len()].copy_from_slice(src);
        b
    }
    fn p(&mut self) -> *mut u16 {
        unsafe { self.0.as_mut_ptr().add(GUARD) }
    }
    fn img(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.0.len() * 2);
        for w in self.0 {
            v.extend_from_slice(&w.to_le_bytes());
        }
        v
    }
}

fn push_i64(img: &mut Vec<u8>, v: i64) {
    img.extend_from_slice(&v.to_le_bytes());
}

/// Where an `endptr` ended up, as a value both sides can be compared on.
///
/// Seeded with a sentinel rather than NULL, because "the C returned BEFORE writing
/// `*endptr`" is itself part of the contract (`_strtoi64_l`'s three front-door checks all
/// do) and is indistinguishable from "wrote the start" if the seed is the start.
const ENDPTR_SENTINEL: usize = 1;
fn endptr_code<T>(e: *mut T, base: *const T) -> i64 {
    if e as usize == ENDPTR_SENTINEL {
        -1 // never written
    } else if base.is_null() {
        // a NULL nptr that still got an endptr write: report the absolute value, which is
        // 0 when the C wrote back the (NULL) nptr.
        e as i64
    } else {
        (e as isize - base as isize) as i64
    }
}

// ── deterministic RNG ─────────────────────────────────────────────────────────────────
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 17
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n.max(1)
    }
}

// ── generators ────────────────────────────────────────────────────────────────────────

/// Narrow alphabet, as classify's: letters (case fold), digits (conversion), high bytes
/// (the `_tolower_l` sign-extension → EILSEQ path), whitespace and signs.
fn gen_str(r: &mut Rng, maxlen: usize) -> Vec<u8> {
    let n = r.below(maxlen);
    let mut v = Vec::with_capacity(n + 1);
    for _ in 0..n {
        let b = match r.below(6) {
            0 => b'a' + (r.below(26) as u8),
            1 => b'A' + (r.below(26) as u8),
            2 => b'0' + (r.below(10) as u8),
            3 => 0x80u8.wrapping_add(r.below(0x80) as u8),
            4 => b" \t+-.xX"[r.below(7)],
            _ => 1 + (r.below(254) as u8),
        };
        v.push(b);
    }
    v.push(0);
    v
}

/// The boundary numerals. A general string fuzzer essentially never emits a 19-digit run,
/// so every saturate/clamp arm would go untested — including the two this wave exists to
/// pin: `_strtoul_l`'s underflow-to-**1** and `_wtoi64_l`'s missing overflow check.
fn gen_num_core(r: &mut Rng) -> Vec<u8> {
    match r.below(14) {
        0 => b"9223372036854775807".to_vec(),  // I64_MAX exactly
        1 => b"9223372036854775808".to_vec(),  // I64_MAX + 1     -> saturate + ERANGE
        2 => b"-9223372036854775808".to_vec(), // I64_MIN exactly
        3 => b"-9223372036854775809".to_vec(), // I64_MIN - 1     -> saturate + ERANGE
        4 => b"4294967295".to_vec(),           // ULONG_MAX
        5 => b"4294967296".to_vec(),           // ULONG_MAX + 1   -> _strtoul_l clamps high
        6 => b"-4294967296".to_vec(),          // < -ULONG_MAX    -> _strtoul_l returns *1*
        7 => b"-4294967295".to_vec(),          // == -ULONG_MAX   -> falls THROUGH (== 1 boundary)
        8 => b"2147483648".to_vec(),           // LONG_MAX + 1
        9 => b"-2147483649".to_vec(),          // LONG_MIN - 1
        10 => b"18446744073709551615".to_vec(), // UI64_MAX
        11 => b"99999999999999999999".to_vec(), // 20 digits: _wtoi64_l WRAPS, _wcstoi64_l saturates
        12 => {
            let n = 15 + r.below(12);
            let mut v = Vec::new();
            if r.below(2) == 0 {
                v.push(b'-');
            }
            for _ in 0..n {
                v.push(b'0' + r.below(10) as u8);
            }
            v
        }
        _ => {
            let pre: &[u8] = [&b"0x"[..], &b"0X"[..], &b"0"[..], &b""[..]][r.below(4)];
            let n = 1 + r.below(18);
            let mut v = pre.to_vec();
            for _ in 0..n {
                v.push(b"0123456789abcdefABCDEF"[r.below(22)]);
            }
            v
        }
    }
}

fn gen_num_str(r: &mut Rng) -> Vec<u8> {
    let core = gen_num_core(r);
    let mut v = Vec::new();
    for _ in 0..r.below(3) {
        v.push(b" \t\n\x0b\x0c\r"[r.below(6)]);
    }
    v.extend_from_slice(&core);
    if r.below(3) == 0 {
        v.extend_from_slice(b"zz!");
    }
    v.push(0);
    v
}

fn gen_parse_input(r: &mut Rng) -> Vec<u8> {
    if r.below(2) == 0 {
        gen_num_str(r)
    } else {
        gen_str(r, 22)
    }
}

/// The wide alphabet. Weighted toward the top bit (a signed/unsigned element compare would
/// diverge there), the 0x80..0xFF byte range, and — for the parsers — the seventeen
/// non-ASCII digit families `wctoint`'s MAP_FOLDDIGITS table folds.
const FOLD_ZEROS: [u16; 17] = [
    0x660, 0x6f0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xc66, 0xce6, 0xd66, 0xe50, 0xed0, 0xf20,
    0x1040, 0x17e0, 0x1810, 0xff10,
];

fn gen_wstr(r: &mut Rng, maxlen: usize) -> Vec<u16> {
    let n = r.below(maxlen);
    let mut v = Vec::with_capacity(n + 1);
    for _ in 0..n {
        let w = match r.below(8) {
            0 => (b'a' + (r.below(26) as u8)) as u16,
            1 => (b'A' + (r.below(26) as u8)) as u16,
            2 => (b'0' + (r.below(10) as u8)) as u16,
            3 => 0x80u16 + (r.below(0x80) as u16),
            4 => 0x8000u16 + (r.below(0x8000) as u16),
            5 => FOLD_ZEROS[r.below(17)] + (r.below(11) as u16),
            _ => 1 + (r.below(0xFFFE) as u16),
        };
        v.push(w);
    }
    v.push(0);
    v
}

fn widen(s: &[u8]) -> Vec<u16> {
    s.iter().map(|b| *b as u16).collect()
}

fn gen_wnum_str(r: &mut Rng) -> Vec<u16> {
    if r.below(4) == 0 {
        // the FOLDDIGITS path: an ASCII-digit string with some digits swapped for their
        // Arabic-Indic / Devanagari / fullwidth twins, which `wctoint` folds and nothing
        // else in msvcrt does.
        let z = FOLD_ZEROS[r.below(17)];
        let n = 1 + r.below(22);
        let mut v: Vec<u16> = Vec::new();
        if r.below(3) == 0 {
            v.push(b'-' as u16);
        }
        for _ in 0..n {
            let d = r.below(10) as u16;
            v.push(if r.below(2) == 0 { z + d } else { b'0' as u16 + d });
        }
        v.push(0);
        v
    } else if r.below(2) == 0 {
        widen(&gen_num_str(r))
    } else {
        gen_wstr(r, 22)
    }
}

fn gen_wparse_input(r: &mut Rng) -> Vec<u16> {
    gen_wnum_str(r)
}

/// Float literals: the shapes that separate a naive accumulator from a correctly-rounded
/// one, plus the exact `_atoflt` classification boundaries (FLT_MAX / FLT_MIN).
fn gen_float_str(r: &mut Rng) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    match r.below(12) {
        0 => v.extend_from_slice(b"1e40"),      // > FLT_MAX  -> _OVERFLOW
        1 => v.extend_from_slice(b"-1e40"),     // < -FLT_MAX -> _OVERFLOW
        2 => v.extend_from_slice(b"1e-50"),     // < FLT_MIN  -> _UNDERFLOW
        3 => v.extend_from_slice(b"-1e-50"),    // _UNDERFLOW, negative
        4 => v.extend_from_slice(b"3.4028235e38"),  // ~FLT_MAX
        5 => v.extend_from_slice(b"1.17549435e-38"), // ~FLT_MIN
        6 => v.extend_from_slice(b"0"),         // exactly zero -> NOT underflow (d==0, err==0)
        7 => v.extend_from_slice(b"1e400"),     // overflows the DOUBLE too
        8 => v.extend_from_slice(b"1e-400"),    // underflows the double
        9 => {
            for _ in 0..1 + r.below(20) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.push(b'.');
            for _ in 0..1 + r.below(20) {
                v.push(b'0' + r.below(10) as u8);
            }
            if r.below(2) == 0 {
                v.push(b'e');
                if r.below(2) == 0 {
                    v.push(b'-');
                }
                v.extend_from_slice(format!("{}", r.below(60)).as_bytes());
            }
        }
        10 => {
            v.extend_from_slice(b"  ");
            v.push(if r.below(2) == 0 { b'-' } else { b'+' });
            for _ in 0..1 + r.below(8) {
                v.push(b'0' + r.below(10) as u8);
            }
            v.extend_from_slice(b"xyz");
        }
        _ => return gen_str(r, 20),
    }
    v.push(0);
    v
}

// ── x87 80-bit patterns for I10_OUTPUT ────────────────────────────────────────────────

/// A finite `f64` as the x87 80-bit extended pattern msvcrt's `%Lf` callers pass.
fn ld80_from_f64(d: f64) -> Ld80 {
    let bits = d.to_bits();
    let sign = ((bits >> 63) & 1) as u16;
    let exp11 = ((bits >> 52) & 0x7FF) as u16;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let (exp15, mant): (u16, u64) = if exp11 == 0 {
        if frac == 0 {
            (0, 0) // +-0.0
        } else {
            // Subnormal double: renormalise into the explicit-integer-bit format.
            let lz = frac.leading_zeros() - 11;
            (
                (1u16.wrapping_sub(1023).wrapping_add(16383)).wrapping_sub(lz as u16),
                (frac << (11 + lz + 1)) | (1u64 << 63),
            )
        }
    } else {
        ((exp11 - 1023 + 16383) & 0x7FFF, (1u64 << 63) | (frac << 11))
    };
    let mut ld = [0u8; 10];
    ld[..8].copy_from_slice(&mant.to_le_bytes());
    ld[8..].copy_from_slice(&((sign << 15) | exp15).to_le_bytes());
    Ld80 { ld }
}

fn ld80_raw(exp_and_sign: u16, mant: u64) -> Ld80 {
    let mut ld = [0u8; 10];
    ld[..8].copy_from_slice(&mant.to_le_bytes());
    ld[8..].copy_from_slice(&exp_and_sign.to_le_bytes());
    Ld80 { ld }
}

/// The `x80[2]` word (sign + 15-bit exponent) of a pattern — what the C branches on.
fn ld80_expword(l: &Ld80) -> u16 {
    u16::from_le_bytes([l.ld[8], l.ld[9]])
}

fn ld80_mant(l: &Ld80) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&l.ld[..8]);
    u64::from_le_bytes(b)
}

/// The value `I10_OUTPUT`'s `fpnum_double` will hand to `sprintf`, modelled here ONLY to
/// decide whether the substrate probe covers the case. It is never compared against
/// anything — the graded answer always comes from the two real implementations.
fn ld80_as_f64(l: &Ld80) -> f64 {
    let w = ld80_expword(l);
    let m = ld80_mant(l);
    let sign = if (w & 0x8000) != 0 { -1.0f64 } else { 1.0 };
    let e = (w & 0x7FFF) as i32 - 0x3FFF - 63;
    // m * 2^e, computed with enough headroom to see the overflow to infinity.
    sign * (m as f64) * (2.0f64).powi(e)
}

/// Would `I10_OUTPUT` reach `sprintf` with a `prec` outside 0..=20?
///
/// **This is a live, unbounded stack write in msvcrt, not a harness limitation.** The
/// clamp is written `if (prec+1 > I10_OUTPUT_MAX_PREC) prec = MAX-1; else if (prec < 0)
/// { d = 0.0; prec = 0; }` — and when `prec` arrives as INT_MAX, `prec+1` wraps NEGATIVE,
/// so neither arm fires. `sprintf(format, "%%.%dle", prec)` then writes `"%.2147483647le"`
/// (15 bytes) into `char format[8]`, and the resulting conversion pads two gigabytes of
/// spaces into `char buf[30]`. Both stacks are gone.
///
/// It is reachable: with `flag & 1` the code does `prec += (int)(1 + floor(log10(d)))`, and
/// for `d == 0` (or an infinity) that addend is INT_MIN — x86's `cvttsd2si` answer for a
/// non-finite double, which Rust's saturating `as` matches. `prec == 1` then lands exactly
/// on INT_MAX. Every other input `prec` lands on a value one of the two arms catches.
///
/// The Rust reproduces this faithfully, which is correct for a differential port — so the
/// corpus skips it and the count is reported, rather than the row being quietly narrowed.
fn i10_safe(ld: &Ld80, prec: c_int, flag: c_int) -> bool {
    if (ld80_expword(ld) & 0x7FFF) == 0x7FFF {
        return true; // the 1#INF / 1#QNAN / 1#SNAN arm returns before any of this
    }
    let d = ld80_as_f64(ld).abs();
    let mut p = prec;
    if (flag & 1) != 0 {
        // Conservative: anything `fpnum_double` might flush to zero, or overflow to
        // infinity, is treated as the non-finite `log10` case.
        let exp = if d == 0.0 || !d.is_finite() || d < 1e-300 || d > 1e300 {
            i32::MIN
        } else {
            (1.0 + d.log10().floor()) as i32
        };
        p = p.wrapping_add(exp);
        if exp < 0 {
            p = p.wrapping_sub(1);
        }
    }
    p = p.wrapping_sub(1);
    if p.wrapping_add(1) > 21 {
        p = 20;
    } else if p < 0 {
        p = 0;
    }
    (0..=20).contains(&p)
}

fn gen_ld80(r: &mut Rng) -> Ld80 {
    match r.below(10) {
        0 => ld80_raw(0x7FFF, 0x8000_0000_0000_0000), // +INF   ("1#INF")
        1 => ld80_raw(0xFFFF, 0x8000_0000_0000_0000), // -INF
        2 => ld80_raw(0x7FFF, 0xC000_0000_0000_0000), // QNaN   ("1#QNAN")
        3 => ld80_raw(0x7FFF, 0xA000_0000_0000_0000), // SNaN   ("1#SNAN")
        4 => ld80_raw(if r.below(2) == 0 { 0 } else { 0x8000 }, r.next()), // exponent field 0
        5 => ld80_from_f64(0.0),
        6 => {
            let mant = r.next() | (1u64 << 63);
            let exp = (0x3FFF - 40 + r.below(80)) as u16;
            ld80_raw(exp | if r.below(2) == 0 { 0 } else { 0x8000 }, mant)
        }
        7 => ld80_from_f64({
            let m = (r.next() % 1_000_000_000) as f64;
            let e = (r.below(40) as i32) - 20;
            m * (10.0f64).powi(e)
        }),
        8 => ld80_raw((r.next() & 0xFFFF) as u16, r.next()), // fully random
        _ => ld80_from_f64(f64::from_bits(r.next())),
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Link proof
// ══════════════════════════════════════════════════════════════════════════════════════

/// Prove the shared-name shims resolve to MSVCRT-CRT-RUST and not to GLIBC.
///
/// Seven of the names under test — `wcstol`, `wcstoul`, `wctomb`, `wctob`, `wcrtomb`,
/// `wcsrtombs`, `wcsncat` — are also glibc exports. If any bound to glibc the row would
/// compare glibc against msvcrt and report a verdict about nothing. Each check below picks
/// a case where the two are OBSERVABLY different, so this is a proof and not a smoke test.
unsafe fn assert_shims_live() {
    // 1. glibc's wchar_t is 32-bit. A 16-bit L"12" is bytes `31 00 32 00 00 00`, which
    //    glibc reads as the single character U+00320031 — it cannot answer 12.
    let twelve = [b'1' as u16, b'2' as u16, 0];
    assert_eq!(
        wcstol(twelve.as_ptr(), core::ptr::null_mut(), 10),
        12,
        "wcstol(L\"12\") != 12 — the wide shims are NOT msvcrt-crt-rust's (16-bit wchar_t)"
    );
    assert_eq!(
        wcstoul(twelve.as_ptr(), core::ptr::null_mut(), 10),
        12,
        "wcstoul(L\"12\") != 12 — bound to glibc"
    );

    // 2. msvcrt's front door fires the invalid-parameter handler on a NULL nptr and
    //    returns 0; glibc's would fault. This is also the proof that the handler counter
    //    is wired to the SUBJECT's storage.
    subj_invalid_pmt_reset();
    subj_errno_set(0);
    assert_eq!(wcstol(core::ptr::null(), core::ptr::null_mut(), 10), 0);
    assert_eq!(
        subj_invalid_pmt_count(),
        1,
        "wcstol(NULL) did not fire the invalid-parameter handler — wrong binding"
    );
    assert_eq!(subj_errno_get(), 22, "wcstol(NULL) did not set EINVAL");

    // 3. `wctob` uses a STRICT `< 0xff`, and in the raw codepage returns the value as a
    //    SIGNED char. glibc's C locale answers EOF for everything above 0x7F.
    assert_eq!(wctob(0xFE), -2, "wctob(0xFE) != (signed char)0xFE — bound to glibc");
    assert_eq!(wctob(0xFF), -1, "wctob(0xFF) != EOF — the strict `< 0xff` is gone");

    // 4. `wctomb` of a character above 0xFF sets EILSEQ in the SUBJECT's errno and
    //    returns -1 in the raw codepage.
    let mut dst = [0u8; 8];
    subj_errno_set(0);
    assert_eq!(wctomb(dst.as_mut_ptr(), 0x0100), -1, "wctomb(U+0100) != -1");
    assert_eq!(subj_errno_get(), 42, "wctomb(U+0100) did not set EILSEQ");

    // 5. `wcsrtombs` reports EILSEQ through the same channel.
    let hi = [0x0100u16, 0];
    let mut p: *const u16 = hi.as_ptr();
    subj_errno_set(0);
    let mut mb = [0u8; 8];
    assert_eq!(
        wcsrtombs(mb.as_mut_ptr(), &mut p, 4, core::ptr::null_mut()),
        usize::MAX,
        "wcsrtombs did not return (size_t)-1"
    );
    assert_eq!(subj_errno_get(), 42, "wcsrtombs did not set EILSEQ");

    // 6. `wcsncat` on a 16-bit `wchar_t`. Both buffers are laid out so that the 32-bit
    //    reading is also a valid, non-faulting string — and lands the appended character
    //    two u16 slots along instead of one. So the result says which width ran.
    let mut d = [0x0041u16, 0, 0, 0, 0xBBBB, 0xBBBB, 0xBBBB, 0xBBBB];
    let s = [0x0042u16, 0, 0, 0];
    wcsncat(d.as_mut_ptr(), s.as_ptr(), 4);
    assert_eq!(
        &d[..4],
        &[0x0041u16, 0x0042, 0x0000, 0x0000],
        "wcsncat appended at the wrong element — a 32-bit wchar_t ran, i.e. glibc's"
    );

    // 7. `wcrtomb` returns msvcrt's `wctomb` result widened, i.e. (size_t)-1 on failure.
    subj_errno_set(0);
    assert_eq!(
        wcrtomb(dst.as_mut_ptr(), 0x0100, core::ptr::null_mut()),
        usize::MAX,
        "wcrtomb(U+0100) != (size_t)-1"
    );

    // 8. The two locinfos must be the modelled "C" locale and must be READABLE through
    //    msvcrt-sys's layout. If bindgen's view disagreed with the C's, every codepage
    //    plant below would land on some other field and both sides would keep answering
    //    from an unchanged locale — a green from an untouched branch.
    for (who, li) in [("subject", get_locinfo()), ("reference", msvcrt_ref_get_locinfo())] {
        assert_eq!((*li).lc_codepage, 0, "{who} locinfo.lc_codepage != 0 — bad layout");
        assert_eq!((*li).mb_cur_max, 1, "{who} locinfo.mb_cur_max != 1 — bad layout");
        assert!(!(*li).lconv.is_null(), "{who} locinfo.lconv is NULL — bad layout");
        assert!(!(*li).pcumap.is_null(), "{who} locinfo.pcumap is NULL — bad layout");
    }

    println!("  [self-check] msvcrt-crt-rust's wide shims are live (wcstol/wcstoul/wctob/");
    println!("               wctomb/wcrtomb/wcsrtombs/wcsncat all answer msvcrt's contract,");
    println!("               not glibc's) and both locinfos read as the modelled C locale.");
}

/// Plant a codepage (and `mb_cur_max`) on BOTH sides at once.
///
/// Every `WideCharToMultiByte` arm of the wchar->multibyte family is dead at codepage 0,
/// which is what the harness has always pinned — so without this the ENTIRE non-raw half of
/// wave 16's converter family ships ungraded. `WideCharToMultiByte` itself is deliberately
/// un-namespaced (kernel32 is environment, not code under test), so both sides reach the
/// same one and the rows isolate the Rust's logic AROUND the call.
unsafe fn set_codepage(cp: u32, mb_cur_max: c_int) {
    for li in [get_locinfo(), msvcrt_ref_get_locinfo()] {
        (*li).lc_codepage = cp;
        (*li).mb_cur_max = mb_cur_max;
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The rows
// ══════════════════════════════════════════════════════════════════════════════════════

/// `EOF`, as `wctob` returns it.
const EOF_C: c_int = -1;

const BASES: [c_int; 9] = [0, 2, 8, 10, 16, 36, /* invalid: */ 1, 37, -1];
const PARSE_ITERS: usize = 20_000;
const CASE_ITERS: usize = 12_000;

/// Tag an integer-parser outcome from the REFERENCE's effects.
fn parse_tags(fx: &Effects) -> Vec<&'static str> {
    let mut t = vec![];
    if fx.pmt > 0 {
        t.push("front-door");
    }
    if fx.errno == 34 {
        t.push("ERANGE");
    }
    if fx.errno == 42 {
        t.push("EILSEQ");
    }
    if fx.errno == ERRNO_SEED && fx.pmt == 0 {
        t.push("clean");
    }
    t
}

macro_rules! narrow_parse_rows {
    ($suffix:literal) => {
        // ── _strtoi64_l: the engine every narrow clamp below runs on ────────────────
        grade(
            concat!("_strtoi64_l", $suffix),
            &["front-door", "ERANGE", "EILSEQ", "clean"],
            PARSE_ITERS,
            |r| {
                let a = gen_parse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let pass_end = r.below(4) != 0;
                let (mut e1, mut e2) =
                    (ENDPTR_SENTINEL as *mut u8, ENDPTR_SENTINEL as *mut u8);
                let (pe1, pe2) = if pass_end {
                    (&mut e1 as *mut *mut u8, &mut e2 as *mut *mut u8)
                } else {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                };
                let g = Effects::subject(|| unsafe { _strtoi64_l(p, pe1, base, core::ptr::null_mut()) });
                let w = Effects::reference(|| unsafe { ref__strtoi64_l(p, pe2, base, core::ptr::null_mut()) });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let tags = parse_tags(&w);
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("base={base} end={pass_end} {}", show(&a, null)),
                    tags,
                )
            },
        );
        // ── _strtoui64_l: a SEPARATE engine (magnitude + final negate, RAW digit byte) ──
        grade(
            concat!("_strtoui64_l", $suffix),
            &["front-door", "ERANGE", "EILSEQ", "clean"],
            PARSE_ITERS,
            |r| {
                let a = gen_parse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let pass_end = r.below(4) != 0;
                let (mut e1, mut e2) =
                    (ENDPTR_SENTINEL as *mut u8, ENDPTR_SENTINEL as *mut u8);
                let (pe1, pe2) = if pass_end {
                    (&mut e1 as *mut *mut u8, &mut e2 as *mut *mut u8)
                } else {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                };
                let g = Effects::subject(|| unsafe {
                    _strtoui64_l(p, pe1, base, core::ptr::null_mut()) as i64
                });
                let w = Effects::reference(|| unsafe {
                    ref__strtoui64_l(p, pe2, base, core::ptr::null_mut()) as i64
                });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let tags = parse_tags(&w);
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("base={base} end={pass_end} {}", show(&a, null)),
                    tags,
                )
            },
        );
        // ── _strtol_l ───────────────────────────────────────────────────────────────
        grade(
            concat!("_strtol_l", $suffix),
            &["front-door", "ERANGE", "clean"],
            PARSE_ITERS,
            |r| {
                let a = gen_parse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let (mut e1, mut e2) =
                    (ENDPTR_SENTINEL as *mut u8, ENDPTR_SENTINEL as *mut u8);
                let g = Effects::subject(|| unsafe {
                    _strtol_l(p, &mut e1, base, core::ptr::null_mut()) as i64
                });
                let w = Effects::reference(|| unsafe {
                    ref__strtol_l(p, &mut e2, base, core::ptr::null_mut()) as i64
                });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let tags = parse_tags(&w);
                (Obs::with(g, gi), Obs::with(w, wi), format!("base={base} {}", show(&a, null)), tags)
            },
        );
        // ── _strtoul_l — THE underflow-clamps-to-1 row ──────────────────────────────
        //
        // `ret < -(__int64)ULONG_MAX` returns **1**. Not 0, not ULONG_MAX. `clamp1` is a
        // required outcome, so a corpus that stops emitting `"-4294967296"` fails the row
        // instead of silently un-testing the quirk.
        grade(
            concat!("_strtoul_l", $suffix),
            &["front-door", "ERANGE", "clean", "clamp1", "clamp-high"],
            PARSE_ITERS,
            |r| {
                let a = gen_parse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let (mut e1, mut e2) =
                    (ENDPTR_SENTINEL as *mut u8, ENDPTR_SENTINEL as *mut u8);
                let g = Effects::subject(|| unsafe {
                    _strtoul_l(p, &mut e1, base, core::ptr::null_mut()) as i64
                });
                let w = Effects::reference(|| unsafe {
                    ref__strtoul_l(p, &mut e2, base, core::ptr::null_mut()) as i64
                });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let mut tags = parse_tags(&w);
                if w.ret == 1 && w.errno == 34 {
                    tags.push("clamp1");
                }
                if w.ret == 0xFFFF_FFFF && w.errno == 34 {
                    tags.push("clamp-high");
                }
                (Obs::with(g, gi), Obs::with(w, wi), format!("base={base} {}", show(&a, null)), tags)
            },
        );
        // ── the three atoi-shaped clamps ────────────────────────────────────────────
        grade(concat!("_atoi_l", $suffix), &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
            let a = gen_parse_input(r);
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let g = Effects::subject(|| unsafe { _atoi_l(p, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__atoi_l(p, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), show(&a, null), tags)
        });
        grade(concat!("_atol_l", $suffix), &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
            let a = gen_parse_input(r);
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let g = Effects::subject(|| unsafe { _atol_l(p, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__atol_l(p, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), show(&a, null), tags)
        });
        grade(concat!("_atoi64_l", $suffix), &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
            let a = gen_parse_input(r);
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let g = Effects::subject(|| unsafe { _atoi64_l(p, core::ptr::null_mut()) });
            let w = Effects::reference(|| unsafe { ref__atoi64_l(p, core::ptr::null_mut()) });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), show(&a, null), tags)
        });
    };
}

macro_rules! narrow_case_rows {
    ($suffix:literal) => {
        // `count == 0` returns 0 BEFORE the front door, so `_strnicmp_l(NULL, NULL, 0)`
        // is a silent 0 — a separate outcome from the NULL rejection, and required.
        grade(
            concat!("_strnicmp_l", $suffix),
            &["clean", "front-door", "zero-count"],
            CASE_ITERS,
            |r| {
                let (a, b) = (gen_str(r, 20), gen_str(r, 20));
                let n = match r.below(8) {
                    0 => 0,
                    1 => c_int::MAX as usize,
                    _ => r.below(24),
                };
                let na = r.below(16) == 0;
                let nb = r.below(16) == 0;
                let (pa, pb) = (
                    if na { core::ptr::null() } else { a.as_ptr() },
                    if nb { core::ptr::null() } else { b.as_ptr() },
                );
                let g = Effects::subject(|| unsafe { _strnicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__strnicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let mut tags = parse_tags(&w);
                if n == 0 {
                    tags.push("zero-count");
                }
                (Obs::new(g), Obs::new(w), format!("n={n} {} vs {}", show(&a, na), show(&b, nb)), tags)
            },
        );
        grade(concat!("_stricmp_l", $suffix), &["clean", "front-door"], CASE_ITERS, |r| {
            let (a, b) = (gen_str(r, 20), gen_str(r, 20));
            let na = r.below(16) == 0;
            let nb = r.below(16) == 0;
            let (pa, pb) = (
                if na { core::ptr::null() } else { a.as_ptr() },
                if nb { core::ptr::null() } else { b.as_ptr() },
            );
            let g = Effects::subject(|| unsafe { _stricmp_l(pa, pb, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__stricmp_l(pa, pb, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), format!("{} vs {}", show(&a, na), show(&b, nb)), tags)
        });
        // `_memicmp_l`'s NULL guard is hand-rolled, fires the handler ONLY when len != 0,
        // and it folds a SIGNED char through `_tolower_l` — so bytes >= 0x80 both change
        // the answer and set EILSEQ, unlike `_strnicmp_l`'s `(unsigned char)` cast.
        grade(
            concat!("_memicmp_l", $suffix),
            &["clean", "front-door", "zero-len", "EILSEQ"],
            CASE_ITERS,
            |r| {
                let a = gen_str(r, 24);
                let mut b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 24) };
                if r.below(3) == 0 && !b.is_empty() {
                    let i = r.below(b.len());
                    b[i] = b[i].wrapping_add(1);
                }
                let n = match r.below(6) {
                    0 => 0,
                    _ => r.below(a.len().min(b.len()) + 1),
                };
                let na = r.below(16) == 0;
                let nb = r.below(16) == 0;
                let (pa, pb) = (
                    if na { core::ptr::null() } else { a.as_ptr() as *const c_void },
                    if nb { core::ptr::null() } else { b.as_ptr() as *const c_void },
                );
                let g = Effects::subject(|| unsafe { _memicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__memicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let mut tags = parse_tags(&w);
                if n == 0 {
                    tags.push("zero-len");
                }
                (Obs::new(g), Obs::new(w), format!("n={n} na={na} nb={nb} {a:?} vs {b:?}"), tags)
            },
        );
        // The `_s` folders: EINVAL by hand (NO handler), and — the part a NUL-terminated
        // compare cannot see — `str[0] = 0` on the no-terminator-inside-len path while the
        // REST of the buffer is left exactly as it was.
        grade(
            concat!("_strlwr_s_l", $suffix),
            &["ok", "EINVAL", "no-handler"],
            CASE_ITERS,
            |r| {
                let s = gen_str(r, 24);
                let len = match r.below(6) {
                    0 => 0,
                    1 => usize::MAX,
                    _ => r.below(28),
                };
                let null = r.below(24) == 0;
                let (mut d1, mut d2) = (Buf::seeded(&s, 40), Buf::seeded(&s, 40));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let g = Effects::subject(|| unsafe { _strlwr_s_l(q1, len, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__strlwr_s_l(q2, len, core::ptr::null_mut()) as i64 });
                let tags = sec_tags(&w);
                (
                    Obs::with(g, d1.img()),
                    Obs::with(w, d2.img()),
                    format!("len={len} null={null} slen={}", s.len() - 1),
                    tags,
                )
            },
        );
        grade(
            concat!("_strupr_s_l", $suffix),
            &["ok", "EINVAL", "no-handler"],
            CASE_ITERS,
            |r| {
                let s = gen_str(r, 24);
                let len = match r.below(6) {
                    0 => 0,
                    1 => usize::MAX,
                    _ => r.below(28),
                };
                let null = r.below(24) == 0;
                let (mut d1, mut d2) = (Buf::seeded(&s, 40), Buf::seeded(&s, 40));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let g = Effects::subject(|| unsafe { _strupr_s_l(q1, len, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__strupr_s_l(q2, len, core::ptr::null_mut()) as i64 });
                let tags = sec_tags(&w);
                (
                    Obs::with(g, d1.img()),
                    Obs::with(w, d2.img()),
                    format!("len={len} null={null} slen={}", s.len() - 1),
                    tags,
                )
            },
        );
        grade(concat!("_strlwr_l", $suffix), &["ok"], CASE_ITERS, |r| {
            let s = gen_str(r, 30);
            let (mut d1, mut d2) = (Buf::seeded(&s, 40), Buf::seeded(&s, 40));
            let (p1, p2) = (d1.p(), d2.p());
            let g = Effects::subject(|| unsafe { (_strlwr_l(p1, core::ptr::null_mut()) == p1) as i64 });
            let w = Effects::reference(|| unsafe { (ref__strlwr_l(p2, core::ptr::null_mut()) == p2) as i64 });
            (Obs::with(g, d1.img()), Obs::with(w, d2.img()), format!("{s:?}"), vec!["ok"])
        });
        grade(concat!("_strupr_l", $suffix), &["ok"], CASE_ITERS, |r| {
            let s = gen_str(r, 30);
            let (mut d1, mut d2) = (Buf::seeded(&s, 40), Buf::seeded(&s, 40));
            let (p1, p2) = (d1.p(), d2.p());
            let g = Effects::subject(|| unsafe { (_strupr_l(p1, core::ptr::null_mut()) == p1) as i64 });
            let w = Effects::reference(|| unsafe { (ref__strupr_l(p2, core::ptr::null_mut()) == p2) as i64 });
            (Obs::with(g, d1.img()), Obs::with(w, d2.img()), format!("{s:?}"), vec!["ok"])
        });
    };
}

/// Outcome tags for the `_s` family, read off the REFERENCE's return + handler.
fn sec_tags(fx: &Effects) -> Vec<&'static str> {
    let mut t = vec![];
    match fx.ret as c_int {
        0 => t.push("ok"),
        22 => t.push("EINVAL"),
        34 => t.push("ERANGE"),
        42 => t.push("EILSEQ"),
        80 => t.push("STRUNCATE"),
        _ => {}
    }
    if fx.pmt > 0 {
        t.push("handler");
    } else if fx.ret != 0 {
        t.push("no-handler");
    }
    t
}

fn show(a: &[u8], null: bool) -> String {
    if null {
        "NULL".into()
    } else {
        format!("{a:?}")
    }
}

fn showw(a: &[u16], null: bool) -> String {
    if null {
        "NULL".into()
    } else {
        format!("{:04x?}", &a[..a.len().min(28)])
    }
}

macro_rules! wide_parse_rows {
    ($suffix:literal) => {
        // The wide engine writes `*endptr = nptr` EARLY (right after the front door, before
        // the whitespace skip) and again only `if(!empty)` — a different endptr contract
        // from the narrow `_strtoi64_l`, which writes once at the end. The sentinel seed is
        // what makes both observable.
        grade(
            concat!("_wcstoi64_l", $suffix),
            &["front-door", "ERANGE", "clean"],
            PARSE_ITERS,
            |r| {
                let a = gen_wparse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let pass_end = r.below(4) != 0;
                let (mut e1, mut e2) = (ENDPTR_SENTINEL as *mut u16, ENDPTR_SENTINEL as *mut u16);
                let (pe1, pe2) = if pass_end {
                    (&mut e1 as *mut *mut u16, &mut e2 as *mut *mut u16)
                } else {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                };
                let g = Effects::subject(|| unsafe { _wcstoi64_l(p, pe1, base, core::ptr::null_mut()) });
                let w = Effects::reference(|| unsafe { ref__wcstoi64_l(p, pe2, base, core::ptr::null_mut()) });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let tags = parse_tags(&w);
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("base={base} end={pass_end} {}", showw(&a, null)),
                    tags,
                )
            },
        );
        grade(
            concat!("_wcstoui64_l", $suffix),
            &["front-door", "ERANGE", "clean"],
            PARSE_ITERS,
            |r| {
                let a = gen_wparse_input(r);
                let base = BASES[r.below(9)];
                let null = r.below(32) == 0;
                let p = if null { core::ptr::null() } else { a.as_ptr() };
                let pass_end = r.below(4) != 0;
                let (mut e1, mut e2) = (ENDPTR_SENTINEL as *mut u16, ENDPTR_SENTINEL as *mut u16);
                let (pe1, pe2) = if pass_end {
                    (&mut e1 as *mut *mut u16, &mut e2 as *mut *mut u16)
                } else {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                };
                let g = Effects::subject(|| unsafe { _wcstoui64_l(p, pe1, base, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__wcstoui64_l(p, pe2, base, core::ptr::null_mut()) as i64 });
                let (mut gi, mut wi) = (Vec::new(), Vec::new());
                push_i64(&mut gi, endptr_code(e1, p));
                push_i64(&mut wi, endptr_code(e2, p));
                let tags = parse_tags(&w);
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("base={base} end={pass_end} {}", showw(&a, null)),
                    tags,
                )
            },
        );
        wide_clamp_row!(concat!("_wcstol_l", $suffix), _wcstol_l, ref__wcstol_l, i32, &["front-door", "ERANGE", "clean"], false);
        wide_clamp_row!(concat!("_wcstoul_l", $suffix), _wcstoul_l, ref__wcstoul_l, u32, &["front-door", "ERANGE", "clean", "clamp1", "clamp-high"], true);
        grade(concat!("_wtoi_l", $suffix), &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
            let a = gen_wparse_input(r);
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let g = Effects::subject(|| unsafe { _wtoi_l(p, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__wtoi_l(p, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), showw(&a, null), tags)
        });
        grade(concat!("_wtol_l", $suffix), &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
            let a = gen_wparse_input(r);
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let g = Effects::subject(|| unsafe { _wtol_l(p, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__wtol_l(p, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), showw(&a, null), tags)
        });
        // ── _wtoi64_l — THE no-overflow-check row ───────────────────────────────────
        //
        // A completely separate, much older parser: no base, no endptr, no FOLDDIGITS,
        // and no overflow check at all. `"99999999999999999999"` comes back WRAPPED and
        // errno untouched, while `_wcstoi64_l` right next to it saturates and sets ERANGE.
        // `wrapped` is required so a corpus that stops overflowing fails loudly.
        //
        // Note there is no NULL case: `_wtoi64_l` has no front door and dereferences.
        grade(concat!("_wtoi64_l", $suffix), &["clean", "wrapped"], PARSE_ITERS, |r| {
            let a = gen_wparse_input(r);
            let g = Effects::subject(|| unsafe { _wtoi64_l(a.as_ptr(), core::ptr::null_mut()) });
            let w = Effects::reference(|| unsafe { ref__wtoi64_l(a.as_ptr(), core::ptr::null_mut()) });
            let mut tags = parse_tags(&w);
            // Wrapped iff a plain decimal parse of the digits exceeds 64 bits.
            if wide_decimal_overflows(&a) {
                tags.push("wrapped");
            }
            (Obs::new(g), Obs::new(w), showw(&a, false), tags)
        });
    };
}

/// Shared body for the four `__msvcrt_long`-returning wide clamps.
macro_rules! wide_clamp_row {
    ($name:expr, $subj:ident, $refr:ident, $ty:ty, $require:expr, $clamp_tags:literal) => {
        grade($name, $require, PARSE_ITERS, |r| {
            let a = gen_wparse_input(r);
            let base = BASES[r.below(9)];
            let null = r.below(32) == 0;
            let p = if null { core::ptr::null() } else { a.as_ptr() };
            let (mut e1, mut e2) = (ENDPTR_SENTINEL as *mut u16, ENDPTR_SENTINEL as *mut u16);
            let g = Effects::subject(|| unsafe { $subj(p, &mut e1, base, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { $refr(p, &mut e2, base, core::ptr::null_mut()) as i64 });
            let (mut gi, mut wi) = (Vec::new(), Vec::new());
            push_i64(&mut gi, endptr_code(e1, p));
            push_i64(&mut wi, endptr_code(e2, p));
            let mut tags = parse_tags(&w);
            if $clamp_tags {
                if w.ret == 1 && w.errno == 34 {
                    tags.push("clamp1");
                }
                if w.ret == 0xFFFF_FFFF && w.errno == 34 {
                    tags.push("clamp-high");
                }
            }
            (Obs::with(g, gi), Obs::with(w, wi), format!("base={base} {}", showw(&a, null)), tags)
        });
    };
}

/// Would `_wtoi64_l`'s unchecked `running_total = running_total*10 + c - '0'` have wrapped?
/// Modelled on the INPUT, never on either implementation's answer.
fn wide_decimal_overflows(s: &[u16]) -> bool {
    let mut i = 0;
    while i < s.len() && (s[i] == 0x20 || (0x09..=0x0D).contains(&s[i])) {
        i += 1;
    }
    if i < s.len() && (s[i] == b'+' as u16 || s[i] == b'-' as u16) {
        i += 1;
    }
    let mut acc: u128 = 0;
    let mut any = false;
    while i < s.len() && s[i] >= b'0' as u16 && s[i] <= b'9' as u16 {
        acc = acc * 10 + (s[i] - b'0' as u16) as u128;
        any = true;
        if acc > u64::MAX as u128 {
            return true;
        }
        i += 1;
    }
    let _ = any;
    false
}

macro_rules! wide_case_rows {
    ($suffix:literal) => {
        // The wide `_s` folders MUTATE BEFORE THEY FAIL — they fold as they scan, so a
        // buffer with no terminator inside `n` comes back with its first `n` characters
        // folded and THEN `str[0]` truncated. The narrow `_strlwr_s_l` scans first and
        // never touches the buffer. Only a whole-image compare can tell those apart.
        grade(
            concat!("_wcsupr_s_l", $suffix),
            &["ok", "EINVAL", "no-handler"],
            CASE_ITERS,
            |r| {
                let s = gen_wstr(r, 24);
                let n = match r.below(6) {
                    0 => 0,
                    1 => usize::MAX,
                    _ => r.below(28),
                };
                let null = r.below(24) == 0;
                let (mut d1, mut d2) = (WBuf::seeded(&s, 40), WBuf::seeded(&s, 40));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let g = Effects::subject(|| unsafe { _wcsupr_s_l(q1, n, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__wcsupr_s_l(q2, n, core::ptr::null_mut()) as i64 });
                let tags = sec_tags(&w);
                (
                    Obs::with(g, d1.img()),
                    Obs::with(w, d2.img()),
                    format!("n={n} null={null} slen={}", s.len() - 1),
                    tags,
                )
            },
        );
        grade(
            concat!("_wcslwr_s_l", $suffix),
            &["ok", "EINVAL", "no-handler"],
            CASE_ITERS,
            |r| {
                let s = gen_wstr(r, 24);
                let n = match r.below(6) {
                    0 => 0,
                    1 => usize::MAX,
                    _ => r.below(28),
                };
                let null = r.below(24) == 0;
                let (mut d1, mut d2) = (WBuf::seeded(&s, 40), WBuf::seeded(&s, 40));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let g = Effects::subject(|| unsafe { _wcslwr_s_l(q1, n, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__wcslwr_s_l(q2, n, core::ptr::null_mut()) as i64 });
                let tags = sec_tags(&w);
                (
                    Obs::with(g, d1.img()),
                    Obs::with(w, d2.img()),
                    format!("n={n} null={null} slen={}", s.len() - 1),
                    tags,
                )
            },
        );
        grade(concat!("_wcsupr_l", $suffix), &["ok"], CASE_ITERS, |r| {
            let s = gen_wstr(r, 30);
            let (mut d1, mut d2) = (WBuf::seeded(&s, 40), WBuf::seeded(&s, 40));
            let (p1, p2) = (d1.p(), d2.p());
            let g = Effects::subject(|| unsafe { (_wcsupr_l(p1, core::ptr::null_mut()) == p1) as i64 });
            let w = Effects::reference(|| unsafe { (ref__wcsupr_l(p2, core::ptr::null_mut()) == p2) as i64 });
            (Obs::with(g, d1.img()), Obs::with(w, d2.img()), format!("slen={}", s.len() - 1), vec!["ok"])
        });
        grade(concat!("_wcslwr_l", $suffix), &["ok"], CASE_ITERS, |r| {
            let s = gen_wstr(r, 30);
            let (mut d1, mut d2) = (WBuf::seeded(&s, 40), WBuf::seeded(&s, 40));
            let (p1, p2) = (d1.p(), d2.p());
            let g = Effects::subject(|| unsafe { (_wcslwr_l(p1, core::ptr::null_mut()) == p1) as i64 });
            let w = Effects::reference(|| unsafe { (ref__wcslwr_l(p2, core::ptr::null_mut()) == p2) as i64 });
            (Obs::with(g, d1.img()), Obs::with(w, d2.img()), format!("slen={}", s.len() - 1), vec!["ok"])
        });
        // Returns the RAW wchar_t difference, not a clamp — and `_wcsicmp_l` has NO
        // zero-count early-out, unlike `_wcsnicmp_l`.
        grade(concat!("_wcsicmp_l", $suffix), &["clean", "front-door"], CASE_ITERS, |r| {
            let a = gen_wstr(r, 20);
            let b = if r.below(3) == 0 { a.clone() } else { gen_wstr(r, 20) };
            let na = r.below(16) == 0;
            let nb = r.below(16) == 0;
            let (pa, pb) = (
                if na { core::ptr::null() } else { a.as_ptr() },
                if nb { core::ptr::null() } else { b.as_ptr() },
            );
            let g = Effects::subject(|| unsafe { _wcsicmp_l(pa, pb, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__wcsicmp_l(pa, pb, core::ptr::null_mut()) as i64 });
            let tags = parse_tags(&w);
            (Obs::new(g), Obs::new(w), format!("{} vs {}", showw(&a, na), showw(&b, nb)), tags)
        });
        grade(
            concat!("_wcsnicmp_l", $suffix),
            &["clean", "front-door", "zero-count"],
            CASE_ITERS,
            |r| {
                let a = gen_wstr(r, 20);
                let b = if r.below(3) == 0 { a.clone() } else { gen_wstr(r, 20) };
                let n = if r.below(8) == 0 { 0 } else { r.below(24) };
                let na = r.below(16) == 0;
                let nb = r.below(16) == 0;
                let (pa, pb) = (
                    if na { core::ptr::null() } else { a.as_ptr() },
                    if nb { core::ptr::null() } else { b.as_ptr() },
                );
                let g = Effects::subject(|| unsafe { _wcsnicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__wcsnicmp_l(pa, pb, n, core::ptr::null_mut()) as i64 });
                let mut tags = parse_tags(&w);
                if n == 0 {
                    tags.push("zero-count");
                }
                (Obs::new(g), Obs::new(w), format!("n={n} {} vs {}", showw(&a, na), showw(&b, nb)), tags)
            },
        );
    };
}

// ── the wchar -> multibyte family ─────────────────────────────────────────────────────

/// What the single-character converters must reach in a codepage where a character CAN be
/// unrepresentable. `no-handler` is the load-bearing half: EILSEQ returns without firing
/// the invalid-parameter handler, while `size > INT_MAX` fires it — one function, two
/// error contracts, and only the handler count separates them.
const WMB_S_REQ: &[&str] = &["ok", "EILSEQ", "no-handler", "EINVAL", "handler", "ERANGE"];
const WMB_P_REQ: &[&str] = &["ok", "fail"];

/// **CP1252**, which is neither the raw codepage nor DBCS, and so reaches neither ERANGE
/// door: `size >= 1` is only checked on the `lc_codepage == 0` path, and a single Latin-1
/// character always fits in one byte, so the converter never reports an insufficient
/// buffer. Both ERANGE arms ARE graded — the first under codepage 0 and NLS, the second
/// under CP932, where a two-byte character genuinely does not fit in `size == 1`.
const WMB_S_REQ_1252: &[&str] = &["ok", "EILSEQ", "no-handler", "EINVAL", "handler"];

/// And what they can reach under **CP_UTF8**, which is strictly less.
///
/// This is a limit of the harness's shared `WideCharToMultiByte` model, and it is worth
/// stating exactly. UTF-8 has no best-fit substitution, so passing `lpUsedDefaultChar` for
/// CP_UTF8 is illegal and the C passes NULL — which is precisely the branch these rows
/// exist to check. But refimpl.c's model reports an unrepresentable character ONLY through
/// that out-parameter; real kernel32 would additionally return 0 for e.g. a lone surrogate.
/// With the out-parameter NULL the model therefore never fails, and no input can drive
/// `_wctomb_s_l` to EILSEQ in this mode.
///
/// The rows are still worth running, and this is not a hole in the grading of that branch:
/// if the Rust wrongly passed `&error` under CP_UTF8, the model WOULD set it for every
/// character above 0xFF and the row would go DIVERGE (rust=EILSEQ, msvcrt=ok). The `perror
/// = NULL` decision is graded here; only the *consequence of a genuine UTF-8 encoding
/// failure* is not, and that is graded under CP1252 / CP932 / codepage 0 instead.
const WMB_S_REQ_UTF8: &[&str] = &["ok", "EINVAL", "handler"];
const WMB_P_REQ_UTF8: &[&str] = &["ok"];

macro_rules! wmb_rows {
    ($suffix:literal, $req_s:expr, $req_p:expr) => {
        // `wctob` EXHAUSTIVE over the entire 16-bit `wint_t` domain — so U+00FF, the whole
        // reason the strict `< 0xff` is worth preserving, cannot be a case we missed.
        {
            let name = concat!("wctob", $suffix);
            let mut bad = None;
            let mut ff_eof = false;
            for c in 0u32..=0xFFFF {
                let c = c as u16;
                let g = Effects::subject(|| unsafe { wctob(c) as i64 });
                let w = Effects::reference(|| unsafe { ref_wctob(c) as i64 });
                unsafe { CASES += 1 };
                if g != w {
                    bad = Some(format!("c=U+{c:04X}\n        rust  = {g}\n        msvcrt= {w}"));
                    break;
                }
                if c == 0xFF && w.ret == -1 {
                    ff_eof = true;
                }
            }
            // THE `< 0xff` QUIRK IS UNFALSIFIABLE, and saying so is the honest result.
            //
            // A negative control that relaxes it to `<= 0xff` moves NOTHING, and cannot:
            // the only input that separates the two spellings is U+00FF, and both arms
            // answer -1 there — `(signed char)0xFF` IS `EOF`. So no differential test, this
            // one included, can distinguish the preserved quirk from the "fixed" version.
            // Asserted rather than argued, so nobody re-derives it:
            assert_eq!(0xFFu16 as i8 as c_int, EOF_C,
                "(signed char)0xFF is no longer EOF — the wctob `< 0xff` note below is stale");
            // What the row DOES pin is everything else, exhaustively: a control that
            // returned the raw value instead of the signed char is caught (loudly, by
            // assert_shims_live, and by this sweep at U+0080).
            match bad {
                Some(b) => record(name.to_string(), V::Diverge, b),
                None => record(
                    name.to_string(),
                    V::Match,
                    format!(
                        "exhaustive over U+0000..=U+FFFF; U+00FF reports EOF: {ff_eof}. \
                         The strict `< 0xff` itself is UNFALSIFIABLE\n        (see above): \
                         (signed char)0xFF == EOF, so `<` and `<=` agree on the one input \
                         that separates them."
                    ),
                ),
            }
        }
        // `_wctomb_s_l` — five distinct exits, and they disagree about the handler:
        // `size > INT_MAX` and the insufficient-buffer path FIRE it, EILSEQ does not.
        grade(
            concat!("_wctomb_s_l", $suffix),
            $req_s,
            CASE_ITERS,
            |r| {
                let wch = gen_wchar(r);
                let null_mb = r.below(12) == 0;
                let size = gen_size(r, null_mb);
                let null_len = r.below(6) == 0;
                let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null_mb { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let (mut l1, mut l2) = (0x5A5Ai32, 0x5A5Ai32);
                let (pl1, pl2) = if null_len {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                } else {
                    (&mut l1 as *mut c_int, &mut l2 as *mut c_int)
                };
                let g = Effects::subject(|| unsafe { _wctomb_s_l(pl1, q1, size, wch, core::ptr::null_mut()) as i64 });
                let w = Effects::reference(|| unsafe { ref__wctomb_s_l(pl2, q2, size, wch, core::ptr::null_mut()) as i64 });
                let (mut gi, mut wi) = (d1.img(), d2.img());
                push_i64(&mut gi, l1 as i64);
                push_i64(&mut wi, l2 as i64);
                let tags = sec_tags(&w);
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("wch=U+{wch:04X} size={size} nullmb={null_mb} nulllen={null_len}"),
                    tags,
                )
            },
        );
        grade(concat!("wctomb_s", $suffix), $req_s, CASE_ITERS, |r| {
            let wch = gen_wchar(r);
            let null_mb = r.below(12) == 0;
            let size = gen_size(r, null_mb);
            let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
            let (p1, p2) = (d1.p(), d2.p());
            let (q1, q2) = if null_mb { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
            let (mut l1, mut l2) = (0x5A5Ai32, 0x5A5Ai32);
            let g = Effects::subject(|| unsafe { wctomb_s(&mut l1, q1, size, wch) as i64 });
            let w = Effects::reference(|| unsafe { ref_wctomb_s(&mut l2, q2, size, wch) as i64 });
            let (mut gi, mut wi) = (d1.img(), d2.img());
            push_i64(&mut gi, l1 as i64);
            push_i64(&mut wi, l2 as i64);
            let tags = sec_tags(&w);
            (Obs::with(g, gi), Obs::with(w, wi), format!("wch=U+{wch:04X} size={size} nullmb={null_mb}"), tags)
        });
        // `_wctomb_l` is the ReactOS arm: it sizes the destination from `mb_cur_max` and
        // maps any non-zero status to -1, where Wine's passes MB_LEN_MAX and ignores it.
        grade(concat!("_wctomb_l", $suffix), $req_p, CASE_ITERS, |r| {
            let wch = gen_wchar(r);
            let null_dst = r.below(12) == 0;
            let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
            let (p1, p2) = (d1.p(), d2.p());
            let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
            let g = Effects::subject(|| unsafe { _wctomb_l(q1, wch, core::ptr::null_mut()) as i64 });
            let w = Effects::reference(|| unsafe { ref__wctomb_l(q2, wch, core::ptr::null_mut()) as i64 });
            let tags = vec![if w.ret < 0 { "fail" } else { "ok" }];
            (
                Obs::with(g, d1.img()),
                Obs::with(w, d2.img()),
                format!("wch=U+{wch:04X} nulldst={null_dst}"),
                tags,
            )
        });
        grade(concat!("wctomb", $suffix), $req_p, CASE_ITERS, |r| {
            let wch = gen_wchar(r);
            let null_dst = r.below(12) == 0;
            let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
            let (p1, p2) = (d1.p(), d2.p());
            let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
            let g = Effects::subject(|| unsafe { wctomb(q1, wch) as i64 });
            let w = Effects::reference(|| unsafe { ref_wctomb(q2, wch) as i64 });
            let tags = vec![if w.ret < 0 { "fail" } else { "ok" }];
            (Obs::with(g, d1.img()), Obs::with(w, d2.img()), format!("wch=U+{wch:04X} nulldst={null_dst}"), tags)
        });
        // `wcrtomb` widens `wctomb`'s `int` into a `size_t`, so its failure is
        // (size_t)-1 and NOT 0xFFFFFFFF on a 64-bit target. The mbstate is zeroed first.
        grade(concat!("wcrtomb", $suffix), $req_p, CASE_ITERS, |r| {
            let wch = gen_wchar(r);
            let pass_state = r.below(2) == 0;
            let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
            let (p1, p2) = (d1.p(), d2.p());
            let (mut s1, mut s2) = (0x5A5Ai32, 0x5A5Ai32);
            let (ps1, ps2) = if pass_state {
                (&mut s1 as *mut c_int, &mut s2 as *mut c_int)
            } else {
                (core::ptr::null_mut(), core::ptr::null_mut())
            };
            let g = Effects::subject(|| unsafe { wcrtomb(p1, wch, ps1) as i64 });
            let w = Effects::reference(|| unsafe { ref_wcrtomb(p2, wch, ps2) as i64 });
            let (mut gi, mut wi) = (d1.img(), d2.img());
            push_i64(&mut gi, s1 as i64);
            push_i64(&mut wi, s2 as i64);
            let mut tags = vec![if w.ret == -1 { "fail" } else { "ok" }];
            if pass_state {
                tags.push("state");
            }
            (Obs::with(g, gi), Obs::with(w, wi), format!("wch=U+{wch:04X} state={pass_state}"), tags)
        });
        grade(concat!("wcrtomb_s", $suffix), $req_s, CASE_ITERS, |r| {
            let wch = gen_wchar(r);
            // `mbchar` is never NULL here, so `gen_size` withholds the sizes that would
            // make msvcrt memset gigabytes into it.
            let size = gen_size(r, false);
            let pass_state = r.below(2) == 0;
            let null_len = r.below(6) == 0;
            let (mut d1, mut d2) = (Buf::new(24), Buf::new(24));
            let (p1, p2) = (d1.p(), d2.p());
            let (mut n1, mut n2) = (0x5A5Ausize, 0x5A5Ausize);
            let (pn1, pn2) = if null_len {
                (core::ptr::null_mut(), core::ptr::null_mut())
            } else {
                (&mut n1 as *mut usize, &mut n2 as *mut usize)
            };
            let (mut s1, mut s2) = (0x5A5Ai32, 0x5A5Ai32);
            let (ps1, ps2) = if pass_state {
                (&mut s1 as *mut c_int, &mut s2 as *mut c_int)
            } else {
                (core::ptr::null_mut(), core::ptr::null_mut())
            };
            let g = Effects::subject(|| unsafe { wcrtomb_s(pn1, p1, size, wch, ps1) as i64 });
            let w = Effects::reference(|| unsafe { ref_wcrtomb_s(pn2, p2, size, wch, ps2) as i64 });
            let (mut gi, mut wi) = (d1.img(), d2.img());
            push_i64(&mut gi, n1 as i64);
            push_i64(&mut wi, n2 as i64);
            push_i64(&mut gi, s1 as i64);
            push_i64(&mut wi, s2 as i64);
            let tags = sec_tags(&w);
            (Obs::with(g, gi), Obs::with(w, wi), format!("wch=U+{wch:04X} size={size}"), tags)
        });
        // `wcsrtombs` advances the caller's source pointer — NULL when it consumed the
        // terminator, `+i` when it ran out of room — so the pointer is part of the answer.
        grade(concat!("wcsrtombs", $suffix), &["ok", "EILSEQ", "nodst"], CASE_ITERS, |r| {
            let s = gen_conv_wstr(r);
            let count = r.below(s.len() + 6);
            let null_dst = r.below(6) == 0;
            let pass_state = r.below(2) == 0;
            let (mut d1, mut d2) = (Buf::new(48), Buf::new(48));
            let (p1, p2) = (d1.p(), d2.p());
            let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
            let (mut w1, mut w2): (*const u16, *const u16) = (s.as_ptr(), s.as_ptr());
            let (mut s1, mut s2) = (0x5A5Ai32, 0x5A5Ai32);
            let (ps1, ps2) = if pass_state {
                (&mut s1 as *mut c_int, &mut s2 as *mut c_int)
            } else {
                (core::ptr::null_mut(), core::ptr::null_mut())
            };
            let g = Effects::subject(|| unsafe { wcsrtombs(q1, &mut w1, count, ps1) as i64 });
            let w = Effects::reference(|| unsafe { ref_wcsrtombs(q2, &mut w2, count, ps2) as i64 });
            let (mut gi, mut wi) = (d1.img(), d2.img());
            push_i64(&mut gi, srcptr_code(w1, s.as_ptr()));
            push_i64(&mut wi, srcptr_code(w2, s.as_ptr()));
            let mut tags = vec![if w.errno == 42 { "EILSEQ" } else { "ok" }];
            if null_dst {
                tags.push("nodst");
            }
            (
                Obs::with(g, gi),
                Obs::with(w, wi),
                format!("count={count} nulldst={null_dst} src={}", showw(&s, false)),
                tags,
            )
        });
        // ── the `_s` converters, and the mbstr[-1] touch ────────────────────────────
        //
        // `wcsrtombs_s_l`'s `conv == size` arm evaluates `mbstr[conv-1]` — with size 0 and
        // a non-NULL mbstr that is `mbstr[-1]`, a genuine out-of-bounds read AND write in
        // msvcrt. The 8-byte guard prefix in every Buf above is where it lands, and the
        // image compare grades it byte for byte. `oob-1` is a required outcome.
        grade(
            concat!("wcstombs_s", $suffix),
            &["ok", "EINVAL", "ERANGE", "STRUNCATE", "handler", "oob-1"],
            CASE_ITERS,
            |r| {
                let s = gen_conv_wstr(r);
                let (size, count) = gen_size_count(r, s.len());
                let null_dst = r.below(8) == 0;
                // `wcstombs_s` takes the source BY VALUE and passes `&wcstr`, which is
                // never NULL — so `!mbstr && !size && wcstr` is entered on a NULL source
                // too, and `wcsrtombs_l` then calls `wcslen(NULL)`. That is a genuine
                // msvcrt NULL-deref, identical on both sides, and executing it teaches a
                // differential harness nothing except what its own SIGSEGV looks like.
                let null_src = r.below(16) == 0 && !(null_dst && size == 0);
                let null_ret = r.below(6) == 0;
                let (mut d1, mut d2) = (Buf::new(48), Buf::new(48));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let sp = if null_src { core::ptr::null() } else { s.as_ptr() };
                let (mut n1, mut n2) = (0x5A5Ausize, 0x5A5Ausize);
                let (pn1, pn2) = if null_ret {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                } else {
                    (&mut n1 as *mut usize, &mut n2 as *mut usize)
                };
                let g = Effects::subject(|| unsafe { wcstombs_s(pn1, q1, size, sp, count) as i64 });
                let w = Effects::reference(|| unsafe { ref_wcstombs_s(pn2, q2, size, sp, count) as i64 });
                let (mut gi, mut wi) = (d1.img(), d2.img());
                push_i64(&mut gi, n1 as i64);
                push_i64(&mut wi, n2 as i64);
                let mut tags = sec_tags(&w);
                if !null_dst && size == 0 && !null_src {
                    tags.push("oob-1");
                }
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!(
                        "size={size} count={}{} nulldst={null_dst} nullsrc={null_src} src={}",
                        count,
                        if count == usize::MAX { " (_TRUNCATE)" } else { "" },
                        showw(&s, false)
                    ),
                    tags,
                )
            },
        );
        grade(
            concat!("_wcstombs_s_l", $suffix),
            &["ok", "EINVAL", "ERANGE", "STRUNCATE", "handler", "oob-1"],
            CASE_ITERS,
            |r| {
                let s = gen_conv_wstr(r);
                let (size, count) = gen_size_count(r, s.len());
                let null_dst = r.below(8) == 0;
                // see the `wcstombs_s` row: NULL source + the sizing shape is msvcrt's own
                // `wcslen(NULL)`, not something two implementations can disagree about.
                let null_src = r.below(16) == 0 && !(null_dst && size == 0);
                let (mut d1, mut d2) = (Buf::new(48), Buf::new(48));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let sp = if null_src { core::ptr::null() } else { s.as_ptr() };
                let (mut n1, mut n2) = (0x5A5Ausize, 0x5A5Ausize);
                let g = Effects::subject(|| unsafe {
                    _wcstombs_s_l(&mut n1, q1, size, sp, count, core::ptr::null_mut()) as i64
                });
                let w = Effects::reference(|| unsafe {
                    ref__wcstombs_s_l(&mut n2, q2, size, sp, count, core::ptr::null_mut()) as i64
                });
                let (mut gi, mut wi) = (d1.img(), d2.img());
                push_i64(&mut gi, n1 as i64);
                push_i64(&mut wi, n2 as i64);
                let mut tags = sec_tags(&w);
                if !null_dst && size == 0 && !null_src {
                    tags.push("oob-1");
                }
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("size={size} count={count} nulldst={null_dst} nullsrc={null_src}"),
                    tags,
                )
            },
        );
        // `wcsrtombs_s` is the only one that can take the `!mbstr && !size && wcstr` early
        // exit — it hands the caller `conv + 1`, which is 0 (modular) on the failure path.
        grade(
            concat!("wcsrtombs_s", $suffix),
            &["ok", "EINVAL", "ERANGE", "STRUNCATE", "handler", "oob-1", "sizing", "null-srcval"],
            CASE_ITERS,
            |r| {
                let s = gen_conv_wstr(r);
                let (size, count) = gen_size_count(r, s.len());
                // the sizing call: mbstr == NULL && size == 0
                let sizing = r.below(5) == 0;
                let null_dst = sizing || r.below(8) == 0;
                let size = if sizing { 0 } else { size };
                let null_src = r.below(16) == 0;
                // `*wcstr == NULL` is a THIRD front door (`MSVCRT_CHECK_PMT(*wcstr !=
                // NULL)`), distinct from `wcstr == NULL`. It is only safe outside the
                // sizing shape, which reaches `wcslen(*wcstr)` before any check.
                let null_srcval = !null_src && r.below(16) == 0 && !(null_dst && size == 0);
                let (mut d1, mut d2) = (Buf::new(48), Buf::new(48));
                let (p1, p2) = (d1.p(), d2.p());
                let (q1, q2) = if null_dst { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (p1, p2) };
                let (mut w1, mut w2): (*const u16, *const u16) = if null_srcval {
                    (core::ptr::null(), core::ptr::null())
                } else {
                    (s.as_ptr(), s.as_ptr())
                };
                let (pw1, pw2) = if null_src {
                    (core::ptr::null_mut(), core::ptr::null_mut())
                } else {
                    (&mut w1 as *mut *const u16, &mut w2 as *mut *const u16)
                };
                let (mut n1, mut n2) = (0x5A5Ausize, 0x5A5Ausize);
                let g = Effects::subject(|| unsafe {
                    wcsrtombs_s(&mut n1, q1, size, pw1, count, core::ptr::null_mut()) as i64
                });
                let w = Effects::reference(|| unsafe {
                    ref_wcsrtombs_s(&mut n2, q2, size, pw2, count, core::ptr::null_mut()) as i64
                });
                let (mut gi, mut wi) = (d1.img(), d2.img());
                push_i64(&mut gi, n1 as i64);
                push_i64(&mut wi, n2 as i64);
                push_i64(&mut gi, srcptr_code(w1, s.as_ptr()));
                push_i64(&mut wi, srcptr_code(w2, s.as_ptr()));
                let mut tags = sec_tags(&w);
                if sizing && !null_src {
                    tags.push("sizing");
                }
                if !null_dst && size == 0 && !null_src && !null_srcval {
                    tags.push("oob-1");
                }
                if null_srcval {
                    tags.push("null-srcval");
                }
                (
                    Obs::with(g, gi),
                    Obs::with(w, wi),
                    format!("size={size} count={count} sizing={sizing} nulldst={null_dst} nullsrc={null_src} nullsrcval={null_srcval}"),
                    tags,
                )
            },
        );
    };
}

/// Where the caller's `const wchar_t **` ended up: -1 for the NULL the C plants when it
/// consumed the terminator, else the element offset it advanced to.
fn srcptr_code(p: *const u16, base: *const u16) -> i64 {
    if p.is_null() {
        -1
    } else {
        ((p as isize - base as isize) / 2) as i64
    }
}

/// Characters chosen for the converter family: ASCII, the 0x80..0xFF boundary (0xFF is the
/// `wctob` quirk and 0x100 the first EILSEQ), CJK (multi-byte under CP932/CP_UTF8), and
/// unpaired surrogates.
fn gen_wchar(r: &mut Rng) -> u16 {
    match r.below(8) {
        0 => r.below(0x80) as u16,
        1 => 0xFE + r.below(4) as u16, // 0xFE, 0xFF, 0x100, 0x101 — the exact boundary
        2 => 0x80 + r.below(0x80) as u16,
        3 => 0x3040 + r.below(0x100) as u16, // hiragana
        4 => 0xFF61 + r.below(0x3F) as u16,  // halfwidth katakana (single byte in CP932)
        5 => 0xD800 + r.below(0x800) as u16, // lone surrogates
        6 => 0,
        _ => r.next() as u16,
    }
}

/// `size` for the `wctomb_s` family.
///
/// The huge values are only offered when `mbchar` is NULL or when they are ABOVE INT_MAX,
/// and that is a bound on the HARNESS, not a softened oracle. `_wctomb_s_l`'s EILSEQ arm
/// runs `memset(mbchar, 0, size)` on a size the caller merely *claimed* — so `size ==
/// INT_MAX` with a real pointer makes msvcrt zero two gigabytes of the caller's address
/// space. That is a genuine unbounded write in the C, identical on both sides, and the
/// only thing a harness can learn by executing it is what its own SIGSEGV looks like.
/// `size > INT_MAX` is safe because `MSVCRT_CHECK_PMT(size <= INT_MAX)` rejects it before
/// any write, and that front door IS graded.
fn gen_size(r: &mut Rng, mbchar_is_null: bool) -> usize {
    match r.below(8) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => c_int::MAX as usize + 1, // trips `size <= INT_MAX` -> EINVAL + handler
        4 => usize::MAX,
        5 if mbchar_is_null => c_int::MAX as usize, // the `!mbchar && size>0` early exit
        _ => r.below(9),
    }
}

/// `size` and `count` for the `_s` converters. `count == _TRUNCATE` is its own contract
/// (it turns the exact-fit arm into a STRUNCATE), and `size == 0` with a non-NULL mbstr is
/// what reaches `mbstr[-1]`.
fn gen_size_count(r: &mut Rng, slen: usize) -> (usize, usize) {
    let size = match r.below(8) {
        0 => 0,
        1 => 1,
        2 => slen,
        3 => slen.saturating_sub(1),
        4 => slen + 1,
        _ => r.below(slen + 4),
    };
    let count = match r.below(4) {
        0 => usize::MAX, // _TRUNCATE
        1 => size,
        2 => slen,
        _ => r.below(slen + 4),
    };
    (size, count)
}

/// Source strings for the converters: short, and mostly representable, so the loops in
/// `wcsrtombs_l` actually run to completion rather than bailing on the first character.
fn gen_conv_wstr(r: &mut Rng) -> Vec<u16> {
    let n = r.below(8);
    let mut v = Vec::with_capacity(n + 1);
    for _ in 0..n {
        v.push(match r.below(6) {
            0 => b'a' as u16 + r.below(26) as u16,
            1 => r.below(0x80) as u16,
            2 => 0xF0 + r.below(0x20) as u16, // straddles the 0xFF/0x100 EILSEQ boundary
            3 => 0x3042 + r.below(0x40) as u16,
            4 => 0xFF71 + r.below(0x10) as u16,
            _ => 1 + r.next() as u16 % 0xFFFE,
        });
    }
    v.push(0);
    v
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The locale-blind rows (run once)
// ══════════════════════════════════════════════════════════════════════════════════════

unsafe fn blind_rows() {
    // `_strdup` — NULL in, NULL out, no errno, no handler; otherwise a heap copy. Both
    // sides call the host malloc, so the POINTERS differ by construction: what is compared
    // is the bytes it copied and whether it returned NULL at all.
    grade("_strdup", &["ok", "null"], CASE_ITERS, |r| {
        let s = gen_str(r, 40);
        let null = r.below(12) == 0;
        let p = if null { core::ptr::null() } else { s.as_ptr() };
        let mut gi = Vec::new();
        let mut wi = Vec::new();
        let g = Effects::subject(|| unsafe {
            let q = _strdup(p);
            gi = capture_cstr(q);
            if !q.is_null() {
                free(q as *mut c_void);
            }
            !q.is_null() as i64
        });
        let w = Effects::reference(|| unsafe {
            let q = ref__strdup(p);
            wi = capture_cstr(q);
            if !q.is_null() {
                free(q as *mut c_void);
            }
            !q.is_null() as i64
        });
        let tags = vec![if null { "null" } else { "ok" }];
        (Obs::with(g, gi), Obs::with(w, wi), show(&s, null), tags)
    });

    // `_atoflt` — the classification is taken off the NARROWED float, not the double, so
    // `1e-40` is an `_UNDERFLOW` even though the double holds it fine. All three returns
    // are required outcomes.
    grade("_atoflt", &["ok", "OVERFLOW", "UNDERFLOW"], CASE_ITERS, |r| {
        let mut s = gen_float_str(r);
        let mut s2 = s.clone();
        let (mut v1, mut v2) = (CrtFloat { f: f32::from_bits(0x5A5A_5A5A) }, CrtFloat { f: f32::from_bits(0x5A5A_5A5A) });
        let g = Effects::subject(|| unsafe { _atoflt(&mut v1, s.as_mut_ptr()) as i64 });
        let w = Effects::reference(|| unsafe { ref__atoflt(&mut v2, s2.as_mut_ptr()) as i64 });
        let gi = v1.f.to_bits().to_le_bytes().to_vec();
        let wi = v2.f.to_bits().to_le_bytes().to_vec();
        let tags = vec![match w.ret {
            0 => "ok",
            3 => "OVERFLOW",
            4 => "UNDERFLOW",
            _ => "other",
        }];
        (
            Obs::with(g, gi),
            Obs::with(w, wi),
            format!("{:?} -> rust {} / msvcrt {}", core::str::from_utf8(&s2[..s2.len() - 1]), v1.f, v2.f),
            tags,
        )
    });

    // ── the I10_OUTPUT substrate probe ──────────────────────────────────────────────
    //
    // `I10_OUTPUT` formats through `sprintf`. The reference's is renamed to `ref_sprintf`
    // (msvcrt's own engine); the subject's binds to glibc's, because msvcrt-crt-rust does
    // not define `sprintf`. This row measures that difference directly, on exactly the
    // call I10_OUTPUT makes, so the next row can be honest about what it grades.
    let mut substrate_bad = 0u64;
    grade("sprintf %.*le [SUBSTRATE]", &["finite", "nonfinite"], CASE_ITERS, |r| {
        let ld = gen_ld80(r);
        let d = ld80_as_f64(&ld).abs();
        let prec = r.below(21) as c_int;
        let mut fmt = [0u8; 8];
        let (mut b1, mut b2) = ([0u8; 64], [0u8; 64]);
        unsafe {
            sprintf(fmt.as_mut_ptr(), b"%%.%dle\0".as_ptr(), prec);
            sprintf(b1.as_mut_ptr(), fmt.as_ptr(), d);
            ref_sprintf(b2.as_mut_ptr(), fmt.as_ptr(), d);
        }
        let g = Obs::with(Effects { ret: 0, errno: 0, pmt: 0 }, b1.to_vec());
        let w = Obs::with(Effects { ret: 0, errno: 0, pmt: 0 }, b2.to_vec());
        if g.img != w.img {
            substrate_bad += 1;
        }
        let tags = vec![if d.is_finite() { "finite" } else { "nonfinite" }];
        (g, w, format!("prec={prec} d={d:e}"), tags)
    });

    // ── $I10_OUTPUT ────────────────────────────────────────────────────────────────
    //
    // The whole 26-byte out-struct is compared (with guards), not just the return — the
    // function deliberately writes stripped trailing digits PAST the NUL it just planted,
    // which is the part `data->len` alone cannot express.
    //
    // Restricted to the sub-domain the substrate probe above proves is common ground:
    // inputs whose `fpnum_double` result is FINITE. glibc prints an infinity as "inf" and
    // msvcrt as "1.#INF…", so a non-finite `d` reaching `sprintf` measures the two printf
    // engines, not the two I10_OUTPUTs. NaN/INF *patterns* are still graded in full — they
    // exit before `sprintf` on the `1#INF`/`1#QNAN`/`1#SNAN` path, which is required.
    let mut i10_skipped_inf = 0u64;
    let mut i10_skipped_smash = 0u64;
    grade("I10_OUTPUT", &["nonfinite", "finite", "flag1"], CASE_ITERS, |r| {
        let (ld, prec, flag) = loop {
            let ld = gen_ld80(r);
            let prec = match r.below(6) {
                0 => 0,
                1 => 1,
                2 => 21,
                3 => 22,
                4 => -1,
                _ => r.below(25) as c_int,
            };
            let flag = r.below(4) as c_int;
            // Two exclusions, both counted and both reported. (1) A non-finite `d` would
            // measure the two SPRINTF engines, not the two I10_OUTPUTs — see the substrate
            // row. (2) `i10_safe` is msvcrt's own stack smash.
            if (ld80_expword(&ld) & 0x7FFF) != 0x7FFF && !ld80_as_f64(&ld).is_finite() {
                i10_skipped_inf += 1;
                continue;
            }
            if !i10_safe(&ld, prec, flag) {
                i10_skipped_smash += 1;
                continue;
            }
            break (ld, prec, flag);
        };
        let (mut d1, mut d2) = (I10Data::poison(), I10Data::poison());
        let g = Effects::subject(|| unsafe { I10_OUTPUT(ld, prec, flag, &mut d1) as i64 });
        let w = Effects::reference(|| unsafe { ref_I10_OUTPUT(ld, prec, flag, &mut d2) as i64 });
        let mut tags = vec![if w.ret == 0 { "nonfinite" } else { "finite" }];
        if (flag & 1) != 0 {
            tags.push("flag1");
        }
        (
            Obs::with(g, d1.img()),
            Obs::with(w, d2.img()),
            format!(
                "ld80={:02x?} prec={prec} flag={flag} d={:e}",
                ld.ld,
                ld80_as_f64(&ld)
            ),
            tags,
        )
    });

    // `wcsncat` copies at most n elements and terminates by falling out of the loop —
    // which is a WRITE AT dst[len + n], one past the n it was given, whenever the source
    // did not run out first. That extra byte is only visible in a whole-buffer compare,
    // and the returned pointer is the ORIGINAL dst.
    grade("wcsncat", &["ok", "full"], CASE_ITERS, |r| {
        let a = gen_wstr(r, 10);
        let s = gen_wstr(r, 10);
        let n = r.below(14);
        let (mut d1, mut d2) = (WBuf::seeded(&a, 48), WBuf::seeded(&a, 48));
        let (p1, p2) = (d1.p(), d2.p());
        let g = Effects::subject(|| unsafe { (wcsncat(p1, s.as_ptr(), n) == p1) as i64 });
        let w = Effects::reference(|| unsafe { (ref_wcsncat(p2, s.as_ptr(), n) == p2) as i64 });
        let tags = vec![if s.len() - 1 >= n && n > 0 { "full" } else { "ok" }];
        (
            Obs::with(g, d1.img()),
            Obs::with(w, d2.img()),
            format!("n={n} dst={} src={}", showw(&a, false), showw(&s, false)),
            tags,
        )
    });

    // The four plain forwarders. They are one line each over the `_l` engines above, but a
    // forwarder that passes the wrong constant (base 10 vs 0, NULL vs a locale) is exactly
    // the kind of bug a `_l`-only grading cannot see.
    grade("_wcstoi64", &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
        let a = gen_wparse_input(r);
        let base = BASES[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (ENDPTR_SENTINEL as *mut u16, ENDPTR_SENTINEL as *mut u16);
        let g = Effects::subject(|| unsafe { _wcstoi64(p, &mut e1, base) });
        let w = Effects::reference(|| unsafe { ref__wcstoi64(p, &mut e2, base) });
        let (mut gi, mut wi) = (Vec::new(), Vec::new());
        push_i64(&mut gi, endptr_code(e1, p));
        push_i64(&mut wi, endptr_code(e2, p));
        let tags = parse_tags(&w);
        (Obs::with(g, gi), Obs::with(w, wi), format!("base={base} {}", showw(&a, null)), tags)
    });
    grade("_wcstoui64", &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
        let a = gen_wparse_input(r);
        let base = BASES[r.below(9)];
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let (mut e1, mut e2) = (ENDPTR_SENTINEL as *mut u16, ENDPTR_SENTINEL as *mut u16);
        let g = Effects::subject(|| unsafe { _wcstoui64(p, &mut e1, base) as i64 });
        let w = Effects::reference(|| unsafe { ref__wcstoui64(p, &mut e2, base) as i64 });
        let (mut gi, mut wi) = (Vec::new(), Vec::new());
        push_i64(&mut gi, endptr_code(e1, p));
        push_i64(&mut wi, endptr_code(e2, p));
        let tags = parse_tags(&w);
        (Obs::with(g, gi), Obs::with(w, wi), format!("base={base} {}", showw(&a, null)), tags)
    });
    wide_clamp_row!("wcstol", wcstol_fwd, ref_wcstol_fwd, i32, &["front-door", "ERANGE", "clean"], false);
    wide_clamp_row!("wcstoul", wcstoul_fwd, ref_wcstoul_fwd, u32, &["front-door", "ERANGE", "clean", "clamp1", "clamp-high"], true);
    grade("_wtoi", &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
        let a = gen_wparse_input(r);
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let g = Effects::subject(|| unsafe { _wtoi(p) as i64 });
        let w = Effects::reference(|| unsafe { ref__wtoi(p) as i64 });
        let tags = parse_tags(&w);
        (Obs::new(g), Obs::new(w), showw(&a, null), tags)
    });
    grade("_wtol", &["front-door", "ERANGE", "clean"], PARSE_ITERS, |r| {
        let a = gen_wparse_input(r);
        let null = r.below(32) == 0;
        let p = if null { core::ptr::null() } else { a.as_ptr() };
        let g = Effects::subject(|| unsafe { _wtol(p) as i64 });
        let w = Effects::reference(|| unsafe { ref__wtol(p) as i64 });
        let tags = parse_tags(&w);
        (Obs::new(g), Obs::new(w), showw(&a, null), tags)
    });
    grade("_wtoi64", &["clean", "wrapped"], PARSE_ITERS, |r| {
        let a = gen_wparse_input(r);
        let g = Effects::subject(|| unsafe { _wtoi64(a.as_ptr()) });
        let w = Effects::reference(|| unsafe { ref__wtoi64(a.as_ptr()) });
        let mut tags = parse_tags(&w);
        if wide_decimal_overflows(&a) {
            tags.push("wrapped");
        }
        (Obs::new(g), Obs::new(w), showw(&a, false), tags)
    });

    if substrate_bad > 0 {
        println!(
            "  [substrate] glibc's sprintf and msvcrt's disagreed on {substrate_bad} of the \
             {CASE_ITERS} `%.*le` probes — see the `sprintf %.*le [SUBSTRATE]` row."
        );
    }
    let (zero_ret, with_err) = probe_insufficient_buffer();
    if with_err > 0 {
        record(
            "_wctomb_s_l ERANGE arm".to_string(),
            V::Match,
            format!(
                "reachability MEASURED, not assumed: {with_err} of {zero_ret} zero-returns \
                 across 3 codepages x 9363 characters\n        x 4 sizes carry \
                 ERROR_INSUFFICIENT_BUFFER, so `_wctomb_s_l`'s insufficient-buffer arm is \
                 live and the `[CP932]` rows\n        above grade it (they require ERANGE)."
            ),
        );
    } else {
        record(
            "_wctomb_s_l ERANGE arm".to_string(),
            V::Diverge,
            "the insufficient-buffer path became UNREACHABLE — the shared \
             WideCharToMultiByte never returns 0 with\n        ERROR_INSUFFICIENT_BUFFER \
             for a one-character conversion any more, so `_wctomb_s_l`'s ERANGE arm and the\n\
             \x20       whole last-error plumbing under it are no longer graded, however \
             green the rows look. Restore the\n        model's \"never truncate\" behaviour \
             (refimpl.c, WideCharToMultiByte's DBCS branch)."
                .to_string(),
        );
    }

    println!(
        "  [I10_OUTPUT exclusions] {i10_skipped_inf} inputs skipped for a non-finite `d` \
         (the two sprintf engines print those differently — that is the substrate, not the\n   \
         port); {i10_skipped_smash} skipped as msvcrt's own INT_MAX-prec stack smash \
         (`prec == 1 && flag&1 && d in {{0, inf}}`), which the Rust reproduces exactly and\n   \
         which no harness can execute. Both are FINDINGS, not narrowings — see i10_safe()."
    );
}

/// `wcstol`'s 3-argument shape, adapted to the 4-argument `wide_clamp_row!` template. The
/// extra `_locale_t` is dropped, which is the point: this proves the forwarder passes NULL.
#[allow(non_snake_case)]
unsafe fn wcstol_fwd(s: *const u16, e: *mut *mut u16, b: c_int, _l: LocT) -> i32 {
    wcstol(s, e, b)
}
unsafe fn ref_wcstol_fwd(s: *const u16, e: *mut *mut u16, b: c_int, _l: LocT) -> i32 {
    ref_wcstol(s, e, b)
}
unsafe fn wcstoul_fwd(s: *const u16, e: *mut *mut u16, b: c_int, _l: LocT) -> u32 {
    wcstoul(s, e, b)
}
unsafe fn ref_wcstoul_fwd(s: *const u16, e: *mut *mut u16, b: c_int, _l: LocT) -> u32 {
    ref_wcstoul(s, e, b)
}

/// Is `_wctomb_s_l`'s insufficient-buffer arm REACHABLE in this harness at all?
///
///     mblen = WideCharToMultiByte(cp, 0, &wch, 1, mbchar, size, NULL, perror);
///     if (!mblen || error) {
///         if (!mblen && GetLastError() == ERROR_INSUFFICIENT_BUFFER) { ... ERANGE ... }
///
/// That arm — the one the shared last-error plumbing exists for — needs the converter to
/// return **0** for a ONE-character conversion. This does not argue about whether it can;
/// it sweeps the whole (codepage x character x size) space the rows use and COUNTS.
///
/// It is a permanent row rather than a comment because the answer is a property of
/// refimpl.c's `WideCharToMultiByte`, which can change under us. Today it is 0 and the
/// harness says so out loud; the day the model gains real Windows' "insufficient buffer
/// returns 0, never truncates" behaviour, this row's count goes positive and the gap is
/// closed by the rows above with no further work.
unsafe fn probe_insufficient_buffer() -> (u64, u64) {
    let mut zero_ret = 0u64;
    let mut with_err = 0u64;
    let mut buf = [0u8; 8];
    for cp in [1252u32, 65001, 932] {
        let perror_used = cp != 65001;
        for wch in (0u32..=0xFFFF).step_by(7) {
            let w = wch as u16;
            for size in 0..4i32 {
                let mut err: c_int = 0;
                let p = if perror_used { &mut err as *mut c_int } else { core::ptr::null_mut() };
                let n = WideCharToMultiByte(cp, 0, &w, 1, buf.as_mut_ptr(), size, core::ptr::null(), p);
                if n == 0 {
                    zero_ret += 1;
                    if GetLastError() == 122 {
                        with_err += 1;
                    }
                }
            }
        }
    }
    (zero_ret, with_err)
}

unsafe fn capture_cstr(p: *const u8) -> Vec<u8> {
    if p.is_null() {
        return vec![0xFF];
    }
    let mut v = Vec::new();
    let mut i = 0;
    loop {
        let b = *p.add(i);
        v.push(b);
        if b == 0 {
            break;
        }
        i += 1;
        if i > 4096 {
            break;
        }
    }
    v
}

// ══════════════════════════════════════════════════════════════════════════════════════

unsafe fn run() {
    // ── axis 1: the default "C" locale (lc_handle == 0), raw codepage ───────────────
    narrow_parse_rows!("");
    narrow_case_rows!("");
    wide_parse_rows!("");
    wide_case_rows!("");
    wmb_rows!("", WMB_S_REQ, WMB_P_REQ);

    // ── axis 2: a REAL NLS locale — path 2, through _towlower_l and the Latin-1 tables ──
    ref_set_nls(1);
    subj_set_nls(1);
    narrow_parse_rows!(" [NLS]");
    narrow_case_rows!(" [NLS]");
    wide_parse_rows!(" [NLS]");
    wide_case_rows!(" [NLS]");
    wmb_rows!(" [NLS]", WMB_S_REQ, WMB_P_REQ);
    ref_set_nls(0);
    subj_set_nls(0);

    // ── axis 3: a non-zero lc_codepage — the WideCharToMultiByte half of the converter
    //    family, which is DEAD at codepage 0 and would otherwise ship entirely ungraded ──
    set_codepage(1252, 1);
    wmb_rows!(" [CP1252]", WMB_S_REQ_1252, WMB_P_REQ);
    set_codepage(65001, 4); // CP_UTF8: the `pused_default = NULL` arm
    wmb_rows!(" [CP_UTF8]", WMB_S_REQ_UTF8, WMB_P_REQ_UTF8);
    set_codepage(932, 2); // a real DBCS codepage, through iconv
    wmb_rows!(" [CP932]", WMB_S_REQ, WMB_P_REQ);
    set_codepage(0, 1);

    // ── locale-blind ────────────────────────────────────────────────────────────────
    blind_rows();
}

fn main() {
    unsafe { assert_shims_live() }
    unsafe { run() }

    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut d, mut u) = (0, 0, 0);
    println!("\n=== wave 16: msvcrt-crt-rust (SHIPPING) vs GENUINE msvcrt string.c + wcs.c ===\n");
    for r in res {
        let tag = match r.v {
            V::Match => {
                m += 1;
                "MATCH       ✓"
            }
            V::Diverge => {
                d += 1;
                "DIVERGE     ✗"
            }
            V::Unexercised => {
                u += 1;
                "UNEXERCISED ✗"
            }
        };
        println!("  {:<26} {}", r.name, tag);
        // The coverage line is printed for GREEN rows too, deliberately: "MATCH" alone
        // does not say which contracts ran, and a row whose reached-set silently shrinks
        // is on its way to being a green that proves nothing.
        if !r.note.is_empty() {
            println!("        {}", r.note);
        }
    }
    let cases = unsafe { CASES };
    println!("\n  {m} MATCH   {d} DIVERGE   {u} UNEXERCISED   (of {} rows)", res.len());

    if d + u > 0 {
        println!("\n  ✗ FAIL — these functions are FLIPPED, so a divergence means msvcrt.dll is");
        println!("    shipping a wrong answer, and an UNEXERCISED row means the green above is");
        println!("    worthless for the branch it exists to test.\n");
        println!("{cases} wave16 cases passed");
        std::process::exit(1);
    }

    println!("\n{cases} wave16 cases passed");
}
