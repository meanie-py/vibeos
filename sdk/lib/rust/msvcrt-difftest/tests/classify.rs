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
    fn subj_errno_get() -> i32;         // subjectimpl.c (the subject's)
    fn subj_errno_set(v: i32);
    fn subj_invalid_pmt_count() -> i32;
    fn subj_invalid_pmt_reset();
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
    check_int("atoi", false, |r| { let a = gen_str(r, 18); (atoi(a.as_ptr()) as i64, ref_atoi(a.as_ptr()) as i64, format!("{a:?}")) });
    check_int("atol", false, |r| { let a = gen_str(r, 18); (atol(a.as_ptr()) as i64, ref_atol(a.as_ptr()) as i64, format!("{a:?}")) });
    check_int("_atoi64", false, |r| { let a = gen_str(r, 24); (_atoi64(a.as_ptr()), ref__atoi64(a.as_ptr()), format!("{a:?}")) });
    check_int("strtol", false, |r| { let a = gen_str(r, 20); let b = [0i32, 2, 8, 10, 16, 36][r.below(6)];
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = strtol(a.as_ptr(), &mut e1, b); let w = ref_strtol(a.as_ptr(), &mut e2, b);
        let (o1, o2) = (e1 as i64 - a.as_ptr() as i64, e2 as i64 - a.as_ptr() as i64);
        (((g as i64) << 8) ^ o1, ((w as i64) << 8) ^ o2, format!("base={b} {a:?} -> {g}/{w} end+{o1}/{o2}")) });
    check_int("strtoul", false, |r| { let a = gen_str(r, 20); let b = [0i32, 2, 8, 10, 16, 36][r.below(6)];
        let (mut e1, mut e2) = (core::ptr::null_mut(), core::ptr::null_mut());
        let g = strtoul(a.as_ptr(), &mut e1, b); let w = ref_strtoul(a.as_ptr(), &mut e2, b);
        let (o1, o2) = (e1 as i64 - a.as_ptr() as i64, e2 as i64 - a.as_ptr() as i64);
        (((g as i64) << 8) ^ o1, ((w as i64) << 8) ^ o2, format!("base={b} {a:?} -> {g}/{w} end+{o1}/{o2}")) });

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
}
