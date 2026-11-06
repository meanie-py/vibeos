#![no_builtins] // the shims must be CALLED, not folded to LLVM builtins (see diff.rs)
//! Throughput of cnt's compare/search shims against the genuine libcntpr C (build.rs's
//! `ref_*`, cc -O2), in one process on the same data, so a loaded host moves both sides.
//!
//! What it answers is "how far off the C is each body", not an absolute cost. Run it with the
//! flags cnt ships with (libcntpr.cmake), or it measures a build nobody runs:
//!   RUSTFLAGS="-C no-prepopulate-passes" cargo test --release -p cnt-difftest --test perf
//! The C side is whatever the host cc made of the tcs*.h loops, which is what the C build
//! would have shipped modulo target.

use core::ffi::c_void;
use std::hint::black_box;
use std::time::Instant;

extern "C" {
    fn ref_strcmp(a: *const u8, b: *const u8) -> i32;
    fn ref_strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_strchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_stricmp(a: *const u8, b: *const u8) -> i32;
    fn ref_wcscmp(a: *const u16, b: *const u16) -> i32;
    fn ref_wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref_wcschr(s: *const u16, c: u16) -> *mut u16;
    fn ref_wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn ref_strlen(s: *const u8) -> usize;
    fn ref_wcslen(s: *const u16) -> usize;
    fn ref_strrchr(s: *const u8, c: i32) -> *mut u8;
    fn ref_strstr(h: *const u8, n: *const u8) -> *mut u8;
    fn ref_strspn(s: *const u8, a: *const u8) -> usize;
    fn ref_strcspn(s: *const u8, a: *const u8) -> usize;
    fn ref_strpbrk(s: *const u8, a: *const u8) -> *mut u8;
    fn ref_wcsstr(h: *const u16, n: *const u16) -> *mut u16;
    fn ref_wcsspn(s: *const u16, a: *const u16) -> usize;
    fn ref_wcscspn(s: *const u16, a: *const u16) -> usize;
    fn ref_memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn ref_strnlen(s: *const u8, cap: usize) -> usize;
    fn ref_wcsnlen(s: *const u16, n: usize) -> usize;
    fn ref_memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref_memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref_memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_memmove(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_memset(d: *mut c_void, v: i32, n: usize) -> *mut c_void;
    fn ref_strncpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn ref_strlwr(s: *mut u8) -> *mut u8;
    fn ref_wcslwr(s: *mut u16) -> *mut u16;
    fn ref_strtol(n: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn ref_wcstol(n: *const u16, e: *mut *mut u16, b: i32) -> i32;
}

use cnt_rust::cabi::{
    _memicmp, _stricmp, _strlwr, _wcsicmp, _wcslwr, memchr, memcmp, memcpy, memmove, memset,
    strchr, strcmp, strcspn, strlen, strncmp, strncpy, strnlen, strpbrk, strrchr, strspn, strstr,
    strtol, wcschr, wcscmp, wcscspn, wcslen, wcsncmp, wcsnlen, wcsspn, wcsstr, wcstol,
};

/// Best-of-BEST_OF ns per call, alternating sides each round so drift hits both.
/// On a loaded host the nanoseconds are noise; count instructions instead, one row and one
/// side at a time, which load cannot move:
///   CNT_PERF_ONLY=strtol CNT_PERF_SIDE=cnt perf stat -e instructions:u <the perf binary>
/// (and `CNT_PERF_SIDE=c`, and `CNT_PERF_ONLY=none` for the fixed setup cost). Each selected
/// side runs `iters * 7` calls per length.
/// On a loaded host the nanoseconds are noise; count instructions instead, one row and one
/// side at a time, which load cannot move:
///   CNT_PERF_ONLY=strtol CNT_PERF_SIDE=cnt perf stat -e instructions:u <the perf binary>
/// (and `CNT_PERF_SIDE=c`, and `CNT_PERF_ONLY=none` for the fixed cost of the run). Each
/// selected side makes `iters * 7` calls per length.
fn selected(name: &str, side: &str) -> bool {
    // Not `env::var`: this binary exports cnt's `strlen`/`strncmp`, and glibc's `getenv`
    // binds to them, so the lookup came back empty. `vars_os` walks `environ` in Rust.
    let get = |k: &str| std::env::vars_os().find(|(n, _)| n == k).map(|(_, v)| v);
    let row = get("CNT_PERF_ONLY").map_or(true, |o| o == name);
    row && get("CNT_PERF_SIDE").map_or(true, |s| s == side)
}

fn pair(name: &str, len: usize, iters: u32, c: &dyn Fn(), r: &dyn Fn()) -> f64 {
    const BEST_OF: usize = 7;
    let (mut bc, mut br) = (f64::MAX, f64::MAX);
    if !selected(name, "c") && !selected(name, "cnt") {
        return 0.0;
    }
    for _ in 0..BEST_OF {
        for (f, best, side) in [(c, &mut bc, "c"), (r, &mut br, "cnt")] {
            if !selected(name, side) {
                continue;
            }
            let t = Instant::now();
            for _ in 0..iters {
                f();
            }
            *best = best.min(t.elapsed().as_nanos() as f64 / iters as f64);
        }
    }
    let ratio = br / bc;
    println!("{name:>9} len {len:>4}: cnt {br:>8.1} ns  C {bc:>7.1} ns  {ratio:>5.1}x");
    ratio
}

fn main() {
    let mut worst = (0.0f64, String::new());
    for &len in &[8usize, 32, 256] {
        let iters = (4_000_000 / (len as u32 + 8)).max(2000);
        // equal strings: the compare runs to the terminator, the search misses
        let a: Vec<u8> = (0..len).map(|i| b'a' + (i % 23) as u8).chain([0]).collect();
        let b = a.clone();
        let w: Vec<u16> = a.iter().map(|&x| x as u16).collect();
        let v = w.clone();
        let (pa, pb, pw, pv) = (a.as_ptr(), b.as_ptr(), w.as_ptr(), v.as_ptr());
        // search operands that never match, so every scan runs the whole string: a needle
        // whose first byte occurs every 23 bytes but whose tail never does, a 4-byte set
        // absent from the haystack, and the haystack's own alphabet for the *spn rows
        let needle = b"ab!\0";
        let wneedle: Vec<u16> = needle.iter().map(|&x| x as u16).collect();
        let miss = b"!#$%\0";
        let wmiss: Vec<u16> = miss.iter().map(|&x| x as u16).collect();
        let all: Vec<u8> = (0..23u8).map(|i| b'a' + i).chain([0]).collect();
        let wall: Vec<u16> = all.iter().map(|&x| x as u16).collect();
        let (pn, pwn, pm, pwm, pl, pwl) = (
            needle.as_ptr(),
            wneedle.as_ptr(),
            miss.as_ptr(),
            wmiss.as_ptr(),
            all.as_ptr(),
            wall.as_ptr(),
        );
        macro_rules! row {
            ($n:literal, $cnt:expr, $c:expr) => {{
                let k = pair(
                    $n,
                    len,
                    iters,
                    &|| unsafe {
                        black_box($c);
                    },
                    &|| unsafe {
                        black_box($cnt);
                    },
                );
                if k > worst.0 {
                    worst = (k, format!("{} len {}", $n, len));
                }
            }};
        }
        row!("strlen", strlen(black_box(pa)), ref_strlen(black_box(pa)));
        row!("wcslen", wcslen(black_box(pw)), ref_wcslen(black_box(pw)));
        row!(
            "strcmp",
            strcmp(black_box(pa), pb),
            ref_strcmp(black_box(pa), pb)
        );
        row!(
            "strncmp",
            strncmp(black_box(pa), pb, len),
            ref_strncmp(black_box(pa), pb, len)
        );
        row!(
            "_stricmp",
            _stricmp(black_box(pa), pb),
            ref_stricmp(black_box(pa), pb)
        );
        row!(
            "strchr",
            strchr(black_box(pa), b'!' as i32),
            ref_strchr(black_box(pa), b'!' as i32)
        );
        row!(
            "wcscmp",
            wcscmp(black_box(pw), pv),
            ref_wcscmp(black_box(pw), pv)
        );
        row!(
            "wcsncmp",
            wcsncmp(black_box(pw), pv, len),
            ref_wcsncmp(black_box(pw), pv, len)
        );
        row!(
            "_wcsicmp",
            _wcsicmp(black_box(pw), pv),
            ref_wcsicmp(black_box(pw), pv)
        );
        row!(
            "wcschr",
            wcschr(black_box(pw), b'!' as u16),
            ref_wcschr(black_box(pw), b'!' as u16)
        );
        row!(
            "strrchr",
            strrchr(black_box(pa), b'!' as i32),
            ref_strrchr(black_box(pa), b'!' as i32)
        );
        row!(
            "strstr",
            strstr(black_box(pa), pn),
            ref_strstr(black_box(pa), pn)
        );
        row!(
            "strspn",
            strspn(black_box(pa), pl),
            ref_strspn(black_box(pa), pl)
        );
        row!(
            "strcspn",
            strcspn(black_box(pa), pm),
            ref_strcspn(black_box(pa), pm)
        );
        row!(
            "strpbrk",
            strpbrk(black_box(pa), pm),
            ref_strpbrk(black_box(pa), pm)
        );
        row!(
            "wcsstr",
            wcsstr(black_box(pw), pwn),
            ref_wcsstr(black_box(pw), pwn)
        );
        row!(
            "wcsspn",
            wcsspn(black_box(pw), pwl),
            ref_wcsspn(black_box(pw), pwl)
        );
        row!(
            "wcscspn",
            wcscspn(black_box(pw), pwm),
            ref_wcscspn(black_box(pw), pwm)
        );
        row!(
            "memchr",
            memchr(black_box(pa).cast(), b'!' as i32, len),
            ref_memchr(black_box(pa).cast(), b'!' as i32, len)
        );
        // the copy, case-map and parse families, for what is still a slice loop
        let mut d = vec![0u8; len + 1];
        let pd = d.as_mut_ptr();
        let (pam, pwm2) = (a.as_ptr() as *mut u8, w.as_ptr() as *mut u16);
        let num = b"-123456789\0";
        let wnum: Vec<u16> = num.iter().map(|&x| x as u16).collect();
        let (pnum, pwnum) = (num.as_ptr(), wnum.as_ptr());
        row!(
            "memcmp",
            memcmp(black_box(pa).cast(), pb.cast(), len),
            ref_memcmp(black_box(pa).cast(), pb.cast(), len)
        );
        row!(
            "_memicmp",
            _memicmp(black_box(pa).cast(), pb.cast(), len),
            ref_memicmp(black_box(pa).cast(), pb.cast(), len)
        );
        row!(
            "memcpy",
            memcpy(black_box(pd).cast(), pa.cast(), len),
            ref_memcpy(black_box(pd).cast(), pa.cast(), len)
        );
        row!(
            "memmove",
            memmove(black_box(pd).cast(), pa.cast(), len),
            ref_memmove(black_box(pd).cast(), pa.cast(), len)
        );
        row!(
            "memset",
            memset(black_box(pd).cast(), 0x41, len),
            ref_memset(black_box(pd).cast(), 0x41, len)
        );
        row!(
            "strncpy",
            strncpy(black_box(pd), pa, len + 1),
            ref_strncpy(black_box(pd), pa, len + 1)
        );
        // already lower case, so both sides scan and rewrite the same bytes every round
        row!(
            "_strlwr",
            _strlwr(black_box(pam)),
            ref_strlwr(black_box(pam))
        );
        row!(
            "_wcslwr",
            _wcslwr(black_box(pwm2)),
            ref_wcslwr(black_box(pwm2))
        );
        row!(
            "strtol",
            strtol(black_box(pnum), core::ptr::null_mut(), 10),
            ref_strtol(black_box(pnum), core::ptr::null_mut(), 10)
        );
        row!(
            "wcstol",
            wcstol(black_box(pwnum), core::ptr::null_mut(), 10),
            ref_wcstol(black_box(pwnum), core::ptr::null_mut(), 10)
        );
        row!(
            "strnlen",
            strnlen(black_box(pa), len + 1),
            ref_strnlen(black_box(pa), len + 1)
        );
        row!(
            "wcsnlen",
            wcsnlen(black_box(pw), len + 1),
            ref_wcsnlen(black_box(pw), len + 1)
        );
    }
    println!("slowest vs C: {} at {:.1}x", worst.1, worst.0);
}
