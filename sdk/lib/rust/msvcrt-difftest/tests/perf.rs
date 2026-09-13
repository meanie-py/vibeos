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
//! data, under the same load. That was not enough on its own: see `bench_pair`
//! (round structure), `calibrate` (the measured error bar) and `PASSES` (a
//! failure has to reproduce) for the three things a gate host broke.
//!
//! WHAT THE C SIDE ACTUALLY IS. `ref_*` is `string.c` compiled by the HOST cc,
//! and cc rewrites some of it. `ref_memset` is not msvcrt's loop — cc
//! idiom-recognized it into its own size-dispatched expansion (four overlapping
//! 8-byte stores for n in 16..32), so that row races a compiler's best effort.
//! `ref_memcpy` beside it was left alone (a tailcall to the hand-rolled
//! `msvcrt_ref_memmove`), so that row races a byte loop. Both readings are
//! legitimate — production C gets the same treatment — but they are not the same
//! question, and `#pragma function(...)` in the ReactOS source does not change
//! this (MSVC-only; cc ignores it). Disassemble a `ref_*` before drawing a
//! conclusion from its ratio.

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
    /// The two sides' locale state. These are SEPARATE: refstub/msvcrt.h renames the
    /// reference's `get_locinfo` into the anti-circularity namespace (see wave16.rs),
    /// so reference and subject each carry their own `lc_handle`. That is exactly why
    /// they can silently disagree, and why `locale_modes()` below reads BOTH.
    fn get_locinfo() -> *mut msvcrt_sys::threadlocaleinfostruct;
    fn msvcrt_ref_get_locinfo() -> *mut msvcrt_sys::threadlocaleinfostruct;
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

    // the C half of the null hypothesis (perfctl.c, compiled twice) — declared
    // here beside the bodies under test, so it is reached the identical way
    fn perf_ctl_copy_c_a(d: *mut u8, s: *const u8) -> *mut u8;
    fn perf_ctl_copy_c_b(d: *mut u8, s: *const u8) -> *mut u8;
    fn perf_ctl_scan_c_a(s: *const u8) -> usize;
    fn perf_ctl_scan_c_b(s: *const u8) -> usize;
    fn perf_ctl_fill_c_a(d: *mut c_void, val: i32, n: usize) -> *mut c_void;
    fn perf_ctl_fill_c_b(d: *mut c_void, val: i32, n: usize) -> *mut c_void;
}

// ── the Rust half of the null hypothesis ────────────────────────────────────
//
// The same three loop shapes the suite benches, each written TWICE. See
// perfctl.c for the full argument; the short version is that a pair must be the
// same language through the same compiler for its expected ratio to be exactly
// 1.00x — benching Rust against C measures rustc-vs-cc codegen, which is the
// thing under test, not harness error.
//
// `#[no_mangle] extern "C"` so these are global interposable symbols reached the
// same GOT-indirect way as every case above; a plain Rust fn would be inlined
// straight into the timing loop and stop being comparable to anything here. The
// file's `#![no_builtins]` keeps LLVM from folding them into calls to the libc
// routines they imitate — and, for the copy loop, into a call to itself.
//
// The `_a`/`_b` bodies are deliberately identical. If some pass ever merges
// them the control reads a vacuous, permanent 1.00x while measuring nothing at
// all, which is why calibrate() asserts the two addresses differ.

macro_rules! ctl_copy {
    ($name:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(d: *mut u8, s: *const u8) -> *mut u8 {
            let (mut dd, mut ss) = (d, s);
            loop {
                let b = *ss;
                *dd = b;
                if b == 0 {
                    return d;
                }
                dd = dd.add(1);
                ss = ss.add(1);
            }
        }
    };
}

macro_rules! ctl_scan {
    ($name:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(s: *const u8) -> usize {
            let mut p = s;
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(s) as usize
        }
    };
}

macro_rules! ctl_fill {
    ($name:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(d: *mut c_void, val: i32, count: usize) -> *mut c_void {
            let p = d as *mut u8;
            let b = val as u8;
            let w = u64::from_ne_bytes([b; 8]);
            let mut i = 0usize;
            while i + 8 <= count {
                (p.add(i) as *mut u64).write_unaligned(w);
                i += 8;
            }
            while i < count {
                *p.add(i) = b;
                i += 1;
            }
            d
        }
    };
}

