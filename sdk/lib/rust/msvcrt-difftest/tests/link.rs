//! Link + liveness smoke for the msvcrt C reference.
//!
//! This is NOT the differential suite (that comes next) — it proves the two
//! things the reference half has to get right before any fuzzing is worth
//! running:
//!
//!   1. **The binary links.** The reference objects and cnt-rust's
//!      `#[no_mangle]` shims coexist: no duplicate symbol, no undefined
//!      symbol. (cnt-rust exports `memmove`/`strlen`/`_invalid_parameter`/…,
//!      which is exactly why every CRT import of the reference is renamed
//!      into the `msvcrt_ref_*` namespace — see refstub/msvcrt.h.)
//!   2. **The reference is the genuine msvcrt.** Each assert below is a
//!      behaviour where msvcrt is NOT the textbook libc, so a stub or a
//!      mis-linked host function would fail it.
//!
//! `harness = false`, single-threaded, for the reason cnt-difftest documents:
//! cnt-rust's `#[no_mangle] getenv` shadows glibc's, libtest reads
//! `RUST_TEST_THREADS` through it, gets NULL, and misbehaves.

use msvcrt_difftest::*;
use std::ffi::CString;

fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn main() {
    let mut failed = 0usize;

    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            let ok: bool = $cond;
            println!("{} {}", if ok { "ok  " } else { "FAIL" }, $name);
            if !ok {
                failed += 1;
            }
        }};
    }

    // ── plain string surface ──────────────────────────────────────────
    let hello = cstr("hello");
    check!("ref_strlen", unsafe { ref_strlen(hello.as_ptr()) } == 5);

    let abc = cstr("abc");
    let abd = cstr("abd");
    // msvcrt's 32-bit strncmp returns the raw byte difference ('c'-'d' == -1
    // happens to look like a sign, so use a wider gap to actually pin it).
    let aaa = cstr("aaa");
    let aza = cstr("aza");
    check!("ref_strcmp sign", unsafe { ref_strcmp(abc.as_ptr(), abd.as_ptr()) } < 0);
    check!("ref_strncmp raw difference (32-bit msvcrt fork)", unsafe {
        ref_strncmp(aaa.as_ptr(), aza.as_ptr(), 3)
    } == (b'a' as i32 - b'z' as i32));

    // strstr's KMP path — the one that needs ARRAY_SIZE(lps). A long needle
    // is what pushes it off the naive path.
    let hay = cstr("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabxyzzy");
    let needle = cstr("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabxyzzy");
    check!("ref_strstr (KMP path)", unsafe {
        ref_strstr(hay.as_ptr(), needle.as_ptr())
    } == hay.as_ptr() as *mut _);

    // ── memory ────────────────────────────────────────────────────────
    // msvcrt's memcpy IS memmove (it tail-calls it) — overlap-safe, unlike
    // the textbook memcpy. That is a load-bearing msvcrt misfeature.
    let mut buf = *b"abcdefgh\0";
    unsafe {
        ref_memcpy(
            buf.as_mut_ptr().add(2) as *mut _,
            buf.as_ptr() as *const _,
            4,
        );
    }
    check!("ref_memcpy is overlap-safe (msvcrt aliases memmove)", &buf[..8] == b"ababcdgh");

    // ── the Windows 32-bit `long` model ───────────────────────────────
    // _ltoa reinterprets a negative value as UNSIGNED for radix != 10. With a
    // host 64-bit long this would print "ffffffffffffffff"; msvcrt prints the
    // 32-bit "ffffffff". This is the single sharpest test of the LP32 model.
    let mut out = [0i8; 96];
    unsafe { ref__ltoa(-1, out.as_mut_ptr(), 16) };
    let got = unsafe { std::ffi::CStr::from_ptr(out.as_ptr()) };
    check!(
        "ref__ltoa(-1,16) == \"ffffffff\" (32-bit long)",
        got.to_bytes() == b"ffffffff"
    );

    // ...and the saturation that only bites if LONG_MAX is 32-bit.
    let big = cstr("99999999999");
    ref_errno_set_wrapper(0);
    let v = unsafe { ref_strtol(big.as_ptr(), std::ptr::null_mut(), 10) };
    check!("ref_strtol saturates at 32-bit LONG_MAX", v == i32::MAX);
    check!("...and sets ERANGE", ref_errno_get_wrapper() == errno::ERANGE);

    // ── locale-sensitive paths take the genuine C-locale branch ───────
    let mixed = cstr("AbC");
    let mut m = mixed.into_bytes_with_nul();
    unsafe { ref__strlwr(m.as_mut_ptr() as *mut i8) };
    check!("ref__strlwr (C-locale CTYPE branch)", &m[..3] == b"abc");

    // strcoll in the C locale is strcmp — proves lc_handle[LC_COLLATE] == 0
    // and that we did NOT fall into the CompareStringA stub.
    check!("ref_strcoll == strcmp in the C locale", unsafe {
        ref_strcoll(abc.as_ptr(), abd.as_ptr())
    } < 0);

    // ── float parse (real bnum.h, real fpnum_parse) ───────────────────
    let pi = cstr("3.14159265358979");
    check!("ref_strtod (genuine bnum bignum path)", unsafe {
        (ref_strtod(pi.as_ptr(), std::ptr::null_mut()) - 3.14159265358979).abs() < 1e-15
    });

    // ── coexistence: the reference and the shim under test, same process ──
    // If this links and both answer, the whole premise of the harness holds.
    extern "C" {
        fn strlen(s: *const i8) -> usize; // cnt-rust's #[no_mangle] shim
    }
    check!("cnt-rust shim + msvcrt reference agree on strlen", unsafe {
        strlen(hello.as_ptr()) == ref_strlen(hello.as_ptr())
    });

    println!();
    if failed == 0 {
        println!("all reference checks passed");
    } else {
        println!("{failed} FAILED");
        std::process::exit(1);
    }
}

fn ref_errno_get_wrapper() -> i32 {
    unsafe { ref_errno_get() }
}
fn ref_errno_set_wrapper(v: i32) {
    unsafe { ref_errno_set(v) }
}
