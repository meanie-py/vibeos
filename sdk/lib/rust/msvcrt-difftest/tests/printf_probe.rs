//! printf PROBE — assemble-vs-port decision for wave 14. NOT a gate.
//!
//! Question: does cnt-rust's already-Rust printf engine (`cabi::streamout`)
//! agree with msvcrt's `pf_printf_a` closely enough that the msvcrt printf
//! family can be ASSEMBLED on top of it (thin msvcrt-shaped shells over the
//! proven engine), or does msvcrt's engine have to be PORTED from printf.h?
//!
//! Method (transitivity — we cannot link cnt-rust here, its `cabi` feature
//! collides with the wrappers under test):
//!
//!   streamout-Rust  ==  streamout.c-as-compiled-by-this-recipe   (proven by
//!                       cnt-difftest: same flat va_list, same float model)
//!   streamout.c     vs  ref__snprintf (the GENUINE msvcrt engine, already
//!                       compiled from wcs.c + printf.h in this harness)
//!
//! so every row's verdict transfers verbatim to the Rust engine.
//!
//! Both sides get the same logical argument list:
//!   - so_run reads a hand-packed FLAT va_list (one 8-byte slot per arg —
//!     the model the recipe's <stdarg.h> and cnt-rust's `va_next` share);
//!   - ref__snprintf is called as a REAL C variadic. On SysV x86-64 the
//!     callee pulls integer-class args from the GP save area and doubles
//!     from the XMM save area *by format order within each class*, so one
//!     fixed call shape `(gp0,gp1,gp2, f0,f1,f2,f3)` is exact for any row
//!     with ≤3 int/pointer args and ≤4 doubles (everything stays in
//!     registers; interleaving is irrelevant across classes).
//!
//! The probe PASSES regardless of divergence count — its product is the
//! verdict table. Bounded-output rows are deliberately absent: _snprintf's
//! truncate/-1 semantics live in the SHELL, which the assembly would write
//! fresh in Rust anyway; only the ENGINE (parse + convert + pad) is probed.
//!
//! Host-model caveats (annotated per row, not counted as engine verdicts):
//!   - %p / %Id: this streamout TU compiles without _WIN64, so it sizes
//!     pointers at 32 bits; the msvcrt reference uses the host's 64. On
//!     target both engines agree per arch — the row only shows the *style*
//!     (zero-pad width, case) divergence, so it's graded informational.
//!   - %ls/%S/%C: the recipe stubs mbtowc/wctomb to fail (-1) exactly as
//!     cnt-difftest does (and the Rust engine imports the same externs), so
//!     these rows measure the CONVERSION GAP, not row-level bugs.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

extern "C" {
    fn so_run(base: *mut u8, cnt: i32, fmt: *const u8, va: *mut u8) -> i32;
    // wcs.c's genuine `_vsnprintf` → pf_printf_a, from the pfflat_*
    // instantiation compiled under the FLAT `char*` va_list (see
    // build.rs) — the Windows-x64 va_list model, consuming the same
    // hand-packed slot buffer as so_run. NOT ref__snprintf (forwards to
    // plain `vsnprintf` = GLIBC inside the harness) and NOT
    // ref__vsnprintf (SIGSEGVs under the SysV va_list by construction).
    // The %I64d canary below keeps the oracle honest.
    fn pfflat__vsnprintf(buf: *mut c_char, n: usize, fmt: *const c_char, va: *mut u8) -> c_int;
}