ctl_copy!(perf_ctl_copy_rs_a);
ctl_copy!(perf_ctl_copy_rs_b);
ctl_scan!(perf_ctl_scan_rs_a);
ctl_scan!(perf_ctl_scan_rs_b);
ctl_fill!(perf_ctl_fill_rs_a);
ctl_fill!(perf_ctl_fill_rs_b);

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
/// Rounds per side. Nine SHORT rounds, not three long ones, for the same total
/// work: best-of-K only reports the truth if at least one round on EACH side
/// lands in a quiet gap, and a shorter round is likelier to fit inside one. At
/// three ~18ms rounds the gate host — three cold cross-builds and four booting
/// QEMUs — drifted ratios by up to 0.51x, at every size, which is most of the
/// distance from a passing case to the 2.0x bar.
const ROUNDS: usize = 9;

/// Total iterations*bytes per side, held at what the original three-round shape
/// did, so this change trades round LENGTH for round COUNT and costs no wall time.
const WORK: usize = 3 << 26;

fn bench_pair(n: usize, mut rust: impl FnMut(), mut c: impl FnMut()) -> (f64, f64) {
    let iters = core::cmp::max(1, WORK / (n * ROUNDS));
    // warm up BOTH (page-in, branch predictors, i-cache) before either is timed
    for _ in 0..core::cmp::min(iters, 64) {
        rust();
        c();
    }
    let (mut best_r, mut best_c) = (0.0f64, 0.0f64);
    for _ in 0..ROUNDS {
        best_r = best_r.max(round(n, iters, &mut rust));
        best_c = best_c.max(round(n, iters, &mut c));
    }
    (best_r, best_c)
}

/// How far a ratio sits from 1.00x, in either direction, as a multiple >= 1.0.
fn deviation(rust: f64, c: f64) -> f64 {
    if rust <= 0.0 || c <= 0.0 {
        return f64::INFINITY;
    }
    let r = c / rust;
    if r < 1.0 {
        1.0 / r
    } else {
        r
    }
}

/// What the identical twins read at one size, worst-of-both-languages per shape.
/// Every field MUST be 1.00x — each is the same source through the same compiler,
/// differing only in where it landed. What they actually read is this harness's
/// error bar at that size, in this binary, under this load.
struct Cal {
    copy: f64,
    scan: f64,
    fill: f64,
}

impl Cal {
    /// The instrument's resolution here: the worst any twin strayed from 1.00x.
    fn resolution(&self) -> f64 {
        self.copy.max(self.scan).max(self.fill)
    }
}

