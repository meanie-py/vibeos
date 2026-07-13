#![no_builtins]
//! Throughput gate: a flip must not be materially SLOWER than the C it replaces.
//!
//! WHY THIS EXISTS. `classify.rs` grades **behaviour**, so a body that returns
//! the right bytes scores MATCH no matter how slowly it gets there. That blind
//! spot shipped a real regression: msvcrt wave 2 flipped `memcpy`/`memset` —
//! served to EVERY user-mode caller in the system — to bodies that were
//! semantically perfect and orders of magnitude slower (`memset` was a per-byte
//! `core::ptr::write_volatile` loop, and `write_volatile` is unvectorizable *by
//! construction*; under `-C no-prepopulate-passes` nothing inlined either, so it
//! was literally a function call per byte). Mainline's net suite went 71s ->
//! 208s and `nettest.exe` blew its 120s marker timeout. Wave 2 was reverted.
//!
//! So: measure. Compare each flipped hot primitive against the GENUINE msvcrt C
//! body (the `ref_*` symbols build.rs compiles from `dll/win32/msvcrt/string.c`)
//! at sizes that matter, and FAIL the flip if it is materially slower.
//!
//! This is a ratio test, not an absolute one — it is meaningful on a noisy
//! shared host precisely because both sides run in the same process, on the same
//! data, under the same load.

extern crate msvcrt_crt_rust;

use core::ffi::c_void;
use std::time::Instant;

extern "C" {
    // the genuine msvcrt C bodies
    fn ref_memset(d: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn ref_memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn ref_strlen(s: *const u8) -> usize;
    fn ref_memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn ref_memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
    // the shipping Rust (msvcrt-crt-rust)
    fn memset(d: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const u8) -> usize;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
}

/// A flip slower than this multiple of the C it replaces FAILS the gate.
/// These are system-wide CRT primitives; "a bit slower" compounds everywhere.
const MAX_SLOWDOWN: f64 = 2.0;

const SIZES: &[usize] = &[16, 4096, 1 << 20];

fn bench(label: &str, n: usize, mut f: impl FnMut()) -> f64 {
    // size the iteration count so every case does comparable total work
    let iters = core::cmp::max(1, (1 << 26) / n);
    // warm up (page-in, branch predictors, i-cache)
    for _ in 0..core::cmp::min(iters, 64) {
        f();
    }
    let t = Instant::now();
    for _ in 0..iters {
        f();
    }
    let secs = t.elapsed().as_secs_f64();
    let bytes = (iters * n) as f64;
    let mbps = bytes / secs / (1024.0 * 1024.0);
    let _ = label;
    mbps
}

struct Row {
    name: String,
    rust: f64,
    c: f64,
}

fn main() {
    let mut rows: Vec<Row> = Vec::new();

    for &n in SIZES {
        let mut dst = vec![0u8; n];
        let mut dst2 = vec![0u8; n];
        let src = vec![0x5Au8; n];

        let dp = dst.as_mut_ptr() as *mut c_void;
        let d2p = dst2.as_mut_ptr() as *mut c_void;
        let sp = src.as_ptr() as *const c_void;

        unsafe {
            let r = bench("memset", n, || {
                memset(dp, 0x41, n);
            });
            let c = bench("ref_memset", n, || {
                ref_memset(d2p, 0x41, n);
            });
            rows.push(Row { name: format!("memset/{n}"), rust: r, c });

            let r = bench("memcpy", n, || {
                memcpy(dp, sp, n);
            });
            let c = bench("ref_memcpy", n, || {
                ref_memcpy(d2p, sp, n);
            });
            rows.push(Row { name: format!("memcpy/{n}"), rust: r, c });

            let r = bench("memcmp", n, || {
                core::hint::black_box(memcmp(sp, sp, n));
            });
            let c = bench("ref_memcmp", n, || {
                core::hint::black_box(ref_memcmp(sp, sp, n));
            });
            rows.push(Row { name: format!("memcmp/{n}"), rust: r, c });

            let r = bench("memchr", n, || {
                core::hint::black_box(memchr(sp, 0xFF, n));
            });
            let c = bench("ref_memchr", n, || {
                core::hint::black_box(ref_memchr(sp, 0xFF, n));
            });
            rows.push(Row { name: format!("memchr/{n}"), rust: r, c });

            // strlen wants a NUL-terminated buffer of length n
            let mut s = vec![b'x'; n + 1];
            s[n] = 0;
            let strp = s.as_ptr();
            let r = bench("strlen", n, || {
                core::hint::black_box(strlen(strp));
            });
            let c = bench("ref_strlen", n, || {
                core::hint::black_box(ref_strlen(strp));
            });
            rows.push(Row { name: format!("strlen/{n}"), rust: r, c });
        }
    }

    println!("\n=== msvcrt Rust vs GENUINE C — throughput (MiB/s, higher is better) ===\n");
    println!("  {:<16} {:>12} {:>12} {:>10}", "case", "rust", "C", "slowdown");
    let mut bad: Vec<String> = Vec::new();
    for r in &rows {
        let slow = if r.rust > 0.0 { r.c / r.rust } else { f64::INFINITY };
        let tag = if slow > MAX_SLOWDOWN {
            bad.push(format!("{} ({:.1}x slower)", r.name, slow));
            "  ✗ TOO SLOW"
        } else {
            "  ✓"
        };
        println!(
            "  {:<16} {:>12.0} {:>12.0} {:>9.2}x{}",
            r.name, r.rust, r.c, slow, tag
        );
    }

    if bad.is_empty() {
        println!("\n  every flipped primitive is within {MAX_SLOWDOWN}x of the C it replaces ✓\n");
    } else {
        println!("\n  SLOWER THAN THE C IT REPLACES: {bad:?}");
        println!("  These are system-wide CRT primitives — a semantically perfect but");
        println!("  slow body is still a regression (wave 2: net suite 71s -> 208s,");
        println!("  nettest.exe blew its 120s timeout). Fix the body or drop the flip.\n");
        std::process::exit(1);
    }
}
