#![no_builtins]
//! Guard-page probe for the **bounded** ("n"-suffixed) msvcrt CRT functions.
//!
//! WHY THIS EXISTS (it caught a real shipped bug — see below).
//!
//! `classify.rs` fuzzes the Rust against the genuine C and compares *values*.
//! That structurally cannot see an **out-of-bounds READ**: a function that
//! scans too far but then clamps its answer produces byte-for-byte identical
//! output, so 20,000 fuzz cases all report MATCH. The over-read only bites in
//! the guest, when the scan happens to walk into an unmapped page and faults.
//!
//! That is exactly how `_strnset` shipped broken for one gate cycle. Its Rust
//! did `min(cstr_slice(p).len(), len)` — an **unbounded** NUL scan, clamped
//! afterwards. The fill stayed bounded, so every value matched; but the *scan*
//! ran off the end of a buffer that an `n`-suffixed API is explicitly allowed
//! to leave unterminated. Reached via `mbcs.c`'s `u__strnset()` →
//! `_mbsnset`/`_mbsnbset`, it faulted in msvcrt.dll and killed the process.
//! The C never does this: `while (*str && len--)` reads at most `len` bytes.
//!
//! THE PROBE: map two pages, `mprotect(PROT_NONE)` the second, and place a
//! deliberately **NUL-FREE** buffer of `k` bytes ending flush against the
//! boundary. Call each function with `n <= k`. A correct implementation never
//! reads past `p[k-1]`; one that scans unbounded touches the guard page and
//! takes SIGSEGV. Each call runs in a forked child, so a fault is *reported*
//! (naming the function) instead of taking the harness down.
//!
//! Add every new bounded flip here. The rule this encodes:
//! **an `n`-suffixed CRT function must never contain an unbounded scan** —
//! bound it by `n` (`cstr_slice_bounded`), never scan-then-`min`.

extern crate msvcrt_crt_rust;

use core::ffi::c_void;

