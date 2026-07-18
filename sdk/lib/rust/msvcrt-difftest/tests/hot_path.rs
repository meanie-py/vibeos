//! `hot_path` — the SIXTH instrument: *does a per-element loop do per-element
//! ENVIRONMENT LOOKUPS?*
//!
//! # Why this exists
//!
//! Wave 11 gave `_wcsicmp` its locale branch and, in doing so, made it re-enter
//! `get_locinfo()` **once per character**. Every other instrument passed:
//!
//! | instrument | verdict | why it was blind |
//! |---|---|---|
//! | `classify` | MATCH | the ANSWERS were right. It grades values, not cost. |
//! | `oob_guard` | in bounds | it grades access, not cost. |
//! | `perf` | **0.91x, PASSING** | see below — and it can *never* catch this. |
//! | `ref_integrity` | oracle honest | different question entirely. |
//!
//! and the OS booted in **128s instead of 39s**, timing out the kernel TDI self-test and
//! turning the gate red. The smoke tests — the North Star — caught what the whole
//! differential harness could not.
//!
//! # Why `perf` cannot catch it, ever
//!
//! In the real msvcrt, `get_locinfo()` costs **three Win32 calls**:
//!
//! ```c
//! thread_data_t *msvcrt_get_thread_data(void) {
//!     DWORD err = GetLastError();                       // call
//!     if (!(ptr = TlsGetValue( msvcrt_tls_index )))     // call
//!     ...                                               // SetLastError
//! ```
//!
//! In the *harness* it is a static-address return. Both the reference and the subject pay
//! that same trivial cost, so the RATIO stays healthy while the real DLL burns three
//! syscalls a character. **A stub in the harness hid a real cost** — the same shape as
//! §9.73 (`refimpl`'s `_tolower_l` stub hid a real *behaviour*), and it will keep
//! happening: every stub is a claim about the environment, and a cheap stub is a claim
//! that the environment is cheap.
//!
//! Tuning the stub to be "realistically slow" would just be tuning a fake. So don't
//! measure — **count**.
//!
//! # The invariant
//!
//! > A function that walks `n` elements may consult the environment **O(1)** times, not
//! > `O(n)`.
//!
//! This is exactly what msvcrt's own structure encodes: `_wcsicmp_l` resolves the locale
//! **once** (`get_current_locale_noalloc(&tmp)`) and passes it down to `_towlower_l`. Our
//! Rust must do the same — hoist the `get_locinfo()` out of the loop and fold against an
//! already-resolved `locinfo`.
//!
//! A count is deterministic, immune to machine noise, and cannot be argued with.

// Forces the crate under test into the link — its `#[no_mangle]` bodies are what we
// are counting against. (Same line every other harness in this crate carries.)
extern crate msvcrt_crt_rust;

use std::ffi::c_void;

extern "C" {
    fn subj_locinfo_calls_get() -> i32;
    fn subj_locinfo_calls_reset();

    fn _stricmp(a: *const u8, b: *const u8) -> i32;
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _strlwr_s(s: *mut u8, n: usize) -> i32;
    fn _strupr_s(s: *mut u8, n: usize) -> i32;
    fn _wcslwr_s(s: *mut u16, n: usize) -> i32;
    fn _wcsupr_s(s: *mut u16, n: usize) -> i32;
    fn strtol(s: *const u8, e: *mut *mut u8, base: i32) -> i32;
}

/// How many `get_locinfo()` calls we tolerate for a single call of a walking function.
///
/// Not zero: the function legitimately needs the locale ONCE (and `_strlwr_s`-class
/// functions may resolve it once in a helper too). It is the SCALING that is the bug —
/// so we run each function over two very different lengths and demand the count does not
/// grow with `n`.
const TOLERANCE: i32 = 4;

struct Case {
    name: &'static str,
    /// Runs the function once over a string of `n` elements.
    run: Box<dyn Fn(usize)>,
}

fn count(run: &dyn Fn(usize), n: usize) -> i32 {
    unsafe {
        subj_locinfo_calls_reset();
        run(n);
        subj_locinfo_calls_get()
    }
}

