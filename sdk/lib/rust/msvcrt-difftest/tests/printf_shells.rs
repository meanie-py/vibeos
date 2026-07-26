//! Wave 14c GRADING — the 30 `va_list` printf ENTRY POINTS vs the genuine `wcs.c`.
//!
//! `printf_port.rs` grades the ENGINE (`pf_printf_a` vs `pfflat_pf_printf_a`) and it
//! grades it hard — 142 narrow + 30 wide rows, byte-identical. It cannot grade what
//! surrounds the engine, and that is where these functions live: which `options` bits go
//! in, whether the trailing NUL is appended through the callback or forced, how a
//! `size_t` buffer size truncates into the C's `int len`, which of `ret < 0` / `ret ==
//! len` decides a `-1`, whether errno is set before or after the handler fires, and
//! whether the `-1` path zeroes the caller's buffer or just terminates it. Every one of
//! those is a shell decision, and a shell that got them all wrong would still pass
//! `printf_port` perfectly.
//!
//! **The reference is `pfflat_*`, never `ref_*`.** build.rs compiles wcs.c a second time
//! under a flat `char*` `<stdarg.h>`, which IS the Windows-x64/aarch64/i386 va_list, and
//! that instantiation is the only callable one: `ref__vsnprintf` forwards `&valist` as
//! `va_list*`, coherent only when a va_list is a plain pointer, and SIGSEGVs under the
//! SysV register-save-area model this host actually compiles with.
//!
//! Both sides consume the SAME hand-packed 8-byte-slot buffer and write into their OWN
//! poisoned buffer, and a row must agree on all four channels:
//!
//!   * the **return value** — a truncation `-1` is the whole point of the `_s` family;
//!   * the **full buffer image**, including a guard prefix, so a NUL written at `str[-1]`
//!     (which `vsnprintf_s_l_opt` genuinely does when `len == 0`) or a stray byte past
//!     the reported length is caught rather than being invisible past `strlen`;
//!   * **errno**, seeded with a sentinel so "did not touch it" is distinguishable from
//!     "wrote 0";
//!   * the **invalid-parameter handler count** — `_s` and `_p` shells are contractually
//!     required to FIRE it, and an app can install one that aborts.
//!
//! **The `%I64d` rows are an oracle canary, not coverage.** wcs.c calls plain `vsnprintf`
//! from five of these shells, and under `_MSVCR_VER=0` it never defines that name — so
//! before build.rs's stage 4 the call left the TU undefined and the linker satisfied it
//! from **glibc**, quietly grading the host libc (no `%I64`, `(nil)` for `%p`, 2-digit
//! exponents). glibc prints `I64d` literally, so if that escape ever comes back these
//! rows fail loudly instead of the oracle lying. See build.rs stage 4 / §9.66.

use std::ffi::CString;
use std::os::raw::c_void;

extern crate msvcrt_crt_rust;

/// Guard bytes before the caller-visible buffer. `vsnprintf_s_l_opt` writes
/// `str[len-1]` with an `int len` that can be 0, and both sides must do the same
/// out-of-bounds thing.
const GUARD: usize = 8;
/// Caller-visible capacity of every scratch buffer.
const CAP: usize = 256;
const POISON_A: u8 = 0xAA;
const POISON_W: u16 = 0xAAAA;

/// Neither 0 nor any real errno value, so "untouched" and "explicitly zeroed" are
/// distinct outcomes. Same convention as classify.rs.
const ERRNO_SEED: i32 = 0x5A5A;

extern "C" {
    // The two independent effect channels (refimpl.c / subjectimpl.c).
    fn ref_errno_get() -> i32;
    fn ref_errno_set(v: i32);
    fn ref_invalid_pmt_count() -> i32;
    fn subj_errno_get() -> i32;
    fn subj_errno_set(v: i32);
    fn subj_invalid_pmt_count() -> i32;
}

// ── the two sides, declared with their real prototypes ───────────────────────
//
// The unprefixed names bind to msvcrt-crt-rust's `#[no_mangle]` shells (the shipping
// artifact); `pfflat_*` is the genuine wcs.c under the flat va_list.

type Loc = *mut c_void;
type Va = *mut u8;

