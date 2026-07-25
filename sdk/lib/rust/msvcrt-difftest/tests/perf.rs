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

    // wave 4 — the wide hot primitives. wcslen/wcscpy/wcscmp are served to every
    // UNICODE caller in the system (which, in the NT world, is most of them).
    fn ref_wcslen(s: *const u16) -> usize;
    fn ref_wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn ref_wcscmp(a: *const u16, b: *const u16) -> i32;
    // ── the case-insensitive + case-mapping families (added after they cost a 3x boot) ──
    fn _stricmp(a: *const u8, b: *const u8) -> i32;
    fn ref__stricmp(a: *const u8, b: *const u8) -> i32;
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref__strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn ref__wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn ref__wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn towlower(c: u16) -> u16;
    fn ref_towlower(c: u16) -> u16;
    fn tolower(c: i32) -> i32;
    fn ref_tolower(c: i32) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn wcscmp(a: *const u16, b: *const u16) -> i32;

    // The COMPARE family. Added after `wcscmp` shipped 3.8x slow and nothing
    // caught it: the gate only benched memset/memcpy/memcmp/memchr/strlen, so
    // `strcmp` — which has the identical 3-pass slice shape and is served to
    // every caller in the system — went unmeasured through two waves.
    // A perf gate only defends what it BENCHES.
    fn ref_strcmp(a: *const u8, b: *const u8) -> i32;
    fn ref_strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn ref_strcpy(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_strcat(d: *mut u8, s: *const u8) -> *mut u8;
    fn ref_strchr(s: *const u8, c: i32) -> *mut u8;
    fn strcmp(a: *const u8, b: *const u8) -> i32;
    fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn strcpy(d: *mut u8, s: *const u8) -> *mut u8;
    fn strcat(d: *mut u8, s: *const u8) -> *mut u8;
    fn strchr(s: *const u8, c: i32) -> *mut u8;
}

/// A flip slower than this multiple of the C it replaces FAILS the gate.
/// These are system-wide CRT primitives; "a bit slower" compounds everywhere.
const MAX_SLOWDOWN: f64 = 2.0;

const SIZES: &[usize] = &[16, 4096, 1 << 20];

/// One timed round: MiB/s for `iters` calls of `f` over `n` bytes each.
fn round(n: usize, iters: usize, f: &mut impl FnMut()) -> f64 {
    let t = Instant::now();
    for _ in 0..iters {
        f();
    }
    (iters * n) as f64 / t.elapsed().as_secs_f64() / (1024.0 * 1024.0)
}

/// Bench the Rust body and the C body it replaced, INTERLEAVED, and return
/// (rust MiB/s, C MiB/s) as the best round each achieved.
///
/// The interleave is the whole point, and it is what this file's header claims
/// ("meaningful on a noisy shared host") but did not previously deliver. The
/// earlier shape ran three consecutive rounds of Rust, then three of C. Best-of-3
/// only rejects a spike SHORTER than one side's window — and the load this runs
/// under is three cold cross-builds and four booting QEMUs, which stays hot for
/// minutes. A spike covering all three Rust rounds and none of the C ones is
/// reported as a 2x Rust regression, which is exactly how the difftest stage went
/// red on its first gate with nothing wrong in the code.
///
/// Round-robin instead: rust, c, rust, c, rust, c. Now any spike lands on both
/// sides of the same round, and taking each side's best picks the round where the
/// host happened to be quietest — for both. The ratio survives; only the absolute
/// numbers sag.
fn bench_pair(n: usize, mut rust: impl FnMut(), mut c: impl FnMut()) -> (f64, f64) {
    // size the iteration count so every case does comparable total work
    let iters = core::cmp::max(1, (1 << 26) / n);
    // warm up BOTH (page-in, branch predictors, i-cache) before either is timed
    for _ in 0..core::cmp::min(iters, 64) {
        rust();
        c();
    }
    let (mut best_r, mut best_c) = (0.0f64, 0.0f64);
    for _ in 0..3 {
        best_r = best_r.max(round(n, iters, &mut rust));
        best_c = best_c.max(round(n, iters, &mut c));
    }
    (best_r, best_c)
}

struct Row {
    name: String,
    rust: f64,
    c: f64,
}