/// Bench the identical-twin pairs at `n`, in both languages, and keep the worst
/// per shape — error contributed by EITHER side lands on the ratio the suite
/// reports, so the bar has to cover both. See perfctl.c for the argument.
fn calibrate(n: usize) -> Cal {
    // A pair whose two halves got merged into one body would read a vacuous,
    // permanent 1.00x. That is the one failure mode this control cannot detect
    // by its own output, so check it directly.
    let merged: Vec<&str> = [
        ("copy/rs", perf_ctl_copy_rs_a as usize, perf_ctl_copy_rs_b as usize),
        ("scan/rs", perf_ctl_scan_rs_a as usize, perf_ctl_scan_rs_b as usize),
        ("fill/rs", perf_ctl_fill_rs_a as usize, perf_ctl_fill_rs_b as usize),
        ("copy/c", perf_ctl_copy_c_a as usize, perf_ctl_copy_c_b as usize),
        ("scan/c", perf_ctl_scan_c_a as usize, perf_ctl_scan_c_b as usize),
        ("fill/c", perf_ctl_fill_c_a as usize, perf_ctl_fill_c_b as usize),
    ]
    .iter()
    .filter(|(_, a, b)| a == b)
    .map(|(name, _, _)| *name)
    .collect();
    assert!(
        merged.is_empty(),
        "perf control is VACUOUS: {merged:?} twin(s) share one address, so the \
         calibration below measures nothing and would read a perfect 1.00x \
         forever. Some ICF pass (gcc -fipa-icf, LLVM mergefunc, a linker --icf) \
         folded the identical bodies together — separate them again."
    );

    let src: Vec<u8> = std::iter::repeat(b'x').take(n).chain([0]).collect();
    let mut dst = vec![0u8; n + 1];
    let sp = src.as_ptr();
    let dp = dst.as_mut_ptr();
    unsafe {
        let (a, b) = bench_pair(n, || {
            perf_ctl_copy_rs_a(dp, sp);
        }, || {
            perf_ctl_copy_rs_b(dp, sp);
        });
        let (ca, cb) = bench_pair(n, || {
            perf_ctl_copy_c_a(dp, sp);
        }, || {
            perf_ctl_copy_c_b(dp, sp);
        });
        let copy = deviation(a, b).max(deviation(ca, cb));

        let (a, b) = bench_pair(n, || {
            core::hint::black_box(perf_ctl_scan_rs_a(sp));
        }, || {
            core::hint::black_box(perf_ctl_scan_rs_b(sp));
        });
        let (ca, cb) = bench_pair(n, || {
            core::hint::black_box(perf_ctl_scan_c_a(sp));
        }, || {
            core::hint::black_box(perf_ctl_scan_c_b(sp));
        });
        let scan = deviation(a, b).max(deviation(ca, cb));

        let (a, b) = bench_pair(n, || {
            perf_ctl_fill_rs_a(dp as *mut c_void, 0x41, n);
        }, || {
            perf_ctl_fill_rs_b(dp as *mut c_void, 0x41, n);
        });
        let (ca, cb) = bench_pair(n, || {
            perf_ctl_fill_c_a(dp as *mut c_void, 0x41, n);
        }, || {
            perf_ctl_fill_c_b(dp as *mut c_void, 0x41, n);
        });
        let fill = deviation(a, b).max(deviation(ca, cb));

        Cal { copy, scan, fill }
    }
}

/// Past this the twins disagree so wildly that no slowdown verdict is possible
/// at all, and saying "✓" would be a lie. Chosen well above anything a loaded
/// gate host produces (observed worst ~1.5x) and well below the class of
/// regression this gate exists to catch (wave 2's `memset` was orders of
/// magnitude, at every size). If this trips, the answer is a quieter host — not
/// a bigger number here.
const UNMEASURABLE: f64 = 5.0;

struct Row {
    name: String,
    rust: f64,
    c: f64,
    n: usize,
}

/// How many independent measurement passes a case must fail before the gate
/// calls it a regression.
///
/// The calibrated bar handles error that is STABLE across a run — code layout,
/// steady background load. It cannot handle a burst, because the calibration is
/// one snapshot and a burst is not: on a host at load 21 the bar widened
/// correctly to 2.81x at n=16 and `memset/16` still read 3.41x, its Rust rounds
/// having been hit 3.9x while the C rounds took only 1.6x. Nothing measured
/// once can tell that apart from a real regression.
///
/// Reproduction can. A slow body is slow in every pass; a burst has to land on
/// the same case, on the same side, three times running. Costs nothing on a
/// green run — extra passes happen only after something has already failed.
const PASSES: usize = 3;

