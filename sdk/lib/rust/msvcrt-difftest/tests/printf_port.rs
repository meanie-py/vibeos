//! Wave 14b GRADING — the Rust printf ENGINE vs the genuine msvcrt engine.
//!
//! Unlike printf_probe (a third-party C-vs-C comparison that DECIDED the
//! port), this drives the actual shipping code: `msvcrt_crt_rust::printf`'s
//! `pf_printf_a` against `pfflat__vsnprintf` (wcs.c's `_vsnprintf` →
//! `pf_printf_a`, compiled under the flat va_list — see build.rs). Both sides
//! consume the SAME hand-packed 8-byte-slot buffer, both sink into the same
//! `_str_ctx_a` semantics, and every row MUST match byte-for-byte: the port's
//! contract is "the same engine", so a DIVERGE here is a transcription bug,
//! full stop.
//!
//! The subject engine's environment externs (`get_locinfo`, `_errno`,
//! `_get_output_format`, `_mbtowc_l`, `_wctomb_l`, `GetProcessHeap`/
//! `HeapAlloc`/`HeapFree`) resolve to the harness's shared environment
//! (refimpl.c/subjectimpl.c) — the same objects the reference reads, exactly
//! as in msvcrt.dll where the C still owns those symbols.
//!
//! Rows come from the probe's corpus (plus engine-only extras: positional
//! params are exercised at the pf_printf level even though plain sprintf
//! never sets the flag). The %I64d oracle canary is inherited.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

use msvcrt_crt_rust::printf::{arg_clbk_valist, pf_printf_a, puts_clbk_str_a, StrCtxA};

extern "C" {
    fn pfflat__vsnprintf(buf: *mut c_char, n: usize, fmt: *const c_char, va: *mut u8) -> c_int;
}

/// wcs.c:881-890 (`_vsnprintf`), reproduced over the Rust engine: same sink,
/// same trailing-NUL append through the callback, same return.
unsafe fn rust_vsnprintf(buf: *mut u8, len: usize, fmt: *const u8, va: *mut u8) -> i32 {
    let mut ctx = StrCtxA { len, buf };
    let mut cursor = va;
    let ret = pf_printf_a(
        puts_clbk_str_a,
        &mut ctx as *mut StrCtxA as *mut c_void,
        fmt,
        core::ptr::null_mut(),
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut cursor,
    );
    let nul = 0u8;
    puts_clbk_str_a(&mut ctx as *mut StrCtxA as *mut c_void, 1, &nul);
    ret
}

