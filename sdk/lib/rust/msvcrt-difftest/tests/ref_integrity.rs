//! Does the REFERENCE still behave like the C it was compiled from?
//!
//! Every other harness in this crate treats the compiled `string.c`/`wcs.c` as
//! ground truth. This one audits that assumption — it is the only test here
//! whose subject is the *oracle* rather than the code under test.
//!
//! WHY IT EXISTS (§9.66). The rename prelude deliberately lands `#define atoi
//! ref_atoi` BEFORE the host headers so that glibc declares the renamed symbol
//! and we inherit a correct prototype for free. But glibc's declaration carries
//! more than a prototype — it carries ATTRIBUTES:
//!
//!     extern int atoi (const char *) __THROW __attribute_pure__ __nonnull ((1));
//!
//! After the rename that is a declaration of `ref_atoi` promising (a) the arg is
//! never NULL and (b) the call has no side effects. Both are FALSE for msvcrt:
//! its `atoi` has an explicit `if(!str) return 0;` (string.c:1720), and its
//! `strtol` family sets errno. Armed with (a), the optimizer simply DELETED the
//! NULL guard — so the "genuine C reference" faulted on an input the real DLL
//! handles. Armed with (b) it may CSE or elide errno-setting calls, which would
//! quietly corrupt the errno oracle without ever crashing.
//!
//! The prelude now neuters glibc's attribute macros (build.rs, "stage 1.5").
//! This test is the tripwire that keeps it neutered.
//!
//! It asserts only what the C SOURCE actually promises — nothing aspirational.
//! Each call runs in a forked child, so a deleted guard is *reported by name*
//! rather than taking the run down with a bare SIGSEGV.

use core::ptr::null_mut;

extern "C" {
    fn ref_atoi(s: *const u8) -> i32;
    fn ref_atol(s: *const u8) -> i32;
    fn ref__atoi64(s: *const u8) -> i64;
    fn ref_strtol(s: *const u8, e: *mut *mut u8, b: i32) -> i32;
    fn ref_strtoul(s: *const u8, e: *mut *mut u8, b: i32) -> u32;
    fn ref_errno_get() -> i32;
    fn ref_errno_set(v: i32);
    fn ref_invalid_pmt_count() -> i32;
    fn ref_invalid_pmt_reset();
}

/// Run `f` in a forked child. `None` if it died on a signal (a deleted guard);
/// `Some(v)` with the value it produced otherwise.
fn probe(f: impl FnOnce() -> i64) -> Option<i64> {
    unsafe {
        let mut fds = [0i32; 2];
        assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
        let pid = libc::fork();
        if pid == 0 {
            libc::close(fds[0]);
            let v = f();
            libc::write(fds[1], &v as *const i64 as *const _, 8);
            libc::_exit(0);
        }
        libc::close(fds[1]);
        let mut v: i64 = 0;
        let n = libc::read(fds[0], &mut v as *mut i64 as *mut _, 8);
        libc::close(fds[0]);
        let mut st = 0i32;
        libc::waitpid(pid, &mut st, 0);
        if libc::WIFEXITED(st) && libc::WEXITSTATUS(st) == 0 && n == 8 { Some(v) } else { None }
    }
}