/// `lc_handle[LC_CTYPE]` for (subject, reference) -- which locale MODE each side's
/// case-mapping family is running in. Zero is the "C" locale: `towlower`/`towupper`
/// take an ASCII add and are a few instructions. NONZERO is a real NLS locale, and
/// that side then routes through **one `LCMapStringW` call per character**.
///
/// The two sides hold SEPARATE locale state, so they can disagree -- and when they
/// do, the benchmark is comparing an ASCII add against a Win32 call and the ratio
/// means nothing. A gate run on 2026-09-13 reported `towlower` 10.8x TOO SLOW in all
/// three passes at a flat 44 MiB/s across a 65536x size range: the signature of a
/// fixed per-character Win32 call, not of a slow body. Disassembly of the failing
/// binary confirmed the subject sitting on the `LCMapStringW` arm while the C
/// reference jumped straight to its ASCII path. Four theories were chased and killed
/// (host contention, a recent commit, suite context, an in-process global leak)
/// before anyone looked at the branch -- because the harness reported a STATE
/// difference as a BEHAVIOUR difference, and nothing printed the state.
///
/// Both sides NLS is fine: they pay the same call and the ratio is still honest.
/// It is DISAGREEMENT that invalidates the comparison. Note what this deliberately
/// does not do: pin either side to the "C" locale. That would make the run green
/// while the state leak sat there, and the leak is the only bug in this still alive.
fn locale_modes() -> (u32, u32) {
    // SAFETY: both accessors are the harness's own and never return null -- each
    // falls back to its side's static "C" locinfo when no thread locale is set.
    unsafe {
        (
            (*get_locinfo()).lc_handle[msvcrt_sys::LC_CTYPE] as u32,
            (*msvcrt_ref_get_locinfo()).lc_handle[msvcrt_sys::LC_CTYPE] as u32,
        )
    }
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

    // Precondition, checked BEFORE any measuring: the two sides must be in the SAME
    // locale mode. See `locale_modes()` for what this cost us when nothing checked it.
    let (subj_lc, ref_lc) = locale_modes();
    let mode = |v: u32| if v == 0 { "C locale, ASCII add" } else { "NLS, LCMapStringW per char" };
    println!("\n  locale: lc_handle[LC_CTYPE]  subject={subj_lc:#x} ({}), reference={ref_lc:#x} ({})",
             mode(subj_lc), mode(ref_lc));
    if (subj_lc == 0) != (ref_lc == 0) {
        println!(
            "\n  perf: ABORTED -- ORACLE ASYMMETRY, not a slow body.\n  \
             subject lc_handle[LC_CTYPE]={subj_lc:#x}, reference={ref_lc:#x}: one side runs\n  \
             one LCMapStringW call PER CHARACTER in the case-mapping family while the\n  \
             other takes an ASCII add. The two sides are not running comparable code,\n  \
             so every ratio here is meaningless and calling one TOO SLOW would be a\n  \
             FALSE statement about a TRUE measurement -- which is what cost a lane its\n  \
             batch on 2026-09-13.\n\n  \
             This is a harness-ENVIRONMENT fault: nothing in perf.rs sets a locale.\n  \
             Find what did before trusting any number in this file.\n"
        );
        std::process::exit(1);
    }

    let mut survivors: Option<Vec<String>> = None;
    for pass in 1..=PASSES {
        if pass > 1 {
            println!(
                "\n  ── pass {pass}/{PASSES}: re-measuring, because a pass must fail {PASSES}\n  \
                 times running before this gate blames the code rather than the host ──"
            );
        }
        let (rows, cals) = measure();
        let bad = report(&rows, &cals, pass);
        let still: Vec<String> = match survivors {
            None => bad,
            Some(prev) => bad.into_iter().filter(|b| prev.contains(b)).collect(),
        };
        if still.is_empty() {
            if pass > 1 {
                println!(
                    "\n  cleared on pass {pass} — the earlier failure did not reproduce, so it\n  \
                     was host noise, not a regression. Passing.\n"
                );
            }
            return;
        }
        survivors = Some(still);
    }

    let bad = survivors.unwrap_or_default();
    println!("\n  SLOWER THAN THE C IT REPLACES, in all {PASSES} passes: {bad:?}");
    println!("  These are system-wide CRT primitives — a semantically perfect but");
    println!("  slow body is still a regression (wave 2: net suite 71s -> 208s,");
    println!("  nettest.exe blew its 120s timeout). Fix the body or drop the flip.");
    println!("  Each cleared the error bar measured on its own run, {PASSES} times over,");
    println!("  so this is not host noise. Reproduce with:");
    println!("    cargo test -p msvcrt-difftest --test perf --release\n");
    std::process::exit(1);
}