extern "C" {
    fn strnlen(s: *const u8, m: usize) -> usize;
    fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn strncpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn strncat(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memchr(b: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn _memccpy(d: *mut c_void, s: *const c_void, c: i32, n: usize) -> *mut c_void;
    fn _strnset(s: *mut u8, c: i32, n: usize) -> *mut u8;

    // wave 4 — the WIDE n-suffixed family. `_wcsnset` has the exact shape that
    // shipped the out-of-bounds read in the narrow `_strnset` (§9.61), so it is
    // guarded from day one rather than after the fact.
    fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _wcsnset(s: *mut u16, c: u16, n: usize) -> *mut u16;

    // wave 5a — bounded, so it belongs here from day one (§9.61).
    fn __strncnt(s: *const u8, size: usize) -> usize;

    // wave 5b — the bounded members of the `_s` family. Their whole PURPOSE is
    // not to run off a buffer, so an over-read here would be especially absurd.
    fn strncpy_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn strncat_s(d: *mut u8, elem: usize, s: *const u8, n: usize) -> i32;
    fn _strnset_s(s: *mut u8, size: usize, c: i32, n: usize) -> i32;

    // wave 6b — the radix `_s` family. These are WRITERS: the risk is not an
    // over-READ but an over-WRITE past `size`. Their too-small-buffer arm is the
    // dangerous one — it deliberately fills the caller's buffer with reversed
    // digits, and the loop bound is `size` (minus the sign slot). An off-by-one
    // there scribbles one byte past the caller's allocation. See below.
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

    // wave 6c — the wide `_s` writers + the bounded wide readers.
    fn wcsncpy_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn wcsncat_s(d: *mut u16, e: usize, s: *const u16, n: usize) -> i32;
    fn _wcsnset_s(s: *mut u16, sz: usize, c: u16, n: usize) -> i32;
    fn _wcsset_s(s: *mut u16, n: usize, c: u16) -> i32;
    fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    fn wcsnlen(s: *const u16, m: usize) -> usize;

    // wave 7 — bounded case-folders/comparers.
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    fn _memicmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn _strlwr_s(s: *mut u8, len: usize) -> i32;
    fn _strupr_s(s: *mut u8, len: usize) -> i32;
}

const PAGE: usize = 4096;

/// `k` NUL-free bytes ending EXACTLY at a PROT_NONE guard page.
/// Reading `p[k]` faults; reading `p[k-1]` is fine.
unsafe fn guarded(k: usize) -> *mut u8 {
    assert!(k <= PAGE);
    let base = libc::mmap(
        core::ptr::null_mut(),
        2 * PAGE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
    ) as *mut u8;
    assert!(base as isize != -1, "mmap failed");
    assert_eq!(
        libc::mprotect(base.add(PAGE) as *mut c_void, PAGE, libc::PROT_NONE),
        0,
        "mprotect failed"
    );
    let p = base.add(PAGE - k);
    // NUL-free on purpose: an unbounded scan has nothing to stop it.
    for i in 0..k {
        *p.add(i) = b'A' + (i % 26) as u8;
    }
    p
}

/// `guarded()`'s wide twin: `k` NUL-free u16s ending EXACTLY at the guard page.
/// Reading `p[k]` faults. Every element has BOTH bytes non-zero, so a scan that
/// is looking for a u16 NUL has nothing to stop it before the guard.
unsafe fn wguarded(k: usize) -> *mut u16 {
    assert!(k * 2 <= PAGE);
    let base = libc::mmap(
        core::ptr::null_mut(),
        2 * PAGE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
    ) as *mut u8;
    assert!(base as isize != -1, "mmap failed");
    assert_eq!(
        libc::mprotect(base.add(PAGE) as *mut c_void, PAGE, libc::PROT_NONE),
        0,
        "mprotect failed"
    );
    let p = base.add(PAGE - k * 2) as *mut u16;
    for i in 0..k {
        // 0x4141.. — never zero in EITHER byte.
        *p.add(i) = 0x4141u16 + (i % 26) as u16;
    }
    p
}

/// Run `f` in a forked child; `false` if it died on a signal (the over-read).
fn probes_clean(f: impl FnOnce()) -> bool {
    unsafe {
        let pid = libc::fork();
        if pid == 0 {
            f();
            libc::_exit(0);
        }
        let mut st: i32 = 0;
        libc::waitpid(pid, &mut st, 0);
        libc::WIFEXITED(st) && libc::WEXITSTATUS(st) == 0
    }
}

fn main() {
    let mut failed: Vec<&str> = Vec::new();
    let k = 32usize; // buffer length; every call below passes n <= k

    // (name, probe). Each MUST NOT read past p[k-1].
    let cases: Vec<(&str, Box<dyn Fn()>)> = vec![
        ("strnlen", Box::new(move || unsafe { core::hint::black_box(strnlen(guarded(k), k)); })),
        ("strncmp", Box::new(move || unsafe {
            let (a, b) = (guarded(k), guarded(k));
            core::hint::black_box(strncmp(a, b, k));
        })),
        ("strncpy", Box::new(move || unsafe {
            let mut d = [0u8; 64];
            core::hint::black_box(strncpy(d.as_mut_ptr(), guarded(k), k));
        })),
        ("strncat", Box::new(move || unsafe {
            let mut d = [0u8; 128];
            d[0] = 0;
            core::hint::black_box(strncat(d.as_mut_ptr(), guarded(k), k));
        })),
        ("memcmp", Box::new(move || unsafe {
            let (a, b) = (guarded(k), guarded(k));
            core::hint::black_box(memcmp(a.cast(), b.cast(), k));
        })),
        ("memchr", Box::new(move || unsafe {
            // byte that is NOT present -> forces the full n-byte scan
            core::hint::black_box(memchr(guarded(k).cast(), 0xFF, k));
        })),
        ("_memccpy", Box::new(move || unsafe {
            let mut d = [0u8; 64];
            core::hint::black_box(_memccpy(d.as_mut_ptr().cast(), guarded(k).cast(), 0xFF, k));
        })),
        ("_strnset", Box::new(move || unsafe {
            core::hint::black_box(_strnset(guarded(k), b'z' as i32, k));
        })),

        // ── wave 4: the wide n-suffixed family ──
        ("wcsncmp", Box::new(move || unsafe {
            let (a, b) = (wguarded(k), wguarded(k));
            core::hint::black_box(wcsncmp(a, b, k));
        })),
        ("_wcsnset", Box::new(move || unsafe {
            core::hint::black_box(_wcsnset(wguarded(k), 0x7A7A, k));
        })),
        ("__strncnt", Box::new(move || unsafe {
            core::hint::black_box(__strncnt(guarded(k), k));
        })),

        // ── wave 5b: the bounded `_s` family ──
        // src is the guarded (NUL-free) buffer; the copy is bounded by count<=k.
        ("strncpy_s", Box::new(move || unsafe {
            let mut d = [0u8; 128];
            core::hint::black_box(strncpy_s(d.as_mut_ptr(), d.len(), guarded(k), k));
        })),
        ("strncat_s", Box::new(move || unsafe {
            let mut d = [0u8; 128];
            d[0] = 0;
            core::hint::black_box(strncat_s(d.as_mut_ptr(), d.len(), guarded(k), k));
        })),
        // _strnset_s SCANS its own buffer up to `size` — so the guarded buffer is
        // the subject, and size must not walk past it.
        ("_strnset_s", Box::new(move || unsafe {
            core::hint::black_box(_strnset_s(guarded(k), k, b'z' as i32, k));
        })),

        // ── wave 6b: the radix `_s` family — WRITE bounds ─────────────────────
        //
        // Every call below is deliberately in the TOO-SMALL arm: base 2 of a
        // 64-bit value needs 65 bytes, and we hand it a `size` of 8 flush against
        // a guard page. That is the arm that fills the caller's buffer with
        // reversed digits, so its loop bound is the one that must not be off by
        // one. `sz` is the FULL writable extent — a single byte past it faults.
        //
        // (i32::MIN at radix 10 also puts the sign-slot skip under test: the C
        // does `p++; size--;` and then writes `size` more bytes.)
        ("_itoa_s",    Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_itoa_s(i32::MIN, guarded(sz), sz, 2)); })),
        ("_ltoa_s",    Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ltoa_s(i32::MIN, guarded(sz), sz, 10)); })),
        ("_i64toa_s",  Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_i64toa_s(i64::MIN, guarded(sz), sz, 2)); })),
        ("_ui64toa_s", Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ui64toa_s(u64::MAX, guarded(sz), sz, 2)); })),
        ("_ultoa_s",   Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ultoa_s(u32::MAX, guarded(sz), sz, 2)); })),
        ("_itow_s",    Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_itow_s(i32::MIN, wguarded(sz), sz, 2)); })),
        ("_ltow_s",    Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ltow_s(i32::MIN, wguarded(sz), sz, 10)); })),
        ("_i64tow_s",  Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_i64tow_s(i64::MIN, wguarded(sz), sz, 2)); })),
        ("_ui64tow_s", Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ui64tow_s(u64::MAX, wguarded(sz), sz, 2)); })),
        ("_ultow_s",   Box::new(move || unsafe { let sz = 8;  core::hint::black_box(_ultow_s(u32::MAX, wguarded(sz), sz, 2)); })),

        // ── wave 6c: the wide `_s` family ─────────────────────────────────────
        // Destination is the guarded buffer with `elem` == its FULL extent, and a
        // source longer than it — i.e. the ERANGE/truncate arms, which are the
        // ones that walk the destination. wcsncpy_s's _TRUNCATE arm writes
        // `*(p-1)`, so it is exercised explicitly.
        ("wcsncpy_s", Box::new(move || unsafe {
            let src = [0x41u16; 40];
            core::hint::black_box(wcsncpy_s(wguarded(k), k, src.as_ptr(), usize::MAX));
        })),
        ("wcsncat_s", Box::new(move || unsafe {
            let src = [0x41u16; 40];
            let d = wguarded(k); *d.add(1) = 0;   // dst terminated inside elem
            core::hint::black_box(wcsncat_s(d, k, src.as_ptr(), usize::MAX));
        })),
        ("_wcsnset_s", Box::new(move || unsafe {
            core::hint::black_box(_wcsnset_s(wguarded(k), k, 0x7a, k));
        })),
        ("_wcsset_s", Box::new(move || unsafe {
            core::hint::black_box(_wcsset_s(wguarded(k), k, 0x7a));
        })),
        ("wcsncpy", Box::new(move || unsafe {
            let src = [0x41u16; 40];
            core::hint::black_box(wcsncpy(wguarded(k), src.as_ptr(), k));
        })),
        ("wcsnlen", Box::new(move || unsafe {
            core::hint::black_box(wcsnlen(wguarded(k), k));
        })),

        // ── wave 7 ────────────────────────────────────────────────────────────
        // The guarded buffers are NUL-FREE, so a fold or compare that scans for a
        // terminator instead of honouring `n`/`len` walks straight into the guard.
        ("_strnicmp", Box::new(move || unsafe {
            let (a, b) = (guarded(k), guarded(k));
            core::hint::black_box(_strnicmp(a, b, k));
        })),
        ("_memicmp", Box::new(move || unsafe {
            let (a, b) = (guarded(k), guarded(k));
            core::hint::black_box(_memicmp(a.cast(), b.cast(), k));
        })),
        // _strlwr_s/_strupr_s SCAN their own buffer for a terminator within `len`
        // and must stop at len — the whole reason they return EINVAL.
        ("_strlwr_s", Box::new(move || unsafe {
            core::hint::black_box(_strlwr_s(guarded(k), k));
        })),
        ("_strupr_s", Box::new(move || unsafe {
            core::hint::black_box(_strupr_s(guarded(k), k));
        })),
    ];

    println!("\n=== guard-page probe: bounded CRT fns must not read past n ===\n");
    for (name, f) in cases {
        let ok = probes_clean(|| f());
        println!("  {:<10} {}", name, if ok { "✓ in bounds" } else { "✗ OVER-READ (faulted on the guard page)" });
        if !ok {
            failed.push(name);
        }
    }

    if failed.is_empty() {
        println!("\n  all bounded fns stay within n ✓\n");
    } else {
        println!("\n  OVER-READ in: {failed:?}");
        println!("  An n-suffixed CRT fn contains an UNBOUNDED scan. Bound it by n");
        println!("  (cstr_slice_bounded), never scan-then-min. See _strnset's doc comment.\n");
        std::process::exit(1);
    }
}