extern "C" {
    // narrow — subject
    fn _vsnprintf(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn _vsnprintf_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn _vsprintf_l(b: *mut u8, f: *const u8, l: Loc, v: Va) -> i32;
    fn vsprintf(b: *mut u8, f: *const u8, v: Va) -> i32;
    fn vsprintf_s(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn _vscprintf(f: *const u8, v: Va) -> i32;
    fn _vscprintf_l(f: *const u8, l: Loc, v: Va) -> i32;
    fn _vscprintf_p(f: *const u8, v: Va) -> i32;
    fn _vscprintf_p_l(f: *const u8, l: Loc, v: Va) -> i32;
    fn _vsnprintf_s(b: *mut u8, sz: usize, c: usize, f: *const u8, v: Va) -> i32;
    fn _vsnprintf_s_l(b: *mut u8, sz: usize, c: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn _vsprintf_s_l(b: *mut u8, c: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn _vsnprintf_c(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn _vsnprintf_c_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn _vsprintf_p(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn _vsprintf_p_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;
    // narrow — reference
    fn pfflat__vsnprintf(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn pfflat__vsnprintf_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vsprintf_l(b: *mut u8, f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat_vsprintf(b: *mut u8, f: *const u8, v: Va) -> i32;
    fn pfflat_vsprintf_s(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn pfflat__vscprintf(f: *const u8, v: Va) -> i32;
    fn pfflat__vscprintf_l(f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vscprintf_p(f: *const u8, v: Va) -> i32;
    fn pfflat__vscprintf_p_l(f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vsnprintf_s(b: *mut u8, sz: usize, c: usize, f: *const u8, v: Va) -> i32;
    fn pfflat__vsnprintf_s_l(b: *mut u8, sz: usize, c: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vsprintf_s_l(b: *mut u8, c: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vsnprintf_c(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn pfflat__vsnprintf_c_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;
    fn pfflat__vsprintf_p(b: *mut u8, n: usize, f: *const u8, v: Va) -> i32;
    fn pfflat__vsprintf_p_l(b: *mut u8, n: usize, f: *const u8, l: Loc, v: Va) -> i32;

    // wide — subject
    fn _vsnwprintf(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn _vsnwprintf_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn _vswprintf(b: *mut u16, f: *const u16, v: Va) -> i32;
    fn _vswprintf_l(b: *mut u16, f: *const u16, l: Loc, v: Va) -> i32;
    fn _vscwprintf(f: *const u16, v: Va) -> i32;
    fn _vscwprintf_l(f: *const u16, l: Loc, v: Va) -> i32;
    fn _vscwprintf_p_l(f: *const u16, l: Loc, v: Va) -> i32;
    fn _vsnwprintf_s(b: *mut u16, sz: usize, c: usize, f: *const u16, v: Va) -> i32;
    fn _vsnwprintf_s_l(b: *mut u16, sz: usize, c: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn _vswprintf_c(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn _vswprintf_c_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn vswprintf_s(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn _vswprintf_s_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn _vswprintf_p_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    // wide — reference
    fn pfflat__vsnwprintf(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn pfflat__vsnwprintf_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat__vswprintf(b: *mut u16, f: *const u16, v: Va) -> i32;
    fn pfflat__vswprintf_l(b: *mut u16, f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat__vscwprintf(f: *const u16, v: Va) -> i32;
    fn pfflat__vscwprintf_l(f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat__vscwprintf_p_l(f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat__vsnwprintf_s(b: *mut u16, sz: usize, c: usize, f: *const u16, v: Va) -> i32;
    fn pfflat__vsnwprintf_s_l(b: *mut u16, sz: usize, c: usize, f: *const u16, l: Loc, v: Va)
        -> i32;
    fn pfflat__vswprintf_c(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn pfflat__vswprintf_c_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat_vswprintf_s(b: *mut u16, n: usize, f: *const u16, v: Va) -> i32;
    fn pfflat__vswprintf_s_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
    fn pfflat__vswprintf_p_l(b: *mut u16, n: usize, f: *const u16, l: Loc, v: Va) -> i32;
}

/// One uniform shape every narrow shell is adapted to. `cap` is the caller-declared
/// buffer size and `count` the `_s` family's separate character budget; a shell that has
/// no such parameter simply ignores the one it lacks, which is exactly the point — the
/// row still runs and still has to agree.
type DriveA = unsafe fn(buf: *mut u8, cap: usize, count: usize, fmt: *const u8, va: Va) -> i32;
type DriveW = unsafe fn(buf: *mut u16, cap: usize, count: usize, fmt: *const u16, va: Va) -> i32;

/// Adapters. Deliberately mechanical: each is the shell's real prototype with the
/// unused uniform parameters dropped, so what a shell does NOT take is visible here.
macro_rules! adapters_a {
    ($( $tag:ident => |$b:ident, $cap:ident, $cnt:ident, $f:ident, $v:ident| $s:expr, $r:expr; )*) => {
        $(
            #[allow(non_snake_case, unused_variables)]
            mod $tag {
                use super::*;
                pub unsafe fn subj($b: *mut u8, $cap: usize, $cnt: usize, $f: *const u8, $v: Va) -> i32 { $s }
                pub unsafe fn refr($b: *mut u8, $cap: usize, $cnt: usize, $f: *const u8, $v: Va) -> i32 { $r }
            }
        )*
    };
}

macro_rules! adapters_w {
    ($( $tag:ident => |$b:ident, $cap:ident, $cnt:ident, $f:ident, $v:ident| $s:expr, $r:expr; )*) => {
        $(
            #[allow(non_snake_case, unused_variables)]
            mod $tag {
                use super::*;
                pub unsafe fn subj($b: *mut u16, $cap: usize, $cnt: usize, $f: *const u16, $v: Va) -> i32 { $s }
                pub unsafe fn refr($b: *mut u16, $cap: usize, $cnt: usize, $f: *const u16, $v: Va) -> i32 { $r }
            }
        )*
    };
}

const NULL_LOC: Loc = std::ptr::null_mut();

adapters_a! {
    a_vsnprintf      => |b, cap, cnt, f, v| _vsnprintf(b, cap, f, v),                      pfflat__vsnprintf(b, cap, f, v);
    a_vsnprintf_l    => |b, cap, cnt, f, v| _vsnprintf_l(b, cap, f, NULL_LOC, v),          pfflat__vsnprintf_l(b, cap, f, NULL_LOC, v);
    a_vsprintf_l     => |b, cap, cnt, f, v| _vsprintf_l(b, f, NULL_LOC, v),                pfflat__vsprintf_l(b, f, NULL_LOC, v);
    a_vsprintf       => |b, cap, cnt, f, v| vsprintf(b, f, v),                             pfflat_vsprintf(b, f, v);
    a_vsprintf_s     => |b, cap, cnt, f, v| vsprintf_s(b, cap, f, v),                      pfflat_vsprintf_s(b, cap, f, v);
    a_vscprintf      => |b, cap, cnt, f, v| _vscprintf(f, v),                              pfflat__vscprintf(f, v);
    a_vscprintf_l    => |b, cap, cnt, f, v| _vscprintf_l(f, NULL_LOC, v),                  pfflat__vscprintf_l(f, NULL_LOC, v);
    a_vscprintf_p    => |b, cap, cnt, f, v| _vscprintf_p(f, v),                            pfflat__vscprintf_p(f, v);
    a_vscprintf_p_l  => |b, cap, cnt, f, v| _vscprintf_p_l(f, NULL_LOC, v),                pfflat__vscprintf_p_l(f, NULL_LOC, v);
    a_vsnprintf_s    => |b, cap, cnt, f, v| _vsnprintf_s(b, cap, cnt, f, v),               pfflat__vsnprintf_s(b, cap, cnt, f, v);
    a_vsnprintf_s_l  => |b, cap, cnt, f, v| _vsnprintf_s_l(b, cap, cnt, f, NULL_LOC, v),   pfflat__vsnprintf_s_l(b, cap, cnt, f, NULL_LOC, v);
    a_vsprintf_s_l   => |b, cap, cnt, f, v| _vsprintf_s_l(b, cnt, f, NULL_LOC, v),         pfflat__vsprintf_s_l(b, cnt, f, NULL_LOC, v);
    a_vsnprintf_c    => |b, cap, cnt, f, v| _vsnprintf_c(b, cap, f, v),                    pfflat__vsnprintf_c(b, cap, f, v);
    a_vsnprintf_c_l  => |b, cap, cnt, f, v| _vsnprintf_c_l(b, cap, f, NULL_LOC, v),        pfflat__vsnprintf_c_l(b, cap, f, NULL_LOC, v);
    a_vsprintf_p     => |b, cap, cnt, f, v| _vsprintf_p(b, cap, f, v),                     pfflat__vsprintf_p(b, cap, f, v);
    a_vsprintf_p_l   => |b, cap, cnt, f, v| _vsprintf_p_l(b, cap, f, NULL_LOC, v),         pfflat__vsprintf_p_l(b, cap, f, NULL_LOC, v);
}

adapters_w! {
    w_vsnwprintf     => |b, cap, cnt, f, v| _vsnwprintf(b, cap, f, v),                     pfflat__vsnwprintf(b, cap, f, v);
    w_vsnwprintf_l   => |b, cap, cnt, f, v| _vsnwprintf_l(b, cap, f, NULL_LOC, v),         pfflat__vsnwprintf_l(b, cap, f, NULL_LOC, v);
    w_vswprintf      => |b, cap, cnt, f, v| _vswprintf(b, f, v),                           pfflat__vswprintf(b, f, v);
    w_vswprintf_l    => |b, cap, cnt, f, v| _vswprintf_l(b, f, NULL_LOC, v),               pfflat__vswprintf_l(b, f, NULL_LOC, v);
    w_vscwprintf     => |b, cap, cnt, f, v| _vscwprintf(f, v),                             pfflat__vscwprintf(f, v);
    w_vscwprintf_l   => |b, cap, cnt, f, v| _vscwprintf_l(f, NULL_LOC, v),                 pfflat__vscwprintf_l(f, NULL_LOC, v);
    w_vscwprintf_p_l => |b, cap, cnt, f, v| _vscwprintf_p_l(f, NULL_LOC, v),               pfflat__vscwprintf_p_l(f, NULL_LOC, v);
    w_vsnwprintf_s   => |b, cap, cnt, f, v| _vsnwprintf_s(b, cap, cnt, f, v),              pfflat__vsnwprintf_s(b, cap, cnt, f, v);
    w_vsnwprintf_s_l => |b, cap, cnt, f, v| _vsnwprintf_s_l(b, cap, cnt, f, NULL_LOC, v),  pfflat__vsnwprintf_s_l(b, cap, cnt, f, NULL_LOC, v);
    w_vswprintf_c    => |b, cap, cnt, f, v| _vswprintf_c(b, cap, f, v),                    pfflat__vswprintf_c(b, cap, f, v);
    w_vswprintf_c_l  => |b, cap, cnt, f, v| _vswprintf_c_l(b, cap, f, NULL_LOC, v),        pfflat__vswprintf_c_l(b, cap, f, NULL_LOC, v);
    w_vswprintf_s    => |b, cap, cnt, f, v| vswprintf_s(b, cap, f, v),                     pfflat_vswprintf_s(b, cap, f, v);
    w_vswprintf_s_l  => |b, cap, cnt, f, v| _vswprintf_s_l(b, cap, f, NULL_LOC, v),        pfflat__vswprintf_s_l(b, cap, f, NULL_LOC, v);
    w_vswprintf_p_l  => |b, cap, cnt, f, v| _vswprintf_p_l(b, cap, f, NULL_LOC, v),        pfflat__vswprintf_p_l(b, cap, f, NULL_LOC, v);
}

/// A shell as the runner sees it: display name, whether it writes into the buffer at all
/// (the `_vsc*` counting ones do not), and the two drivers.
struct ShellA {
    name: &'static str,
    subj: DriveA,
    refr: DriveA,
}
struct ShellW {
    name: &'static str,
    subj: DriveW,
    refr: DriveW,
}

fn shells_a() -> Vec<ShellA> {
    macro_rules! s {
        ($m:ident, $n:literal) => {
            ShellA { name: $n, subj: $m::subj, refr: $m::refr }
        };
    }
    vec![
        s!(a_vsnprintf, "_vsnprintf"),
        s!(a_vsnprintf_l, "_vsnprintf_l"),
        s!(a_vsprintf_l, "_vsprintf_l"),
        s!(a_vsprintf, "vsprintf"),
        s!(a_vsprintf_s, "vsprintf_s"),
        s!(a_vscprintf, "_vscprintf"),
        s!(a_vscprintf_l, "_vscprintf_l"),
        s!(a_vscprintf_p, "_vscprintf_p"),
        s!(a_vscprintf_p_l, "_vscprintf_p_l"),
        s!(a_vsnprintf_s, "_vsnprintf_s"),
        s!(a_vsnprintf_s_l, "_vsnprintf_s_l"),
        s!(a_vsprintf_s_l, "_vsprintf_s_l"),
        s!(a_vsnprintf_c, "_vsnprintf_c"),
        s!(a_vsnprintf_c_l, "_vsnprintf_c_l"),
        s!(a_vsprintf_p, "_vsprintf_p"),
        s!(a_vsprintf_p_l, "_vsprintf_p_l"),
    ]
}

fn shells_w() -> Vec<ShellW> {
    macro_rules! s {
        ($m:ident, $n:literal) => {
            ShellW { name: $n, subj: $m::subj, refr: $m::refr }
        };
    }
    vec![
        s!(w_vsnwprintf, "_vsnwprintf"),
        s!(w_vsnwprintf_l, "_vsnwprintf_l"),
        s!(w_vswprintf, "_vswprintf"),
        s!(w_vswprintf_l, "_vswprintf_l"),
        s!(w_vscwprintf, "_vscwprintf"),
        s!(w_vscwprintf_l, "_vscwprintf_l"),
        s!(w_vscwprintf_p_l, "_vscwprintf_p_l"),
        s!(w_vsnwprintf_s, "_vsnwprintf_s"),
        s!(w_vsnwprintf_s_l, "_vsnwprintf_s_l"),
        s!(w_vswprintf_c, "_vswprintf_c"),
        s!(w_vswprintf_c_l, "_vswprintf_c_l"),
        s!(w_vswprintf_s, "vswprintf_s"),
        s!(w_vswprintf_s_l, "_vswprintf_s_l"),
        s!(w_vswprintf_p_l, "_vswprintf_p_l"),
    ]
}

// ── the corpus ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum V {
    I(i32),
    L(i64),
    F(f64),
    S(&'static [u8]),
    W(&'static [u16]),
    P(u64),
}

/// Hand-packed argument area: one 8-byte slot per argument, which is the Windows-x64
/// and aarch64 model and the one the flat reference was compiled for.
fn pack(args: &[V]) -> Vec<u64> {
    let mut slots: Vec<u64> = Vec::new();
    for a in args {
        match *a {
            V::I(v) => slots.push(v as u32 as u64),
            V::L(v) => slots.push(v as u64),
            V::F(v) => slots.push(v.to_bits()),
            V::S(b) => slots.push(b.as_ptr() as u64),
            V::W(w) => slots.push(w.as_ptr() as u64),
            V::P(p) => slots.push(p),
        }
    }
    slots.extend([0u64; 4]); // slack: an over-reading engine reads zeros, not the stack
    slots
}

static WHELLO: &[u16] = &[b'w' as u16, b'i' as u16, b'd' as u16, b'e' as u16, 0];
static WABCDEF: &[u16] =
    &[b'a' as u16, b'b' as u16, b'c' as u16, b'd' as u16, b'e' as u16, b'f' as u16, 0];
static WR: &[u16] = &[b'r' as u16, 0];
static WL: &[u16] = &[b'l' as u16, 0];
static WLONG20: &[u16] = &[
    b'0' as u16, b'1' as u16, b'2' as u16, b'3' as u16, b'4' as u16, b'5' as u16, b'6' as u16,
    b'7' as u16, b'8' as u16, b'9' as u16, b'a' as u16, b'b' as u16, b'c' as u16, b'd' as u16,
    b'e' as u16, b'f' as u16, b'g' as u16, b'h' as u16, b'i' as u16, b'j' as u16, 0,
];
static WPOS: &[u16] = &[b'p' as u16, b'o' as u16, b's' as u16, 0];
static WMIX: &[u16] = &[b'm' as u16, b'i' as u16, b'x' as u16, 0];

/// (format, args, count-for-the-`_s`-family).
///
/// `count` is chosen per row to reach all three `_s` outcomes: comfortably inside the
/// buffer, exactly filling it (the `ret == len` truncation test), and larger than the
/// buffer (the ERANGE + zero-the-buffer arm).
fn corpus() -> Vec<(&'static str, Vec<V>, usize, usize)> {
    let long = "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij";
    vec![
        ("plain text", vec![], CAP, 32),
        ("", vec![], CAP, 32),
        ("%d", vec![V::I(0)], CAP, 32),
        ("%d", vec![V::I(-2147483648)], CAP, 32),
        ("[%5d][%-5d][%05d]", vec![V::I(42), V::I(42), V::I(42)], CAP, 32),
        ("%u %o %x %X", vec![V::I(-1), V::I(8), V::I(255), V::I(255)], CAP, 32),
        ("%c%c", vec![V::I(b'h' as i32), V::I(0)], CAP, 32),
        ("%s", vec![V::S(b"str\0")], CAP, 32),
        ("%s", vec![V::P(0)], CAP, 32), // NULL %s -> "(null)"
        ("%.2s|%8s|%-8s|", vec![V::S(b"abcdef\0"), V::S(b"r\0"), V::S(b"l\0")], CAP, 32),
        ("%ls", vec![V::W(WHELLO)], CAP, 32),
        ("%p", vec![V::P(0xdead_beef)], CAP, 32),
        ("%p", vec![V::P(0)], CAP, 32),
        // ORACLE CANARY — glibc has no %I64; see the module docstring.
        ("%I64d", vec![V::L(-1234567890123)], CAP, 32),
        ("%I64x", vec![V::L(0x0123_4567_89ab_cdef)], CAP, 32),
        ("%f", vec![V::F(3.5)], CAP, 32),
        ("%f", vec![V::F(0.0)], CAP, 32),
        ("%e|%E", vec![V::F(1.0e-7), V::F(1.0e-7)], CAP, 32),
        ("%g|%G", vec![V::F(0.0001), V::F(100000.0)], CAP, 32),
        ("%.17g", vec![V::F(0.1)], CAP, 32),
        ("%a", vec![V::F(1.5)], CAP, 32),
        ("%%|%%d", vec![], CAP, 32),
        ("%q", vec![V::I(1)], CAP, 32), // unknown conversion: literal, and no advance
        ("%*d", vec![V::I(6), V::I(7)], CAP, 32),
        ("%.*f", vec![V::I(3), V::F(2.0)], CAP, 32),
        // positional — only the `_p` shells set the flag, but every shell must AGREE
        // about what it does with the syntax, which for the others is "treat it as
        // ordinary text/conversions".
        ("%1$s-%2$d", vec![V::S(b"pos\0"), V::I(9)], CAP, 32),
        ("%2$d-%1$s", vec![V::S(b"pos\0"), V::I(9)], CAP, 32),
        ("%1$s-%d", vec![V::S(b"mix\0"), V::I(9)], CAP, 32), // mixed: the <0 EINVAL arm
        // longer than the buffer: truncation, the `_s` -1, the ERANGE arm
        (Box::leak(long.to_string().into_boxed_str()), vec![], CAP, 16),
        ("%s", vec![V::S(b"0123456789abcdefghij\0")], CAP, 8),
        ("%s", vec![V::S(b"0123456789abcdefghij\0")], CAP, 300), // count > cap: ERANGE
        ("exact", vec![], CAP, 5),                              // ret == len boundary
        ("exactly", vec![], CAP, 0),                            // count 0: str[len-1] at [-1]
        // ── the `_s` arms, which need a SMALL BUFFER, not just a small count ──────
        //
        // Every row above leaves the buffer with room to spare, so `ret < 0 || ret ==
        // len` is false and the truncation branch is never entered AT ALL. That made
        // the ERANGE arm — invalid-parameter handler, errno, and zeroing the caller's
        // whole buffer — **vacuous**: halving its `memset` as a negative control moved
        // zero rows. These reach both arms, and the same control then fails them.
        //
        // The two arms are selected by `count > sizeOfBuffer`, so each shape runs with
        // a count above AND below the buffer size.
        ("abcdefgh", vec![], 8, 300),   // exact fit, count > cap  -> ERANGE + zero
        ("abcdefgh", vec![], 8, 7),     // exact fit, count < cap  -> terminate at [len-1]
        ("abcdefgh", vec![], 8, 8),
        ("abcdefghij", vec![], 8, 300), // over-long, count > cap
        ("abcdefghij", vec![], 8, 4),   // over-long, count < cap
        ("abcd", vec![], 8, 3),         // count+1 is the binding limit, not the buffer
        ("abc", vec![], 4, 300),
        ("%d", vec![V::I(12345678)], 8, 300),
        ("", vec![], 1, 300),           // len 1: nothing but the terminator fits
    ]
}

/// The wide corpus is NOT the narrow one with a width swap, and the difference is not
/// cosmetic: in a wide format `%s` consumes a **`wchar_t*`**, so handing it a narrow byte
/// string makes the engine walk memory as UTF-16 until it happens on a zero unit. Both
/// sides read the same bytes and would still agree — but the walk is unbounded, and
/// `_vswprintf`/`_vswprintf_l` pass `INT_MAX` as the length, so the first run of this
/// corpus wrote hundreds of units past the scratch buffer and tripped glibc's allocator
/// (`malloc(): unaligned tcache chunk`). Every `%s` row here therefore carries a wide
/// argument, and `%hs` carries the narrow one.
///
/// One deliberate omission: **`%s` of NULL**. The wide `"(null)"` fallback is printf.h's
/// single wide literal, and gcc compiles it as 32-bit wchars on this host, so the
/// reference emits `['(',0,'n',0,…]` while the target-true engine emits 16-bit units.
/// printf_port excluded the same row for the same reason and pins the Rust side with a
/// dedicated assert — it convicts the HOST, not either implementation.
fn corpus_w() -> Vec<(&'static str, Vec<V>, usize, usize)> {
    let long = "0123456789abcdefghij0123456789abcdefghij0123456789abcdefghij";
    vec![
        ("plain text", vec![], CAP, 32),
        ("", vec![], CAP, 32),
        ("%d", vec![V::I(0)], CAP, 32),
        ("%d", vec![V::I(-2147483648)], CAP, 32),
        ("[%5d][%-5d][%05d]", vec![V::I(42), V::I(42), V::I(42)], CAP, 32),
        ("%u %o %x %X", vec![V::I(-1), V::I(8), V::I(255), V::I(255)], CAP, 32),
        ("%c%c", vec![V::I(b'h' as i32), V::I(0)], CAP, 32),
        ("%s", vec![V::W(WHELLO)], CAP, 32),
        ("%hs", vec![V::S(b"narrow\0")], CAP, 32),
        ("%.2s|%8s|%-8s|", vec![V::W(WABCDEF), V::W(WR), V::W(WL)], CAP, 32),
        ("%ls", vec![V::W(WHELLO)], CAP, 32),
        ("%p", vec![V::P(0xdead_beef)], CAP, 32),
        ("%p", vec![V::P(0)], CAP, 32),
        ("%I64d", vec![V::L(-1234567890123)], CAP, 32),
        ("%I64x", vec![V::L(0x0123_4567_89ab_cdef)], CAP, 32),
        ("%f", vec![V::F(3.5)], CAP, 32),
        ("%f", vec![V::F(0.0)], CAP, 32),
        ("%e|%E", vec![V::F(1.0e-7), V::F(1.0e-7)], CAP, 32),
        ("%g|%G", vec![V::F(0.0001), V::F(100000.0)], CAP, 32),
        ("%.17g", vec![V::F(0.1)], CAP, 32),
        ("%a", vec![V::F(1.5)], CAP, 32),
        ("%%|%%d", vec![], CAP, 32),
        ("%q", vec![V::I(1)], CAP, 32),
        ("%*d", vec![V::I(6), V::I(7)], CAP, 32),
        ("%.*f", vec![V::I(3), V::F(2.0)], CAP, 32),
        ("%1$s-%2$d", vec![V::W(WPOS), V::I(9)], CAP, 32),
        ("%2$d-%1$s", vec![V::W(WPOS), V::I(9)], CAP, 32),
        ("%1$s-%d", vec![V::W(WMIX), V::I(9)], CAP, 32),
        (Box::leak(long.to_string().into_boxed_str()), vec![], CAP, 16),
        ("%s", vec![V::W(WLONG20)], CAP, 8),
        ("%s", vec![V::W(WLONG20)], CAP, 300),
        ("exact", vec![], CAP, 5),
        ("exactly", vec![], CAP, 0),
        // ── the `_s` arms, which need a SMALL BUFFER, not just a small count ──────
        //
        // Every row above leaves the buffer with room to spare, so `ret < 0 || ret ==
        // len` is false and the truncation branch is never entered AT ALL. That made
        // the ERANGE arm — invalid-parameter handler, errno, and zeroing the caller's
        // whole buffer — **vacuous**: halving its `memset` as a negative control moved
        // zero rows. These reach both arms, and the same control then fails them.
        //
        // The two arms are selected by `count > sizeOfBuffer`, so each shape runs with
        // a count above AND below the buffer size.
        ("abcdefgh", vec![], 8, 300),   // exact fit, count > cap  -> ERANGE + zero
        ("abcdefgh", vec![], 8, 7),     // exact fit, count < cap  -> terminate at [len-1]
        ("abcdefgh", vec![], 8, 8),
        ("abcdefghij", vec![], 8, 300), // over-long, count > cap
        ("abcdefghij", vec![], 8, 4),   // over-long, count < cap
        ("abcd", vec![], 8, 3),         // count+1 is the binding limit, not the buffer
        ("abc", vec![], 4, 300),
        ("%d", vec![V::I(12345678)], 8, 300),
        ("", vec![], 1, 300),           // len 1: nothing but the terminator fits
    ]
}

// ── the runner ───────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq)]
struct Effects {
    ret: i32,
    errno: i32,
    pmt: i32,
}

fn esc(b: &[u8]) -> String {
    let mut s = String::new();
    for &c in b {
        match c {
            0x20..=0x7e => s.push(c as char),
            0 => s.push_str("\\0"),
            _ => s.push_str(&format!("\\x{c:02x}")),
        }
    }
    s
}

fn run_a(sh: &ShellA, fmt: &str, args: &[V], cap: usize, count: usize) -> Option<String> {
    let slots = pack(args);
    let fmtz = CString::new(fmt).unwrap();

    let mut rbuf = vec![POISON_A; GUARD + CAP];
    let (re, rimg) = unsafe {
        ref_errno_set(ERRNO_SEED);
        let p0 = ref_invalid_pmt_count();
        let ret = (sh.refr)(
            rbuf.as_mut_ptr().add(GUARD),
            cap,
            count,
            fmtz.as_ptr() as *const u8,
            slots.as_ptr() as *mut u8,
        );
        (Effects { ret, errno: ref_errno_get(), pmt: ref_invalid_pmt_count() - p0 }, rbuf.clone())
    };

    let mut sbuf = vec![POISON_A; GUARD + CAP];
    let (se, simg) = unsafe {
        subj_errno_set(ERRNO_SEED);
        let p0 = subj_invalid_pmt_count();
        let ret = (sh.subj)(
            sbuf.as_mut_ptr().add(GUARD),
            cap,
            count,
            fmtz.as_ptr() as *const u8,
            slots.as_ptr() as *mut u8,
        );
        (Effects { ret, errno: subj_errno_get(), pmt: subj_invalid_pmt_count() - p0 }, sbuf.clone())
    };

    if re == se && rimg == simg {
        return None;
    }
    let n = rimg.iter().zip(&simg).position(|(a, b)| a != b);
    Some(format!(
        "  DIVERGE {:<16} fmt={fmt:?} cap={cap} count={count}\n\
         \x20   ref : ret={} errno={:#x} pmt={} buf={:?}\n\
         \x20   rust: ret={} errno={:#x} pmt={} buf={:?}{}",
        sh.name,
        re.ret,
        re.errno,
        re.pmt,
        esc(&rimg[GUARD..GUARD + 40.min(CAP)]),
        se.ret,
        se.errno,
        se.pmt,
        esc(&simg[GUARD..GUARD + 40.min(CAP)]),
        n.map(|i| format!("\n      first byte differing: index {} (guard={})", i, i < GUARD))
            .unwrap_or_default(),
    ))
}

fn wz(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.chars().map(|c| c as u16).collect();
    v.push(0);
    v
}

fn wesc(w: &[u16]) -> String {
    esc(&w.iter().map(|&c| if c < 0x100 { c as u8 } else { b'?' }).collect::<Vec<u8>>())
}

fn run_w(sh: &ShellW, fmt: &str, args: &[V], cap: usize, count: usize) -> Option<String> {
    let slots = pack(args);
    let fmtw = wz(fmt);

    let mut rbuf = vec![POISON_W; GUARD + CAP];
    let (re, rimg) = unsafe {
        ref_errno_set(ERRNO_SEED);
        let p0 = ref_invalid_pmt_count();
        let ret = (sh.refr)(
            rbuf.as_mut_ptr().add(GUARD),
            cap,
            count,
            fmtw.as_ptr(),
            slots.as_ptr() as *mut u8,
        );
        (Effects { ret, errno: ref_errno_get(), pmt: ref_invalid_pmt_count() - p0 }, rbuf.clone())
    };

    let mut sbuf = vec![POISON_W; GUARD + CAP];
    let (se, simg) = unsafe {
        subj_errno_set(ERRNO_SEED);
        let p0 = subj_invalid_pmt_count();
        let ret = (sh.subj)(
            sbuf.as_mut_ptr().add(GUARD),
            cap,
            count,
            fmtw.as_ptr(),
            slots.as_ptr() as *mut u8,
        );
        (Effects { ret, errno: subj_errno_get(), pmt: subj_invalid_pmt_count() - p0 }, sbuf.clone())
    };

    if re == se && rimg == simg {
        return None;
    }
    Some(format!(
        "  DIVERGE(w) {:<14} fmt={fmt:?} cap={cap} count={count}\n\
         \x20   ref : ret={} errno={:#x} pmt={} buf={:?}\n\
         \x20   rust: ret={} errno={:#x} pmt={} buf={:?}",
        sh.name,
        re.ret,
        re.errno,
        re.pmt,
        wesc(&rimg[GUARD..GUARD + 40.min(CAP)]),
        se.ret,
        se.errno,
        se.pmt,
        wesc(&simg[GUARD..GUARD + 40.min(CAP)]),
    ))
}

fn main() {
    eprintln!("\nwave 14c — wcs.c printf shells vs the genuine C\n");

    let corpus = corpus();
    let mut graded = 0usize;
    let mut fails: Vec<String> = Vec::new();

    for sh in shells_a() {
        let before = fails.len();
        for (fmt, args, cap, count) in &corpus {
            if let Some(d) = run_a(&sh, fmt, args, *cap, *count) {
                fails.push(d);
            }
            graded += 1;
        }
        eprintln!(
            "  {:<18} {:>3} rows {}",
            sh.name,
            corpus.len(),
            if fails.len() == before { "ok" } else { "DIVERGE" }
        );
    }

    let corpus_w = corpus_w();
    for sh in shells_w() {
        let before = fails.len();
        for (fmt, args, cap, count) in &corpus_w {
            if let Some(d) = run_w(&sh, fmt, args, *cap, *count) {
                fails.push(d);
            }
            graded += 1;
        }
        eprintln!(
            "  {:<18} {:>3} rows {}",
            sh.name,
            corpus_w.len(),
            if fails.len() == before { "ok" } else { "DIVERGE" }
        );
    }

    if !fails.is_empty() {
        eprintln!("\n{} diverging rows:\n", fails.len());
        for f in fails.iter().take(40) {
            eprintln!("{f}");
        }
        panic!("printf shells diverge from the genuine wcs.c");
    }

    // A suite that cannot fail is worse than no suite: assert the corpus actually ran.
    assert!(graded > 0, "graded NOTHING");
    println!("\n  {graded} printf shell rows passed\n");
}