#[derive(Clone, Copy)]
enum V {
    I(i32),
    L(i64),
    F(f64),
    S(&'static [u8]),
    W(&'static [u16]),
    P(u64),
}

fn esc(b: &[u8]) -> String {
    let mut s = String::new();
    for &c in b {
        match c {
            b'\\' => s.push_str("\\\\"),
            0x20..=0x7e => s.push(c as char),
            b'\n' => s.push_str("\\n"),
            b'\0' => s.push_str("\\0"),
            _ => s.push_str(&format!("\\x{c:02x}")),
        }
    }
    s
}

/// Returns None on MATCH, Some(readable diff line) on DIVERGE.
fn run_row(fmt: &str, args: &[V]) -> Option<String> {
    let mut slots: Vec<u64> = Vec::new();
    for a in args {
        match *a {
            V::I(v) => slots.push(v as u32 as u64),
            V::L(v) => slots.push(v as u64),
            V::F(v) => slots.push(v.to_bits()),
            V::S(b) => {
                assert_eq!(b.last(), Some(&0));
                slots.push(b.as_ptr() as u64);
            }
            V::W(w) => {
                assert_eq!(w.last(), Some(&0));
                slots.push(w.as_ptr() as u64);
            }
            V::P(p) => slots.push(p),
        }
    }
    slots.extend([0u64; 4]);

    let fmtz = CString::new(fmt).unwrap();

    let mut rbuf = [0xAAu8; 512];
    let rn = unsafe {
        pfflat__vsnprintf(
            rbuf.as_mut_ptr() as *mut c_char,
            512,
            fmtz.as_ptr(),
            slots.as_ptr() as *mut u8,
        )
    };

    let mut sbuf = [0xAAu8; 512];
    let sn = unsafe {
        rust_vsnprintf(
            sbuf.as_mut_ptr(),
            512,
            fmtz.as_ptr() as *const u8,
            slots.as_ptr() as *mut u8,
        )
    };

    // Compare return + the full 512-byte image (catches over-writes past the
    // reported length and NUL-placement differences, not just the text).
    if rn == sn && rbuf[..] == sbuf[..] {
        return None;
    }
    let rlen = rn.clamp(0, 511) as usize;
    let slen = sn.clamp(0, 511) as usize;
    Some(format!(
        "  DIVERGE {:<18} C(ret {rn})={:?}  rust(ret {sn})={:?}",
        format!("{fmt:?}"),
        esc(&rbuf[..rlen]),
        esc(&sbuf[..slen]),
    ))
}

static HELLO: &[u8] = b"Hello, world\0";
static EMPTY: &[u8] = b"\0";
static WHELLO: &[u16] = &[b'W' as u16, b'i' as u16, b'd' as u16, b'e' as u16, 0];

#[test]
fn printf_port_matches_genuine_engine() {
    // Oracle canary (see printf_probe.rs): the reference must be the real
    // pf engine, not a host-libc escape.
    {
        let mut buf = [0u8; 64];
        let slots: [u64; 5] = [(-5i64) as u64, 0, 0, 0, 0];
        let rn = unsafe {
            pfflat__vsnprintf(
                buf.as_mut_ptr() as *mut c_char,
                64,
                b"%I64d\0".as_ptr() as *const c_char,
                slots.as_ptr() as *mut u8,
            )
        };
        assert_eq!(
            (rn, &buf[..2]),
            (2, &b"-5"[..]),
            "ORACLE DISHONEST — reference is not pf_printf_a"
        );
    }

    let rows: &[(&str, &[V])] = &[
        // misc / malformed
        ("plain text", &[]),
        ("100%%", &[]),
        ("%%d", &[]),
        ("a%%b%%c", &[]),
        ("%q", &[V::I(42)]),
        ("%-", &[]),
        ("%5", &[V::I(42)]),
        ("%.", &[V::I(42)]),
        ("% ", &[]),
        ("%", &[]),
        // integers
        ("%d", &[V::I(0)]),
        ("%d", &[V::I(42)]),
        ("%d", &[V::I(-42)]),
        ("%i", &[V::I(-1)]),
        ("%d", &[V::I(i32::MIN)]),
        ("%d", &[V::I(i32::MAX)]),
        ("%u", &[V::I(0)]),
        ("%u", &[V::I(-294967296i64 as i32)]),
        ("%x", &[V::I(0xdeadbeefu32 as i32)]),
        ("%X", &[V::I(0xabcd)]),
        ("%o", &[V::I(0o777)]),
        ("%5d", &[V::I(42)]),
        ("%-5d", &[V::I(42)]),
        ("%05d", &[V::I(42)]),
        ("%+d", &[V::I(42)]),
        ("%+d", &[V::I(-42)]),
        ("% d", &[V::I(42)]),
        ("%8.5d", &[V::I(42)]),
        ("%.0d", &[V::I(0)]),
        ("%#x", &[V::I(0x2a)]),
        ("%#X", &[V::I(0x2a)]),
        ("%#o", &[V::I(0o17)]),
        ("%#x", &[V::I(0)]),
        ("%10.4x", &[V::I(0xab)]),
        ("%-#10x", &[V::I(0x2a)]),
        ("%0-5d", &[V::I(42)]),
        ("%+ d", &[V::I(7)]),
        ("%*d", &[V::I(6), V::I(42)]),
        ("%*d", &[V::I(-6), V::I(42)]),
        ("%.*d", &[V::I(4), V::I(42)]),
        ("%.*d", &[V::I(-4), V::I(42)]),
        ("%*.*d", &[V::I(8), V::I(4), V::I(42)]),
        ("a=%d b=%x c=%u", &[V::I(1), V::I(0x10), V::I(99)]),
        // length modifiers
        ("%hd", &[V::I(0x12345678u32 as i32)]),
        ("%hu", &[V::I(-1)]),
        ("%hhd", &[V::I(0x1234)]),
        ("%hx", &[V::I(0xabcd1234u32 as i32)]),
        ("%ld", &[V::I(-5)]),
        ("%lu", &[V::I(-1)]),
        ("%lld", &[V::L(0x1_0000_0000)]),
        ("%llx", &[V::L(0xdead_beef_cafe)]),
        ("%llu", &[V::L(-1)]),
        ("%I64d", &[V::L(-5)]),
        ("%I64x", &[V::L(0x1234_5678_9abc_def0u64 as i64)]),
        ("%I32d", &[V::I(-5)]),
        ("%Id", &[V::L(-5)]),
        ("%zd", &[V::L(42)]),
        ("%jd", &[V::L(42)]),
        ("%td", &[V::L(42)]),
        ("%wd", &[V::I(42)]),
        // chars & strings
        ("%c", &[V::I(b'A' as i32)]),
        ("%c", &[V::I(0)]),
        ("%5c", &[V::I(b'Z' as i32)]),
        ("%-5c", &[V::I(b'Z' as i32)]),
        ("%s", &[V::S(HELLO)]),
        ("%s", &[V::S(EMPTY)]),
        ("%s", &[V::P(0)]),
        ("%10s", &[V::S(HELLO)]),
        ("%-15s", &[V::S(HELLO)]),
        ("%.5s", &[V::S(HELLO)]),
        ("[%8.3s]", &[V::S(HELLO)]),
        ("%.0s", &[V::S(HELLO)]),
        ("%hs", &[V::S(HELLO)]),
        // wide args (shared _wctomb_l/_mbtowc_l environment on both sides)
        ("%ls", &[V::W(WHELLO)]),
        ("%S", &[V::W(WHELLO)]),
        ("%ws", &[V::W(WHELLO)]),
        ("%ls", &[V::P(0)]),
        ("%lc", &[V::I(b'A' as i32)]),
        ("%C", &[V::I(b'A' as i32)]),
        // pointers
        ("%p", &[V::P(0x1234)]),
        ("%p", &[V::P(0)]),
        ("%#p", &[V::P(0xabcd)]),
        ("%08p", &[V::P(0x42)]),
        // floats
        ("%f", &[V::F(0.0)]),
        ("%f", &[V::F(-0.0)]),
        ("%f", &[V::F(1.0)]),
        ("%f", &[V::F(-1.0)]),
        ("%f", &[V::F(0.5)]),
        ("%f", &[V::F(3.141592653589793)]),
        ("%f", &[V::F(1234.5678)]),
        ("%f", &[V::F(1e-6)]),
        ("%f", &[V::F(123456789.0)]),
        ("%e", &[V::F(0.0)]),
        ("%e", &[V::F(1.0)]),
        ("%e", &[V::F(3.141592653589793)]),
        ("%e", &[V::F(0.000123)]),
        ("%e", &[V::F(1e17)]),
        ("%e", &[V::F(1e-300)]),
        ("%e", &[V::F(1e300)]),
        ("%E", &[V::F(12345.6789)]),
        ("%g", &[V::F(0.0)]),
        ("%g", &[V::F(1.0)]),
        ("%g", &[V::F(0.0001)]),
        ("%g", &[V::F(0.00001)]),
        ("%g", &[V::F(100000.0)]),
        ("%g", &[V::F(1000000.0)]),
        ("%g", &[V::F(3.141592653589793)]),
        ("%G", &[V::F(1e-10)]),
        ("%.0f", &[V::F(0.5)]),
        ("%.0f", &[V::F(1.5)]),
        ("%.0f", &[V::F(2.5)]),
        ("%.2f", &[V::F(0.125)]),
        ("%.2f", &[V::F(0.135)]),
        ("%.17g", &[V::F(3.141592653589793)]),
        ("%.17g", &[V::F(0.1)]),
        ("%.30f", &[V::F(0.1)]),
        ("%20.10e", &[V::F(-2.5)]),
        ("%-20g|", &[V::F(2.5)]),
        ("%+f", &[V::F(1.0)]),
        ("%#.0f", &[V::F(2.0)]),
        ("%#g", &[V::F(1.0)]),
        ("%010.3f", &[V::F(-1.5)]),
        ("%.3g", &[V::F(999999.0)]),
        ("%f", &[V::F(1e-300)]),
        ("%f", &[V::F(5e-324)]),
        ("%e", &[V::F(5e-324)]),
        ("%g", &[V::F(f64::MAX)]),
        ("%f", &[V::F(f64::INFINITY)]),
        ("%f", &[V::F(f64::NEG_INFINITY)]),
        ("%f", &[V::F(f64::NAN)]),
        ("%e", &[V::F(f64::NAN)]),
        ("%g", &[V::F(f64::INFINITY)]),
        ("x=%f y=%e", &[V::F(1.5), V::F(2.5)]),
        ("%d %f %s", &[V::I(7), V::F(0.25), V::S(HELLO)]),
        // hex float
        ("%a", &[V::F(1.0)]),
        ("%a", &[V::F(0.5)]),
        ("%A", &[V::F(255.5)]),
        ("%.3a", &[V::F(1.0 / 3.0)]),
        // big width / padding paths (exercise pf_fill + the HeapAlloc arm)
        ("%128d", &[V::I(7)]),
        ("%-200s|", &[V::S(HELLO)]),
        ("%.100d", &[V::I(42)]),
        ("%300.250x", &[V::I(0xbeef)]),
    ];

    let mut diverges: Vec<String> = Vec::new();
    for (fmt, args) in rows {
        if let Some(d) = run_row(fmt, args) {
            diverges.push(d);
        }
    }
    println!(
        "printf_port: {}/{} rows match",
        rows.len() - diverges.len(),
        rows.len()
    );
    if !diverges.is_empty() {
        for d in &diverges {
            println!("{d}");
        }
        panic!(
            "{} printf_port rows diverge from the genuine engine — transcription bugs",
            diverges.len()
        );
    }
}