fn main() {
    println!("\n=== reference integrity: does the compiled C still honour its own guards? ===\n");
    let mut bad: Vec<String> = Vec::new();

    // ── 1. the NULL guards the C source explicitly writes ──────────────────
    //
    // msvcrt's atoi: `if(!str) return 0;`            (string.c:1720)
    // _atoi64/strtol/strtoul funnel into _strtoi64_l, whose first act is
    // `if (!MSVCRT_CHECK_PMT(nptr != NULL)) return 0;` — return 0, set EINVAL,
    // and FIRE the invalid-parameter handler. All five must survive NULL.
    let null_cases: Vec<(&str, Box<dyn Fn() -> i64>)> = vec![
        ("atoi",    Box::new(|| unsafe { ref_atoi(core::ptr::null()) as i64 })),
        ("atol",    Box::new(|| unsafe { ref_atol(core::ptr::null()) as i64 })),
        ("_atoi64", Box::new(|| unsafe { ref__atoi64(core::ptr::null()) })),
        ("strtol",  Box::new(|| unsafe { ref_strtol(core::ptr::null(), null_mut(), 10) as i64 })),
        ("strtoul", Box::new(|| unsafe { ref_strtoul(core::ptr::null(), null_mut(), 10) as i64 })),
    ];

    for (name, f) in null_cases {
        match probe(|| f()) {
            None => {
                println!("  ref_{name:<9} ✗ FAULTED on NULL — its guard was optimized away");
                bad.push(format!("ref_{name} faults on NULL"));
            }
            Some(v) if v != 0 => {
                println!("  ref_{name:<9} ✗ returned {v} on NULL (the C returns 0)");
                bad.push(format!("ref_{name} returned {v} on NULL"));
            }
            Some(_) => println!("  ref_{name:<9} ✓ NULL -> 0, guard intact"),
        }
    }

    // ── 2. `pure` must NOT have been inherited ─────────────────────────────
    //
    // The subtle half, and the one that never crashes. glibc marks strtol
    // __attribute_pure__ ("no side effects"). msvcrt's SETS ERRNO on overflow.
    // If `pure` leaked, the compiler is entitled to elide a call whose return
    // value is unused — and the errno write vanishes with it.
    //
    // So: call it for its SIDE EFFECT ONLY, discarding the result, and demand
    // the side effect still happened. A `pure` ref_strtol optimizes this away.
    let huge = b"99999999999999999999\0";
    let observed = probe(|| unsafe {
        ref_errno_set(0);
        ref_invalid_pmt_reset();
        core::hint::black_box(ref_strtol(huge.as_ptr(), null_mut(), 10)); // result DISCARDED
        ref_errno_get() as i64
    });
    match observed {
        Some(34) => println!("  ref_strtol   ✓ overflow set errno=ERANGE with its result discarded — not `pure`"),
        Some(0) => {
            println!("  ref_strtol   ✗ errno UNSET after a discarded overflowing call");
            println!("               -> glibc's __attribute_pure__ leaked; the call was elided.");
            bad.push("ref_strtol inherited __attribute_pure__".into());
        }
        Some(e) => println!("  ref_strtol   ? errno={e} (expected 34/ERANGE) — check the C's clamp path"),
        None => { println!("  ref_strtol   ✗ FAULTED"); bad.push("ref_strtol faulted".into()); }
    }

    // ── 3. the invalid-parameter handler must actually fire ────────────────
    // NULL into strtol goes through MSVCRT_CHECK_PMT, which fires the handler.
    // If this reads 0 the reference's handler is not wired up and every
    // handler-comparison in classify.rs is vacuously true.
    let fired = probe(|| unsafe {
        ref_invalid_pmt_reset();
        ref_errno_set(0);
        core::hint::black_box(ref_strtol(core::ptr::null(), null_mut(), 10));
        ref_invalid_pmt_count() as i64
    });
    match fired {
        Some(n) if n > 0 => println!("  ref_strtol   ✓ invalid-parameter handler fired on NULL ({n}x)"),
        Some(_) => {
            println!("  ref_strtol   ✗ handler did NOT fire on NULL — CHECK_PMT is not wired up");
            bad.push("reference invalid-parameter handler never fires".into());
        }
        None => { println!("  ref_strtol   ✗ FAULTED"); bad.push("ref_strtol faulted on NULL".into()); }
    }

    if bad.is_empty() {
        println!("\n  the reference honours its own source. the oracle can be trusted.\n");
    } else {
        println!("\n  REFERENCE IS MISCOMPILED — the oracle is lying:");
        for b in &bad { println!("    - {b}"); }
        println!("\n  The host's headers are rewriting the C's semantics. See build.rs");
        println!("  \"stage 1.5\" (the glibc attribute disarm) and field manual §9.66.");
        println!("  Do NOT trust any classify/perf result until this is green.\n");
        std::process::exit(1);
    }
}