fn main() {
    // This gate is meaningless outside --release, and worse than meaningless:
    // msvcrt-crt-rust's `opt-level = 3` lives in `[profile.release.package.*]`,
    // so a debug run compares an UNOPTIMIZED Rust body against an -O2 C one and
    // reports ~100x "regressions" that do not exist. A gate that cries wolf in
    // the default profile is a gate people learn to ignore. Say so and stop.
    if cfg!(debug_assertions) {
        println!(
            "\n  perf: SKIPPED — this gate only means anything under --release.\n  \
             msvcrt-crt-rust's opt-level=3 is a [profile.release.package] setting;\n  \
             in debug the Rust side is unoptimized and every case reports a bogus\n  \
             regression. Re-run:  cargo test -p msvcrt-difftest --test perf --release\n"
        );
        return;
    }

    let mut rows: Vec<Row> = Vec::new();

    for &n in SIZES {
        let mut dst = vec![0u8; n];
        let mut dst2 = vec![0u8; n];
        let src = vec![0x5Au8; n];

        let dp = dst.as_mut_ptr() as *mut c_void;
        let d2p = dst2.as_mut_ptr() as *mut c_void;
        let sp = src.as_ptr() as *const c_void;

        unsafe {
            let (r, c) = bench_pair(n, || {
                memset(dp, 0x41, n);
            }, || {
                ref_memset(d2p, 0x41, n);
            });
            rows.push(Row { name: format!("memset/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                memcpy(dp, sp, n);
            }, || {
                ref_memcpy(d2p, sp, n);
            });
            rows.push(Row { name: format!("memcpy/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(memcmp(sp, sp, n));
            }, || {
                core::hint::black_box(ref_memcmp(sp, sp, n));
            });
            rows.push(Row { name: format!("memcmp/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(memchr(sp, 0xFF, n));
            }, || {
                core::hint::black_box(ref_memchr(sp, 0xFF, n));
            });
            rows.push(Row { name: format!("memchr/{n}"), rust: r, c });

            // strlen wants a NUL-terminated buffer of length n
            let mut s = vec![b'x'; n + 1];
            s[n] = 0;
            let strp = s.as_ptr();
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strlen(strp));
            }, || {
                core::hint::black_box(ref_strlen(strp));
            });
            rows.push(Row { name: format!("strlen/{n}"), rust: r, c });

            // ── wave 4: the wide hot primitives ──
            // `n` here is a byte budget, so the wide buffers hold n/2 chars —
            // that keeps the MiB/s column comparable across the narrow and wide
            // rows (same bytes touched per call).
            let wn = n / 2;
            let mut ws = vec![b'x' as u16; wn + 1];
            ws[wn] = 0;
            let wsp = ws.as_ptr();
            let mut wd = vec![0u16; wn + 1];
            let wdp = wd.as_mut_ptr();

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(wcslen(wsp));
            }, || {
                core::hint::black_box(ref_wcslen(wsp));
            });
            rows.push(Row { name: format!("wcslen/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                wcscpy(wdp, wsp);
            }, || {
                ref_wcscpy(wdp, wsp);
            });
            rows.push(Row { name: format!("wcscpy/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(wcscmp(wsp, wsp));
            }, || {
                core::hint::black_box(ref_wcscmp(wsp, wsp));
            });
            rows.push(Row { name: format!("wcscmp/{n}"), rust: r, c });

            // ══ the CASE-INSENSITIVE + CASE-MAPPING families ═══════════════════
            //
            // ADDED AFTER THEY BROKE THE GATE. Wave 11 gave `_wcsicmp` its locale
            // branch and, in doing so, made it re-enter `get_locinfo()` — a TLS lookup
            // — ONCE PER CHARACTER. `classify` said MATCH (the answers were right),
            // `oob_guard` said in-bounds, and the boot went from 39s to 128s, timing
            // out the kernel TDI self-test. The only instrument that could have caught
            // it was this one, and it was not benching a single member of the family.
            //
            // §9.63, restated: A PERF GATE ONLY DEFENDS WHAT IT BENCHES. Every time a
            // new family is flipped, it gets benched here — or the next per-character
            // TLS lookup ships, and we find out from a red gate instead of a red bar.
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_stricmp(strp, strp));
            }, || {
                core::hint::black_box(ref__stricmp(strp, strp));
            });
            rows.push(Row { name: format!("_stricmp/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_strnicmp(strp, strp, n));
            }, || {
                core::hint::black_box(ref__strnicmp(strp, strp, n));
            });
            rows.push(Row { name: format!("_strnicmp/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_wcsicmp(wsp, wsp));
            }, || {
                core::hint::black_box(ref__wcsicmp(wsp, wsp));
            });
            rows.push(Row { name: format!("_wcsicmp/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_wcsnicmp(wsp, wsp, n));
            }, || {
                core::hint::black_box(ref__wcsnicmp(wsp, wsp, n));
            });
            rows.push(Row { name: format!("_wcsnicmp/{n}"), rust: r, c });

            // The scalar mappers, hammered n times — these are what the loops above
            // call per character, so a regression here multiplies through everything.
            let (r, c) = bench_pair(n, || {
                for c in 0..n { core::hint::black_box(towlower((c & 0xFFFF) as u16)); }
            }, || {
                for c in 0..n { core::hint::black_box(ref_towlower((c & 0xFFFF) as u16)); }
            });
            rows.push(Row { name: format!("towlower/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                for c in 0..n { core::hint::black_box(tolower((c & 0xFF) as i32)); }
            }, || {
                for c in 0..n { core::hint::black_box(ref_tolower((c & 0xFF) as i32)); }
            });
            rows.push(Row { name: format!("tolower/{n}"), rust: r, c });

            // ── the compare/copy family (equal inputs: forces the full walk) ──
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strcmp(strp, strp));
            }, || {
                core::hint::black_box(ref_strcmp(strp, strp));
            });
            rows.push(Row { name: format!("strcmp/{n}"), rust: r, c });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strncmp(strp, strp, n));
            }, || {
                core::hint::black_box(ref_strncmp(strp, strp, n));
            });
            rows.push(Row { name: format!("strncmp/{n}"), rust: r, c });

            let mut sd = vec![0u8; n + 1];
            let sdp = sd.as_mut_ptr();
            let (r, c) = bench_pair(n, || {
                strcpy(sdp, strp);
            }, || {
                ref_strcpy(sdp, strp);
            });
            rows.push(Row { name: format!("strcpy/{n}"), rust: r, c });

            // strcat onto an EMPTY dst: the cost is the dst NUL-scan + the copy,
            // which is what the C does too.
            let mut cd = vec![0u8; 2 * n + 2];
            let cdp = cd.as_mut_ptr();
            let (r, c) = bench_pair(n, || {
                *cdp = 0;
                strcat(cdp, strp);
            }, || {
                *cdp = 0;
                ref_strcat(cdp, strp);
            });
            rows.push(Row { name: format!("strcat/{n}"), rust: r, c });

            // absent byte -> forces the full scan
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strchr(strp, b'Z' as i32));
            }, || {
                core::hint::black_box(ref_strchr(strp, b'Z' as i32));
            });
            rows.push(Row { name: format!("strchr/{n}"), rust: r, c });
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