fn main() {
    // Two lengths, an order of magnitude apart. An O(1) function reports the same count
    // for both; an O(n) one reports ~n.
    const SHORT: usize = 8;
    const LONG: usize = 512;

    let cases: Vec<Case> = vec![
        Case {
            name: "_stricmp",
            run: Box::new(|n| unsafe {
                let s = vec![b'a'; n + 1];
                let mut s = s;
                s[n] = 0;
                std::hint::black_box(_stricmp(s.as_ptr(), s.as_ptr()));
            }),
        },
        Case {
            name: "_strnicmp",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'a'; n + 1];
                s[n] = 0;
                std::hint::black_box(_strnicmp(s.as_ptr(), s.as_ptr(), n));
            }),
        },
        Case {
            name: "_memicmp",
            run: Box::new(|n| unsafe {
                let s = vec![b'a'; n];
                std::hint::black_box(_memicmp(
                    s.as_ptr().cast(),
                    s.as_ptr().cast(),
                    n,
                ));
            }),
        },
        Case {
            name: "_wcsicmp",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'a' as u16; n + 1];
                s[n] = 0;
                std::hint::black_box(_wcsicmp(s.as_ptr(), s.as_ptr()));
            }),
        },
        Case {
            name: "_wcsnicmp",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'a' as u16; n + 1];
                s[n] = 0;
                std::hint::black_box(_wcsnicmp(s.as_ptr(), s.as_ptr(), n));
            }),
        },
        Case {
            name: "_strlwr_s",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'A'; n + 1];
                s[n] = 0;
                std::hint::black_box(_strlwr_s(s.as_mut_ptr(), n + 1));
            }),
        },
        Case {
            name: "_strupr_s",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'a'; n + 1];
                s[n] = 0;
                std::hint::black_box(_strupr_s(s.as_mut_ptr(), n + 1));
            }),
        },
        Case {
            name: "_wcslwr_s",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'A' as u16; n + 1];
                s[n] = 0;
                std::hint::black_box(_wcslwr_s(s.as_mut_ptr(), n + 1));
            }),
        },
        Case {
            name: "_wcsupr_s",
            run: Box::new(|n| unsafe {
                let mut s = vec![b'a' as u16; n + 1];
                s[n] = 0;
                std::hint::black_box(_wcsupr_s(s.as_mut_ptr(), n + 1));
            }),
        },
        Case {
            name: "strtol",
            run: Box::new(|n| unsafe {
                // The int parsers fold every digit through `_tolower_l`, which resolves
                // the locale. Same invariant.
                let mut s = vec![b'7'; n + 1];
                s[n] = 0;
                std::hint::black_box(strtol(s.as_ptr(), std::ptr::null_mut(), 10));
            }),
        },
    ];

    println!(
        "\n=== hot_path: does a per-element loop do per-element ENVIRONMENT lookups? ===\n"
    );
    println!("  {:<12} {:>8} {:>8}   verdict", "fn", "n=8", "n=512");

    let mut bad: Vec<String> = Vec::new();
    for c in &cases {
        let short = count(&*c.run, SHORT);
        let long = count(&*c.run, LONG);

        // The bug is SCALING, not the absolute number. A function that resolves the
        // locale once reports the same count at both lengths.
        let scales = long > short + TOLERANCE;
        let mark = if scales { "✗ SCALES WITH n" } else { "✓ O(1)" };
        println!("  {:<12} {short:>8} {long:>8}   {mark}", c.name);
        if scales {
            bad.push(format!(
                "{}: get_locinfo() called {short}x at n=8 but {long}x at n=512 — \
                 the lookup is INSIDE the per-element loop",
                c.name
            ));
        }
    }

    if !bad.is_empty() {
        println!("\n  ENVIRONMENT LOOKUP IN A PER-ELEMENT LOOP:\n");
        for b in &bad {
            println!("    - {b}");
        }
        println!(
            "\n  In the real msvcrt, get_locinfo() is three Win32 calls (GetLastError +\n  \
             TlsGetValue + SetLastError). Once per call is free; once per CHARACTER took\n  \
             the boot from 39s to 128s and timed out the kernel TDI self-test.\n\n  \
             Fix: resolve the locinfo ONCE at the top of the function and fold against it\n  \
             (see `towlower_li` / `tolower_li` in msvcrt-crt-rust). That is what the C\n  \
             does — `_wcsicmp_l` calls get_current_locale_noalloc(&tmp) exactly once.\n\n  \
             NOTE: `perf` cannot catch this and never will — the harness stubs\n  \
             get_locinfo() as a cheap static return, so BOTH sides pay the same trivial\n  \
             cost and the ratio stays healthy. Do not 'fix' this by tuning the stub.\n"
        );
        std::process::exit(1);
    }

    println!("\n  every walking function resolves the locale O(1) times. ✓\n");
}