/// One printf argument, carrying its register class.
#[derive(Clone, Copy)]
enum V {
    /// int-class 32-bit (int, char, unsigned — read as 4 bytes of the slot)
    I(i32),
    /// int-class 64-bit (long long / __int64)
    L(i64),
    /// double (XMM class)
    F(f64),
    /// nul-terminated narrow string
    S(&'static [u8]),
    /// nul-terminated wide (16-bit) string
    W(&'static [u16]),
    /// raw pointer value (NULL probes, %p)
    P(u64),
}

struct Outcome {
    verdict: &'static str,
    ref_out: String,
    so_out: String,
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

fn run_row(fmt: &str, args: &[V]) -> Outcome {
    // Pack the flat va_list ONCE; both engines walk the same bytes. Four
    // trailing zero slots so a malformed row that over-consumes an arg
    // reads deterministic zeros on both sides instead of past the Vec.
    let mut slots: Vec<u64> = Vec::new();
    for a in args {
        match *a {
            V::I(v) => slots.push(v as u32 as u64),
            V::L(v) => slots.push(v as u64),
            V::F(v) => slots.push(v.to_bits()),
            V::S(b) => {
                assert_eq!(b.last(), Some(&0), "narrow arg must be nul-terminated");
                slots.push(b.as_ptr() as u64);
            }
            V::W(w) => {
                assert_eq!(w.last(), Some(&0), "wide arg must be nul-terminated");
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
    let va = slots.as_ptr() as *mut u8;
    let sn = unsafe { so_run(sbuf.as_mut_ptr(), 511, fmtz.as_ptr() as *const u8, va) };

    let rlen = rn.clamp(0, 511) as usize;
    let slen = sn.clamp(0, 511) as usize;
    let ref_out = if rn < 0 {
        format!("<ret {rn}>")
    } else {
        esc(&rbuf[..rlen])
    };
    let so_out = if sn < 0 {
        format!("<ret {sn}>")
    } else {
        esc(&sbuf[..slen])
    };
    let verdict = if rn == sn && rbuf[..rlen] == sbuf[..slen] {
        "MATCH  "
    } else {
        "DIVERGE"
    };
    Outcome {
        verdict,
        ref_out,
        so_out,
    }
}

struct Tally {
    section: &'static str,
    matched: usize,
    diverged: usize,
}

fn section(name: &'static str, rows: &[(&str, &[V])], tallies: &mut Vec<Tally>) {
    println!("\n── {name} ─────────────────────────────────────────────");
    let mut t = Tally {
        section: name,
        matched: 0,
        diverged: 0,
    };
    for (fmt, args) in rows {
        let o = run_row(fmt, args);
        if o.verdict.trim() == "MATCH" {
            t.matched += 1;
            println!("  {} {:<18} {:?}", o.verdict, format!("{fmt:?}"), o.ref_out);
        } else {
            t.diverged += 1;
            println!(
                "  {} {:<18} msvcrt={:?}  streamout={:?}",
                o.verdict,
                format!("{fmt:?}"),
                o.ref_out,
                o.so_out
            );
        }
    }
    tallies.push(t);
}

static HELLO: &[u8] = b"Hello, world\0";
static EMPTY: &[u8] = b"\0";
static WHELLO: &[u16] = &[
    b'W' as u16,
    b'i' as u16,
    b'd' as u16,
    b'e' as u16,
    0,
];

#[test]
fn printf_probe() {
    // ── oracle-honesty canary ────────────────────────────────────────
    // %I64d is a Microsoft-only length modifier: the genuine pf engine
    // prints "-5"; glibc parses it as width 64 and right-pads. If this
    // fails, the reference call has escaped to the host libc and every
    // verdict below is meaningless — HARD FAIL, no table.
    {
        let o = run_row("%I64d", &[V::L(-5)]);
        assert_eq!(
            o.ref_out, "-5",
            "ORACLE DISHONEST: the msvcrt reference printed {:?} for %I64d — \
             that is not pf_printf_a (glibc escape?). Fix the harness before \
             reading any verdicts.",
            o.ref_out
        );
    }

    let mut tallies: Vec<Tally> = Vec::new();

    // ── literals, %%, malformed / unknown specifiers ────────────────
    let misc: &[(&str, &[V])] = &[
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
    ];
    section("misc / malformed", misc, &mut tallies);

    // ── integers: conversions, flags, width, precision, star ───────
    let ints: &[(&str, &[V])] = &[
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
    ];
    section("integers", ints, &mut tallies);

    // ── length modifiers ────────────────────────────────────────────
    let lens: &[(&str, &[V])] = &[
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
    ];
    section("length modifiers", lens, &mut tallies);

    // ── chars & strings ─────────────────────────────────────────────
    let strs: &[(&str, &[V])] = &[
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
    ];
    section("chars & strings", strs, &mut tallies);

    // ── wide-on-narrow (mbtowc/wctomb stubbed on the streamout side —
    //    these rows measure the conversion gap, see header) ──────────
    let wides: &[(&str, &[V])] = &[
        ("%ls", &[V::W(WHELLO)]),
        ("%S", &[V::W(WHELLO)]),
        ("%ws", &[V::W(WHELLO)]),
        ("%ls", &[V::P(0)]),
        ("%lc", &[V::I(b'A' as i32)]),
        ("%C", &[V::I(b'A' as i32)]),
    ];
    section("wide args (conversion-gap rows)", wides, &mut tallies);

    // ── pointers (host-model caveat: streamout TU is 32-bit-pointer
    //    here; informational only) ─────────────────────────────────────
    let ptrs: &[(&str, &[V])] = &[
        ("%p", &[V::P(0x1234)]),
        ("%p", &[V::P(0)]),
        ("%#p", &[V::P(0xabcd)]),
        ("%08p", &[V::P(0x42)]),
    ];
    section("pointers (host-model caveat)", ptrs, &mut tallies);

    // ── floats: %f/%e/%g/%E/%G, precision, specials ────────────────
    let floats: &[(&str, &[V])] = &[
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
    ];
    section("floats", floats, &mut tallies);

    // ── hex float %a (gap check: does streamout implement it at all?) ─
    let hexf: &[(&str, &[V])] = &[
        ("%a", &[V::F(1.0)]),
        ("%a", &[V::F(0.5)]),
        ("%A", &[V::F(255.5)]),
        ("%.3a", &[V::F(1.0 / 3.0)]),
    ];
    section("hex float %a (gap check)", hexf, &mut tallies);

    // ── the verdict table ────────────────────────────────────────────
    println!("\n════ PROBE SUMMARY ════════════════════════════════════");
    let (mut m, mut d) = (0, 0);
    for t in &tallies {
        println!(
            "  {:<36} {:>3} match  {:>3} diverge",
            t.section, t.matched, t.diverged
        );
        m += t.matched;
        d += t.diverged;
    }
    println!("  {:<36} {m:>3} match  {d:>3} diverge", "TOTAL");
    println!("(probe, not gate: divergences are the product, not failures)");
}