/// One full measurement pass: every case, then the calibration.
fn measure() -> (Vec<Row>, Vec<(usize, Cal)>) {
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
            rows.push(Row { name: format!("memset/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                memcpy(dp, sp, n);
            }, || {
                ref_memcpy(d2p, sp, n);
            });
            rows.push(Row { name: format!("memcpy/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(memcmp(sp, sp, n));
            }, || {
                core::hint::black_box(ref_memcmp(sp, sp, n));
            });
            rows.push(Row { name: format!("memcmp/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(memchr(sp, 0xFF, n));
            }, || {
                core::hint::black_box(ref_memchr(sp, 0xFF, n));
            });
            rows.push(Row { name: format!("memchr/{n}"), rust: r, c, n });

            // strlen wants a NUL-terminated buffer of length n
            let mut s = vec![b'x'; n + 1];
            s[n] = 0;
            let strp = s.as_ptr();
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strlen(strp));
            }, || {
                core::hint::black_box(ref_strlen(strp));
            });
            rows.push(Row { name: format!("strlen/{n}"), rust: r, c, n });

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
            rows.push(Row { name: format!("wcslen/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                wcscpy(wdp, wsp);
            }, || {
                ref_wcscpy(wdp, wsp);
            });
            rows.push(Row { name: format!("wcscpy/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(wcscmp(wsp, wsp));
            }, || {
                core::hint::black_box(ref_wcscmp(wsp, wsp));
            });
            rows.push(Row { name: format!("wcscmp/{n}"), rust: r, c, n });

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
            rows.push(Row { name: format!("_stricmp/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_strnicmp(strp, strp, n));
            }, || {
                core::hint::black_box(ref__strnicmp(strp, strp, n));
            });
            rows.push(Row { name: format!("_strnicmp/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_wcsicmp(wsp, wsp));
            }, || {
                core::hint::black_box(ref__wcsicmp(wsp, wsp));
            });
            rows.push(Row { name: format!("_wcsicmp/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(_wcsnicmp(wsp, wsp, n));
            }, || {
                core::hint::black_box(ref__wcsnicmp(wsp, wsp, n));
            });
            rows.push(Row { name: format!("_wcsnicmp/{n}"), rust: r, c, n });

            // The scalar mappers, hammered n times — these are what the loops above
            // call per character, so a regression here multiplies through everything.
            let (r, c) = bench_pair(n, || {
                for c in 0..n { core::hint::black_box(towlower((c & 0xFFFF) as u16)); }
            }, || {
                for c in 0..n { core::hint::black_box(ref_towlower((c & 0xFFFF) as u16)); }
            });
            rows.push(Row { name: format!("towlower/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                for c in 0..n { core::hint::black_box(tolower((c & 0xFF) as i32)); }
            }, || {
                for c in 0..n { core::hint::black_box(ref_tolower((c & 0xFF) as i32)); }
            });
            rows.push(Row { name: format!("tolower/{n}"), rust: r, c, n });

            // ── the compare/copy family (equal inputs: forces the full walk) ──
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strcmp(strp, strp));
            }, || {
                core::hint::black_box(ref_strcmp(strp, strp));
            });
            rows.push(Row { name: format!("strcmp/{n}"), rust: r, c, n });

            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strncmp(strp, strp, n));
            }, || {
                core::hint::black_box(ref_strncmp(strp, strp, n));
            });
            rows.push(Row { name: format!("strncmp/{n}"), rust: r, c, n });

            let mut sd = vec![0u8; n + 1];
            let sdp = sd.as_mut_ptr();
            let (r, c) = bench_pair(n, || {
                strcpy(sdp, strp);
            }, || {
                ref_strcpy(sdp, strp);
            });
            rows.push(Row { name: format!("strcpy/{n}"), rust: r, c, n });

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
            rows.push(Row { name: format!("strcat/{n}"), rust: r, c, n });

            // absent byte -> forces the full scan
            let (r, c) = bench_pair(n, || {
                core::hint::black_box(strchr(strp, b'Z' as i32));
            }, || {
                core::hint::black_box(ref_strchr(strp, b'Z' as i32));
            });
            rows.push(Row { name: format!("strchr/{n}"), rust: r, c, n });
        }
    }

    // Calibrate AFTER the cases, on the same host in the same state, so the
    // error bar describes the run it is applied to.
    let cals: Vec<(usize, Cal)> = SIZES.iter().map(|&n| (n, calibrate(n))).collect();
    (rows, cals)
}

