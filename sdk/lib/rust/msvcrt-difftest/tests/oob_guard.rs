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
