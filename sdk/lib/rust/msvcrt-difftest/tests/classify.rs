#![no_builtins] // cnt-rust's #[no_mangle] libc-named shims must be CALLED, not folded to LLVM builtins.
//! Classify each cnt-rust CRT shim against the GENUINE compiled msvcrt
//! reference (`dll/win32/msvcrt/string.c`, compiled by build.rs as `ref_*`).
//!
//! Output is a verdict table, not a pass/fail: the point is to learn WHICH
//! functions cnt-rust can safely take over in msvcrt.dll and which have a
//! divergent contract (cnt-rust tracks *libcntpr*; msvcrt is *Wine*-derived).
//!
//!   MATCH      — identical results across the whole fuzz domain → safe to flip
//!   SIGN-ONLY  — comparison fns: same sign, different magnitude (see note)
//!   DIVERGE    — a real contract difference (counterexample printed) → DO NOT FLIP
//!
//! Reference caveats (from src/lib.rs) that this driver respects:
//!   * locale is the default "C" locale (msvcrt pre-setlocale) — the genuine
//!     ASCII paths of string.c are what we grade against.
//!   * `ref_strncmp`/`ref_strcmp` return the raw byte difference (32-bit arm);
//!     that's why SIGN-ONLY is reported separately rather than folded into MATCH.

// Force cnt-rust's objects into the link. WITHOUT this, rustc omits the unused
// rlib entirely and the shared names (strlen/strcmp/memcpy/…) silently bind to
// GLIBC — the harness would then grade glibc against msvcrt and report a
// meaningless "MATCH". `assert_shims_live()` below proves the binding at runtime.
extern crate cnt_rust;

use core::ffi::c_void;

// ── the genuine msvcrt C reference (build.rs, from the real string.c) ──
extern "C" {
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
    fn _stricmp(a: *const u8, b: *const u8) -> i32;
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _strlwr(s: *mut u8) -> *mut u8;
    fn _strupr(s: *mut u8) -> *mut u8;
    fn _strrev(s: *mut u8) -> *mut u8;
    fn _strset(s: *mut u8, c: i32) -> *mut u8;
    fn _strnset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    fn memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn memset(d: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn _memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn _memccpy(d: *mut c_void, s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn atoi(s: *const u8) -> i32;
    fn atol(s: *const u8) -> i32;
    fn _atoi64(s: *const u8) -> i64;
    fn strtol(s: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn strtoul(s: *const u8, e: *mut *mut u8, b: i32) -> u32;
}

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

/// Prove the shared-name shims resolve to CNT-RUST, not glibc. cnt-rust's
/// `cstr_slice` NULL-guards (NULL → empty slice), so its `strlen(NULL)` is 0;
/// glibc's would fault. If this ever silently "passes" against glibc the whole
/// table is worthless, so it is a hard gate, not a warning.
unsafe fn assert_shims_live() {
    let n = strlen(core::ptr::null());
    assert_eq!(n, 0, "strlen(NULL)!=0 — shared-name shims are NOT cnt-rust's");
    // _strrev has no glibc counterpart: linking at all proves cnt-rust is present.
    let mut b = *b"ab\0";
    let _ = _strrev(b.as_mut_ptr());
    assert_eq!(&b, b"ba\0", "cnt-rust _strrev misbehaved — bad link");
    println!("  [self-check] cnt-rust shims are live (strlen(NULL)==0, _strrev ok)");
}

fn main() {
    unsafe { assert_shims_live() }
    unsafe { run() }
    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut s, mut d) = (0, 0, 0);
    println!("\n=== cnt-rust vs GENUINE msvcrt/string.c — flip safety ===\n");
    for r in res {
        let tag = match r.v { V::Match => { m += 1; "MATCH     ✓ safe to flip" }
                              V::SignOnly => { s += 1; "SIGN-ONLY ⚠ review" }
                              V::Diverge => { d += 1; "DIVERGE   ✗ DO NOT FLIP" } };
        println!("  {:<12} {}", r.name, tag);
        if !r.note.is_empty() { println!("        {}", r.note); }
    }
    println!("\n  {m} MATCH   {s} SIGN-ONLY   {d} DIVERGE   (of {} tested)\n", res.len());
}

unsafe fn run() {
    // ---- pure readers ----
    check_int("strlen", false, |r| { let a = gen_str(r, 40); (strlen(a.as_ptr()) as i64, ref_strlen(a.as_ptr()) as i64, String::new()) });
    check_int("strnlen", false, |r| { let a = gen_str(r, 40); let n = r.below(45); (strnlen(a.as_ptr(), n) as i64, ref_strnlen(a.as_ptr(), n) as i64, format!("n={n}")) });
    check_int("strcmp", true, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) };
        (strcmp(a.as_ptr(), b.as_ptr()) as i64, ref_strcmp(a.as_ptr(), b.as_ptr()) as i64, format!("{a:?} vs {b:?}")) });
    check_int("strncmp", true, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) }; let n = r.below(22);
        (strncmp(a.as_ptr(), b.as_ptr(), n) as i64, ref_strncmp(a.as_ptr(), b.as_ptr(), n) as i64, format!("n={n} {a:?} vs {b:?}")) });
    check_int("_stricmp", true, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) };
        (_stricmp(a.as_ptr(), b.as_ptr()) as i64, ref__stricmp(a.as_ptr(), b.as_ptr()) as i64, format!("{a:?} vs {b:?}")) });
    check_int("_strnicmp", true, |r| { let a = gen_str(r, 20); let b = if r.below(3) == 0 { a.clone() } else { gen_str(r, 20) }; let n = r.below(22);
        (_strnicmp(a.as_ptr(), b.as_ptr(), n) as i64, ref__strnicmp(a.as_ptr(), b.as_ptr(), n) as i64, format!("n={n} {a:?} vs {b:?}")) });
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
    check_int("memcmp", true, |r| { let a = gen_str(r, 25); let mut b = a.clone(); if r.below(2) == 0 && !b.is_empty() { let i = r.below(b.len()); b[i] = b[i].wrapping_add(1); } let n = r.below(a.len() + 1);
        (memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, ref_memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, format!("n={n}")) });
    check_int("_memicmp", true, |r| { let a = gen_str(r, 25); let mut b = a.clone(); if r.below(2) == 0 && !b.is_empty() { let i = r.below(b.len()); b[i] = b[i].wrapping_add(32); } let n = r.below(a.len() + 1);
        (_memicmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, ref__memicmp(a.as_ptr().cast(), b.as_ptr().cast(), n) as i64, format!("n={n}")) });
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
}