/// Print one pass and return the cases that beat their measured bar.
fn report(rows: &[Row], cals: &[(usize, Cal)], pass: usize) -> Vec<String> {
    println!("\n=== instrument calibration — identical twins, each MUST read 1.00x ===\n");
    println!(
        "  {:<10} {:>8} {:>8} {:>8}   {:>10} {:>8}",
        "size", "copy", "scan", "fill", "resolution", "bar"
    );
    for (n, cal) in cals {
        println!(
            "  {:<10} {:>7.2}x {:>7.2}x {:>7.2}x   {:>9.2}x {:>7.2}x",
            n,
            cal.copy,
            cal.scan,
            cal.fill,
            cal.resolution(),
            MAX_SLOWDOWN * cal.resolution()
        );
    }
    println!(
        "\n  Same algorithm on both sides, so every reading above is pure harness\n  \
         error (code layout, host load). A case may only be called TOO SLOW when it\n  \
         beats {MAX_SLOWDOWN}x by more than the error measured at its own size — see perfctl.c.\n"
    );

    let unmeasurable: Vec<String> = cals
        .iter()
        .filter(|(_, c)| c.resolution() > UNMEASURABLE)
        .map(|(n, c)| format!("n={} ({:.1}x)", n, c.resolution()))
        .collect();

    println!("=== msvcrt Rust vs GENUINE C — throughput (MiB/s, higher is better) ===\n");
    println!(
        "  {:<16} {:>12} {:>12} {:>10} {:>8}",
        "case", "rust", "C", "slowdown", "bar"
    );
    // Keyed by case NAME, not by the formatted line: the ratio moves between
    // passes, and main() intersects these sets to decide what reproduced.
    let mut bad: Vec<String> = Vec::new();
    let mut marginal: Vec<String> = Vec::new();
    for r in rows {
        let slow = if r.rust > 0.0 { r.c / r.rust } else { f64::INFINITY };
        let res = cals
            .iter()
            .find(|(n, _)| *n == r.n)
            .map(|(_, c)| c.resolution())
            .unwrap_or(1.0);
        let bar = MAX_SLOWDOWN * res;
        let tag = if slow > bar {
            bad.push(r.name.clone());
            "  ✗ TOO SLOW"
        } else if slow > MAX_SLOWDOWN {
            // Over the threshold but inside the error bar: NOT a verdict, but not
            // nothing either. Printed so a body that is genuinely drifting shows up
            // as a trend instead of hiding until it randomly clears the bar.
            marginal.push(format!("{} ({:.2}x, bar {:.2}x)", r.name, slow, bar));
            "  ~ within error"
        } else {
            "  ✓"
        };
        println!(
            "  {:<16} {:>12.0} {:>12.0} {:>9.2}x {:>7.2}x{}",
            r.name, r.rust, r.c, slow, bar, tag
        );
    }

    if !marginal.is_empty() {
        println!("\n  over {MAX_SLOWDOWN}x but inside this run's error bar: {marginal:?}");
        println!("  Not a failure. Worth a look if the same case keeps appearing here.");
    }

    if !unmeasurable.is_empty() {
        // Not subject to the reproduce-it rule: this says the instrument itself
        // is inoperable, so re-running it would only produce more unusable
        // numbers. Well clear of anything a loaded gate host produces (worst
        // observed 1.41x at load 21), so in practice it never fires.
        println!("\n  PERF UNMEASURABLE — the identical twins disagree by more than");
        println!("  {UNMEASURABLE}x at {unmeasurable:?}, so no slowdown verdict is possible");
        println!("  at those sizes and a ✓ above would be worth nothing. This host is");
        println!("  too loaded to gate on; it is not a finding about the code.\n");
        std::process::exit(1);
    }

    if bad.is_empty() {
        println!(
            "\n  pass {pass}: every flipped primitive is within its measured bar of the C \
             it replaces ✓\n"
        );
    } else {
        println!("\n  pass {pass}: over the measured bar: {bad:?}");
    }
    bad
}
