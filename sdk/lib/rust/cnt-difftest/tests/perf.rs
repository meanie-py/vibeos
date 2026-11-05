#![no_builtins] // the shims must be CALLED, not folded to LLVM builtins (see diff.rs)
//! Throughput of cnt's compare/search shims against the genuine libcntpr C (build.rs's
//! `ref_*`, cc -O2), in one process on the same data, so a loaded host moves both sides.
//!
//! What it answers is "how far off the C is each body", not an absolute cost. Run it with the
//! flags cnt ships with (libcntpr.cmake), or it measures a build nobody runs:
//!   RUSTFLAGS="-C no-prepopulate-passes" cargo test --release -p cnt-difftest --test perf
//! The C side is whatever the host cc made of the tcs*.h loops, which is what the C build
//! would have shipped modulo target.

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
}

use cnt_rust::cabi::{
    _stricmp, _wcsicmp, strchr, strcmp, strlen, strncmp, wcschr, wcscmp, wcslen, wcsncmp,
};

/// Best-of-BEST_OF ns per call, alternating sides each round so drift hits both.
fn pair(name: &str, len: usize, iters: u32, c: &dyn Fn(), r: &dyn Fn()) -> f64 {
    const BEST_OF: usize = 7;
    let (mut bc, mut br) = (f64::MAX, f64::MAX);
    for _ in 0..BEST_OF {
        for (f, best) in [(c, &mut bc), (r, &mut br)] {
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
    }
    println!("slowest vs C: {} at {:.1}x", worst.1, worst.0);
}
