//! Wave 14b — msvcrt's **printf engine** (`dll/win32/msvcrt/printf.h`), in Rust.
//!
//! # What this is
//!
//! A faithful, bit-exact transcription of the Wine/ReactOS msvcrt printf state
//! machine: `pf_printf` (the format parser/dispatcher), the `puts_clbk` sink
//! contract, `pf_output_fp` (the base-10 bignum float formatter, sharing
//! `crate::bnum` with the wave-13 parser so `%f` and `strtod` run the SAME limb
//! arithmetic), `pf_output_hex_fp` (`%a`), `pf_integer_conv`, the wide/narrow
//! string cross-conversion paths, and the positional-argument (`%1$d`) machinery
//! from both `printf.h` and `wcs.c`.
//!
//! The C is a template header instantiated twice (`#include "printf.h"` narrow,
//! then `#define PRINTF_WIDE` + include again). Here the two instantiations are
//! ONE generic engine over [`PfChar`] (u8 narrow / u16 wide), monomorphized the
//! same way the preprocessor did — same trick as cnt-rust's `streamout.rs`.
//!
//! # Contract notes (READ BEFORE EDITING)
//!
//! * **Two CRT versions, selected PER CALL.** msvcrt.dll builds
//!   `-D_MSVCR_VER=0` (`dll/win32/msvcrt/CMakeLists.txt`), so its option flags
//!   come from `_get_output_format()` + hardcoded legacy TRUEs, NOT from the
//!   `options` word; ucrtbase is `_MSVCR_VER >= 140` and reads the word. Both
//!   arms are transcribed, and `PRINTF_UCRT_SEMANTICS` in the `options` word
//!   picks between them.
//!
//!   That used to be a compile-time `MSVCR_VER` const, i.e. a property of the
//!   crate. It could never have served ucrtbase, which is structurally
//!   forbidden from depending on this crate (see crt-shared-rust's Cargo.toml:
//!   that wall is what stops 856 genuine UCRT bodies being silently replaced by
//!   msvcrt-graded ones). A cargo feature would have been worse — unification
//!   across a build graph could hand msvcrt the 140 behaviour with nothing to
//!   notice. A runtime bit is also simply what UCRT does: its entry points take
//!   an options mask so behaviour is per-call. See `PRINTF_UCRT_SEMANTICS`.
//!
//!   NOTE this is printf OUTPUT only. `fpnum.rs` still has genuinely ABSENT
//!   `_MSVCR_VER >= 140` functionality on the input side — hex-float parsing,
//!   the "infinity"/"nan" words, `strtof` — which is missing code, not a gate,
//!   and belongs to the convert-family work.
//! * **Bugs are kept, and marked.** Grep `BUG-KEPT` / `XXX-VERIFY`.
//! * **i386 no-SSE discipline** (house style, see `fpnum.rs`): no Rust-ABI
//!   fn returns/receives `f64` — float-bearing fns are `extern "C"`; float
//!   classification (signbit/isinf/isnan) is done on the raw `u64` bits read
//!   through a pointer, never via `f64::is_infinite`/`from_bits`.
//! * **va_list is a flat byte cursor** (`*mut u8`) on all three Windows targets;
//!   `va_arg` = read T + advance by the 4/8-byte slot — the exact model of
//!   `cnt-rust/src/cabi/streamout.rs`.
//! * No allocation except where the C allocates (`HeapAlloc` in the string
//!   cross-conversion + oversized-integer paths).
//!
//! Every function carries a `// printf.h:NNN` (or `// wcs.c:NNN`) line reference
//! against THIS tree's copy of the C.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::comparison_chain)]

use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

use crate::bnum::{log10i, Bnum, BNUM_PREC64, LIMB_DIGITS, LIMB_MAX, P10S};

// ── constants ─────────────────────────────────────────────────────────────────


// bnum.h:22-23 (the float formatter shares these with fpnum.rs)
const EXP_BITS: i32 = 11;
const MANT_BITS: i32 = 53;

// msvcrt.h:409-410
pub const MSVCRT_PRINTF_POSITIONAL_PARAMS: u32 = 0x0100;
pub const MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER: u32 = 0x0200;

// include/corecrt_stdio_config.h:12-17 — the ucrt option bits. Only consulted by
// the ucrt arm, reached when the caller passes PRINTF_UCRT_SEMANTICS.
pub const CRT_INTERNAL_PRINTF_LEGACY_VSPRINTF_NULL_TERMINATION: u32 = 0x0001;
pub const CRT_INTERNAL_PRINTF_STANDARD_SNPRINTF_BEHAVIOR: u32 = 0x0002;
pub const CRT_INTERNAL_PRINTF_LEGACY_WIDE_SPECIFIERS: u32 = 0x0004;
pub const CRT_INTERNAL_PRINTF_LEGACY_MSVCRT_COMPATIBILITY: u32 = 0x0008;
pub const CRT_INTERNAL_PRINTF_LEGACY_THREE_DIGIT_EXPONENTS: u32 = 0x0010;
pub const CRT_INTERNAL_PRINTF_STANDARD_ROUNDING: u32 = 0x0020;

/// OURS, not Microsoft's — deliberately above their 0x0001..0x0020 block and
/// above our own 0x0100/0x0200 so it cannot collide with either.
///
/// Selects UCRT semantics (`_MSVCR_VER >= 140`) instead of MSVCRT's
/// (`_MSVCR_VER == 0`). This used to be the compile-time `MSVCR_VER` const,
/// which made the engine's version a property of the CRATE. ucrtbase cannot
/// reach this crate at all — `crt-shared-cargo` is structurally forbidden from
/// depending on msvcrt-crt-rust, which is what stops 856 genuine UCRT bodies
/// being silently replaced by msvcrt-graded ones — so a per-crate const could
/// never have served both DLLs, and a cargo feature would have been worse:
/// feature unification across one build graph could quietly hand msvcrt the
/// 140 behaviour with nothing to notice.
///
/// A runtime bit is also what the real UCRT does. Its public entry points take
/// an options mask precisely so behaviour is selected per call, so threading
/// this alongside the other option bits matches the thing we are implementing
/// rather than working around it. The cost is that msvcrt.dll no longer
/// dead-code-eliminates the 140 arm; that is a few hundred bytes and no
/// behaviour change, which is worth paying to delete a build-flag hazard.
pub const PRINTF_UCRT_SEMANTICS: u32 = 0x0400;

/// include/stdio.h:68 — `_TWO_DIGIT_EXPONENT`, `_get_output_format`'s one flag.
const TWO_DIGIT_EXPONENT: u32 = 0x1;

/// include/corecrt.h:157 — `_ARGMAX`, the positional-argument table size.
pub const ARGMAX: i32 = 100;

// VARENUM subset the engine uses as its arg-type vocabulary (sdk/include/psdk/wtypes.idl:734-754).
pub const VT_R8: i32 = 5;
pub const VT_I8: i32 = 20;
pub const VT_INT: i32 = 22;
pub const VT_PTR: i32 = 26;

// wcs.c:42 — `enum { LEN_DEFAULT, LEN_SHORT, LEN_LONG } IntegerLength;`
pub const LEN_DEFAULT: i32 = 0;
pub const LEN_SHORT: i32 = 1;
pub const LEN_LONG: i32 = 2;

/// dll/win32/msvcrt/include/limits.h:7
const MB_LEN_MAX: usize = 5;

/// wcs.c:49 — `static BOOL n_format_enabled = TRUE;`. Shared by BOTH engine
/// widths and by `_get_printf_count_output`/`_set_printf_count_output`
/// (wcs.c:61-74, shells owed to integration). The C is a plain non-atomic BOOL;
/// relaxed atomics are the same codegen class without edition-2024 static-mut
/// friction.
pub static N_FORMAT_ENABLED: AtomicI32 = AtomicI32::new(1);

// ── externs ───────────────────────────────────────────────────────────────────
//
// All of these resolve to msvcrt's OWN implementations once this object is
// linked into msvcrt.dll (same pattern as fpnum.rs / lib.rs). `crate::_errno`,
// `crate::_invalid_parameter`, `crate::check_pmt`, `crate::invalid_pmt`,
// `crate::locinfo_of` and `crate::EINVAL` are reused from lib.rs rather than
// re-declared.

extern "C" {
    /// wcs.c:2147 — `int CDECL _wctomb_l(char *dst, wchar_t ch, _locale_t locale)`
    fn _wctomb_l(dst: *mut u8, ch: u16, locale: *mut c_void) -> i32;
    /// mbcs.c:3173 — `int CDECL _mbtowc_l(wchar_t *dst, const char *str, size_t n, _locale_t locale)`
    fn _mbtowc_l(dst: *mut u16, str_: *const u8, n: usize, locale: *mut c_void) -> i32;
    /// misc.c:401 — `unsigned int CDECL _get_output_format(void)`
    fn _get_output_format() -> u32;
    /// msvcrt's own `frexp` (math.c). `extern "C" -> f64` returns in x87 ST(0)
    /// on i386 — no SSE required (the fpnum.rs rule).
    fn frexp(x: f64, exp: *mut i32) -> f64;
}

extern "system" {
    // kernel32 — WINAPI (stdcall on i386), hence extern "system" (see lib.rs
    // GetStringTypeExA note for why "C" here would break ONLY i386, at link).
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(heap: *mut c_void, flags: u32, size: usize) -> *mut c_void;
    fn HeapFree(heap: *mut c_void, flags: u32, mem: *mut c_void) -> i32;
}

// ── f64 bit helpers (no-SSE discipline — see fpnum.rs `f64_is_inf`) ───────────

#[inline]
unsafe fn f64_bits(p: *const f64) -> u64 {
    *(p as *const u64)
}

#[inline]
unsafe fn f64_set_bits(p: *mut f64, bits: u64) {
    *(p as *mut u64) = bits;
}

/// `isinf(v)` on the bits.
#[inline]
unsafe fn f64_is_inf(p: *const f64) -> bool {
    (f64_bits(p) & 0x7FFF_FFFF_FFFF_FFFF) == 0x7FF0_0000_0000_0000
}

/// `isnan(v)` on the bits.
#[inline]
unsafe fn f64_is_nan(p: *const f64) -> bool {
    (f64_bits(p) & 0x7FFF_FFFF_FFFF_FFFF) > 0x7FF0_0000_0000_0000
}

// ── small C-isms ──────────────────────────────────────────────────────────────

/// C `strchr(set, c)` against a string-literal set, as the engine uses it.
/// **NOTE: `c == 0` matches the set's NUL TERMINATOR** (strchr returns non-NULL
/// for it), so 0 is "in" every set. pf_fill's `!strchr("diaAeEfFgG", Format)`
/// leans on that when Format is 0 — keep it.
fn in_set(set: &[u8], c: u32) -> bool {
    if c == 0 {
        return true;
    }
    let mut i = 0;
    while i < set.len() {
        if set[i] as u32 == c {
            return true;
        }
        i += 1;
    }
    false
}

unsafe fn c_strlen(s: *const u8) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

unsafe fn c_strnlen(s: *const u8, cap: usize) -> usize {
    let mut n = 0usize;
    while n < cap && *s.add(n) != 0 {
        n += 1;
    }
    n
}

unsafe fn c_wcslen(s: *const u16) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

unsafe fn c_wcsnlen(s: *const u16, cap: usize) -> usize {
    let mut n = 0usize;
    while n < cap && *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// printf.h:329 (and :452/:877/:926) —
/// `*(locale ? locale->locinfo : get_locinfo())->lconv->decimal_point`.
/// `crate::locinfo_of` is exactly the `locale ? locale->locinfo : get_locinfo()`
/// preamble (lib.rs), resolved through msvcrt-sys bindgen so the struct offsets
/// are compiler-checked, not hand-rolled.
unsafe fn decimal_point(locale: *mut c_void) -> u8 {
    *(*(*crate::locinfo_of(locale)).lconv).decimal_point as u8
}

// ── the APICHAR abstraction ───────────────────────────────────────────────────
//
// printf.h:21-29: APICHAR is `char` (narrow, `_a` names) or `wchar_t` (wide,
// `_w` names). One trait, two impls, monomorphized like the double-#include.

pub trait PfChar: Copy + Eq + 'static {
    /// printf.h:261 — `sizeof(APICHAR) == sizeof(wchar_t)`.
    const IS_WIDE: bool;
    /// An ASCII byte as this char type (every literal the engine stores is ASCII).
    fn of(b: u8) -> Self;
    /// The char's value, zero-extended (C's promotion of char/wchar_t to int —
    /// msvcrt's wchar_t is unsigned, and every byte the engine compares is ASCII).
    fn u(self) -> u32;
    /// A truncating store (C `APICHAR x = <int expr>`).
    fn from_u32(v: u32) -> Self;
}

impl PfChar for u8 {
    const IS_WIDE: bool = false;
    #[inline]
    fn of(b: u8) -> u8 {
        b
    }
    #[inline]
    fn u(self) -> u32 {
        self as u32
    }
    #[inline]
    fn from_u32(v: u32) -> u8 {
        v as u8
    }
}

impl PfChar for u16 {
    const IS_WIDE: bool = true;
    #[inline]
    fn of(b: u8) -> u16 {
        b as u16
    }
    #[inline]
    fn u(self) -> u32 {
        self as u32
    }
    #[inline]
    fn from_u32(v: u32) -> u16 {
        v as u16
    }
}

// ── the engine's public ABI types ─────────────────────────────────────────────

/// msvcrt.h:382-388 — `union _printf_arg`.
#[repr(C)]
#[derive(Clone, Copy)]
pub union PrintfArg {
    pub get_ptr: *mut c_void,
    pub get_int: i32,
    pub get_longlong: i64,
    pub get_double: f64,
}

/// msvcrt.h:380-381 — `typedef int (*puts_clbk_a)(void*, int, const char*)` and
/// the `_w` twin, generically.
pub type PutsClbk<C> = unsafe extern "C" fn(*mut c_void, i32, *const C) -> i32;

/// msvcrt.h:389 — `typedef printf_arg (*args_clbk)(void*, int, int, va_list*)`.
/// The `va_list*` is a pointer to the flat byte cursor (see module docs).
pub type ArgsClbk = unsafe extern "C" fn(*mut c_void, i32, i32, *mut *mut u8) -> PrintfArg;

/// wcs.c:40-47 — `pf_flags`. Field names kept 1:1 (snake_cased); `Sign` and
/// `Format` are `char` in the C **even for the wide engine** (so a wide format
/// char TRUNCATES on assignment — see `pf_printf`), hence u8 here.
#[derive(Clone, Copy, Default)]
pub struct PfFlags {
    pub integer_length: i32, // LEN_DEFAULT / LEN_SHORT / LEN_LONG
    pub integer_double: bool,
    pub integer_native: bool,
    pub left_align: bool,
    pub alternate: bool,
    pub pad_zero: bool,
    pub wide_string: bool,
    pub natural_string: bool,
    pub field_length: i32,
    pub precision: i32,
    pub sign: u8,
    pub format: u8,
}

// ── va_list as a flat byte cursor (streamout.rs:96-114, mirrored exactly) ─────

#[cfg(target_arch = "x86")]
const VA_SLOT: usize = 4;
#[cfg(not(target_arch = "x86"))]
const VA_SLOT: usize = 8;

#[inline]
fn slot_of(sz: usize) -> usize {
    (sz + VA_SLOT - 1) / VA_SLOT * VA_SLOT
}

/// One `va_arg(*cursor, T)`: unaligned little-endian read at the slot start,
/// then advance by the rounded slot (covers 8-byte args on 4-byte i386 slots).
///
/// `pub(crate)` because `scanf.rs` needs the SAME cursor arithmetic — two copies of the
/// slot rule is exactly the drift that makes a va_list read one slot off forever.
#[inline]
pub(crate) unsafe fn va_next<T: Copy>(cursor: &mut *mut u8) -> T {
    let v = core::ptr::read_unaligned(*cursor as *const T);
    *cursor = (*cursor).add(slot_of(core::mem::size_of::<T>()));
    v
}

// ── the bounded string sink (printf.h:31-54) ──────────────────────────────────

/// printf.h:31-34 — `struct _str_ctx_a` / `_str_ctx_w`. #[repr(C)] because the
/// (future Rust) shells build it on the stack exactly as wcs.c does.
#[repr(C)]
pub struct PfStrCtx<C> {
    pub len: usize,
    pub buf: *mut C,
}

pub type StrCtxA = PfStrCtx<u8>;
pub type StrCtxW = PfStrCtx<u16>;

/// printf.h:36-54 — `puts_clbk_str_a` / `_w`. The classic (pre-c99) sink:
/// counts when `buf` is NULL, truncates-and-returns-`-1` when full.
pub unsafe extern "C" fn puts_clbk_str<C: PfChar>(ctx: *mut c_void, len: i32, str_: *const C) -> i32 {
    let out = ctx as *mut PfStrCtx<C>;

    if (*out).buf.is_null() {
        return len;
    }

    // printf.h:43 — `out->len < len` compares size_t against int: len is
    // PROMOTED TO UNSIGNED, so a negative len takes the truncation branch.
    // `len as usize` sign-extends identically. (Never hit with a negative len
    // by this engine; kept for the shells.)
    if (*out).len < len as usize {
        core::ptr::copy(str_, (*out).buf, (*out).len); // memmove
        (*out).buf = (*out).buf.add((*out).len);
        (*out).len = 0;
        return -1;
    }

    core::ptr::copy(str_, (*out).buf, len as usize); // memmove
    (*out).buf = (*out).buf.add(len as usize);
    (*out).len -= len as usize;
    len
}

/// Concrete instantiations, named as the C names them.
pub unsafe extern "C" fn puts_clbk_str_a(ctx: *mut c_void, len: i32, str_: *const u8) -> i32 {
    puts_clbk_str::<u8>(ctx, len, str_)
}
pub unsafe extern "C" fn puts_clbk_str_w(ctx: *mut c_void, len: i32, str_: *const u16) -> i32 {
    puts_clbk_str::<u16>(ctx, len, str_)
}

// ── parsing / padding primitives ──────────────────────────────────────────────

/// printf.h:56-66 — `pf_parse_int`. Returns the advanced cursor; `*val` gets the
/// (wrapping — C int overflow) decimal value.
unsafe fn pf_parse_int<C: PfChar>(mut fmt: *const C, val: &mut i32) -> *const C {
    *val = 0;

    while (*fmt).u() >= b'0' as u32 && (*fmt).u() <= b'9' as u32 {
        *val = val.wrapping_mul(10).wrapping_add(((*fmt).u() - b'0' as u32) as i32);
        fmt = fmt.add(1);
    }

    fmt
}

/// printf.h:68-107 — `pf_fill`: signs, alignment, zero and field padding.
/// `left == true` is the BEFORE-the-payload call, `false` the after.
unsafe fn pf_fill<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    len: i32,
    flags: &mut PfFlags,
    left: bool,
) -> i32 {
    let mut r: i32 = 0;

    // printf.h:74 — note strchr semantics: Format==0 stays "in" the set.
    if flags.sign != 0 && !in_set(b"diaAeEfFgG", flags.format as u32) {
        flags.sign = 0;
    }

    if left && flags.sign != 0 {
        let ch = C::of(flags.sign);
        flags.field_length -= 1;
        if flags.pad_zero {
            r = pf_puts(puts_ctx, 1, &ch);
        }
    }
    let mut written = r;

    if (!left && flags.left_align) || (left && !flags.left_align) {
        let ch = if left && flags.pad_zero { C::of(b'0') } else { C::of(b' ') };

        let mut i = 0;
        while i < flags.field_length - len && r >= 0 {
            r = pf_puts(puts_ctx, 1, &ch);
            written += r;
            i += 1;
        }
    }

    if r >= 0 && left && flags.sign != 0 && !flags.pad_zero {
        let ch = C::of(flags.sign);
        r = pf_puts(puts_ctx, 1, &ch);
        written += r;
    }

    if r >= 0 {
        written
    } else {
        r
    }
}

// ── PRINTF_HELPERS (printf.h:109-152) — shared, instantiated once in the C ────

/// printf.h:111-123 — `wcstombs_len`. NULL `mbstr` = measuring pass.
unsafe fn wcstombs_len(mbstr: *mut u8, wcstr: *const u16, len: i32, locale: *mut c_void) -> i32 {
    let mut buf = [0u8; MB_LEN_MAX];
    let mut mblen: i32 = 0;

    let mut i: i32 = 0;
    while i < len {
        let dst = if !mbstr.is_null() { mbstr.offset(mblen as isize) } else { buf.as_mut_ptr() };
        let r = _wctomb_l(dst, *wcstr.offset(i as isize), locale);
        if r < 0 {
            return r;
        }
        mblen += r;
        i += 1;
    }
    mblen
}

/// printf.h:125-137 — `mbstowcs_len`. NULL `wcstr` = measuring pass. Note the C
/// advances by `r ? r : 1` (a 0-byte consume — embedded NUL inside the counted
/// range — still moves one byte).
unsafe fn mbstowcs_len(wcstr: *mut u16, mbstr: *const u8, len: i32, locale: *mut c_void) -> i32 {
    let mut buf: u16 = 0;
    let mut wlen: i32 = 0;

    let mut i: i32 = 0;
    while i < len {
        let dst = if !wcstr.is_null() { wcstr.offset(wlen as isize) } else { &mut buf as *mut u16 };
        let r = _mbtowc_l(dst, mbstr.offset(i as isize), (len - i) as usize, locale);
        if r < 0 {
            return r;
        }
        i += if r != 0 { r } else { 1 };
        wlen += 1;
    }
    wlen
}

// (printf.h:139-150 log2i/log10i live in crate::bnum — one implementation for
// parser and formatter, exactly as bnum.h is #include'd by both C files.)

// ── string output with cross-width conversion (printf.h:154-256) ──────────────

/// printf.h:154-174 — `pf_output_wstr`: a WIDE source into the APICHAR stream.
unsafe fn pf_output_wstr<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const u16,
    len: i32,
    locale: *mut c_void,
) -> i32 {
    if C::IS_WIDE {
        return pf_puts(puts_ctx, len, str_ as *const C);
    }

    // narrow instance (printf.h:160-172): wide→multibyte via a HeapAlloc'd temp.
    let len_a = wcstombs_len(core::ptr::null_mut(), str_, len, locale);
    if len_a < 0 {
        return -1;
    }

    let out = HeapAlloc(GetProcessHeap(), 0, len_a as usize) as *mut u8;
    if out.is_null() {
        return -1;
    }

    wcstombs_len(out, str_, len, locale);
    let len2 = pf_puts(puts_ctx, len_a, out as *const C);
    HeapFree(GetProcessHeap(), 0, out as *mut c_void);
    len2
}

/// printf.h:176-196 — `pf_output_str`: a NARROW source into the APICHAR stream.
unsafe fn pf_output_str<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const u8,
    len: i32,
    locale: *mut c_void,
) -> i32 {
    if C::IS_WIDE {
        // wide instance (printf.h:180-192): multibyte→wide via a HeapAlloc'd temp.
        let len_w = mbstowcs_len(core::ptr::null_mut(), str_, len, locale);
        if len_w < 0 {
            return -1;
        }

        let out = HeapAlloc(GetProcessHeap(), 0, (len_w as usize) * 2) as *mut u16;
        if out.is_null() {
            return -1;
        }

        mbstowcs_len(out, str_, len, locale);
        let len2 = pf_puts(puts_ctx, len_w, out as *const C);
        HeapFree(GetProcessHeap(), 0, out as *mut c_void);
        return len2;
    }

    pf_puts(puts_ctx, len, str_ as *const C)
}

/// printf.h:198-226 — `pf_output_format_wstr`: fill + wide payload + fill.
unsafe fn pf_output_format_wstr<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const u16,
    mut len: i32,
    flags: &mut PfFlags,
    locale: *mut c_void,
) -> i32 {
    if len < 0 {
        // Do not search past the length specified by the precision. (printf.h:204)
        if flags.precision >= 0 {
            len = c_wcsnlen(str_, flags.precision as usize) as i32;
        } else {
            len = c_wcslen(str_) as i32;
        }
    }

    if flags.precision >= 0 && flags.precision < len {
        len = flags.precision;
    }

    let mut r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, true);
    let mut ret = r;
    if r >= 0 {
        r = pf_output_wstr::<C>(pf_puts, puts_ctx, str_, len, locale);
        ret += r;
    }
    if r >= 0 {
        r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, false);
        ret += r;
    }

    if r >= 0 {
        ret
    } else {
        r
    }
}

/// printf.h:228-256 — `pf_output_format_str`: fill + narrow payload + fill.
unsafe fn pf_output_format_str<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const u8,
    mut len: i32,
    flags: &mut PfFlags,
    locale: *mut c_void,
) -> i32 {
    if len < 0 {
        // Do not search past the length specified by the precision. (printf.h:234)
        if flags.precision >= 0 {
            len = c_strnlen(str_, flags.precision as usize) as i32;
        } else {
            len = c_strlen(str_) as i32;
        }
    }

    if flags.precision >= 0 && flags.precision < len {
        len = flags.precision;
    }

    let mut r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, true);
    let mut ret = r;
    if r >= 0 {
        r = pf_output_str::<C>(pf_puts, puts_ctx, str_, len, locale);
        ret += r;
    }
    if r >= 0 {
        r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, false);
        ret += r;
    }

    if r >= 0 {
        ret
    } else {
        r
    }
}

/// The `#ifdef PRINTF_WIDE`-dispatch the C repeats at printf.h:389-393, 401-405,
/// 520-524, 531-535, 1159-1163 and 1204-1208: an APICHAR buffer goes through the
/// SAME-width format_str path.
#[inline]
unsafe fn pf_output_format_apichar<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const C,
    len: i32,
    flags: &mut PfFlags,
    locale: *mut c_void,
) -> i32 {
    if C::IS_WIDE {
        pf_output_format_wstr::<C>(pf_puts, puts_ctx, str_ as *const u16, len, flags, locale)
    } else {
        pf_output_format_str::<C>(pf_puts, puts_ctx, str_ as *const u8, len, flags, locale)
    }
}

// ── %s / %S / %c / %C routing (printf.h:258-281) ──────────────────────────────

/// printf.h:258-281 — `pf_handle_string`: decides whether the payload pointer is
/// narrow or wide, per modifier flags + the legacy S/C complement rule.
/// `str_` is `const void*` exactly as in the C (for %c it points at an `int`).
unsafe fn pf_handle_string<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    str_: *const c_void,
    len: i32,
    flags: &mut PfFlags,
    locale: *mut c_void,
    legacy_wide: bool,
) -> i32 {
    let api_is_wide = C::IS_WIDE; // printf.h:261 sizeof(APICHAR)==sizeof(wchar_t)
    let complement_is_narrow = if legacy_wide { api_is_wide } else { false }; // printf.h:262

    if str_.is_null() {
        // printf.h:265-269 — "(null)", in the ENGINE's width.
        return if C::IS_WIDE {
            static NULL_W: [u16; 6] =
                [b'(' as u16, b'n' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b')' as u16];
            pf_output_format_wstr::<C>(pf_puts, puts_ctx, NULL_W.as_ptr(), 6, flags, locale)
        } else {
            pf_output_format_str::<C>(pf_puts, puts_ctx, b"(null)".as_ptr(), 6, flags, locale)
        };
    }

    // printf.h:272-275
    if (flags.natural_string && api_is_wide) || flags.wide_string || flags.integer_length == LEN_LONG {
        return pf_output_format_wstr::<C>(pf_puts, puts_ctx, str_ as *const u16, len, flags, locale);
    }
    if (flags.natural_string && !api_is_wide) || flags.integer_length == LEN_SHORT {
        return pf_output_format_str::<C>(pf_puts, puts_ctx, str_ as *const u8, len, flags, locale);
    }

    // printf.h:277-280 — the BOOL==BOOL complement rule.
    if ((flags.format == b'S') || (flags.format == b'C')) == complement_is_narrow {
        pf_output_format_str::<C>(pf_puts, puts_ctx, str_ as *const u8, len, flags, locale)
    } else {
        pf_output_format_wstr::<C>(pf_puts, puts_ctx, str_ as *const u16, len, flags, locale)
    }
}

// ── inf/nan output (printf.h:283-410) ─────────────────────────────────────────

/// printf.h:283-410 — `pf_output_special_fp`. `v` arrives NON-NEGATIVE (the
/// dispatcher moved the sign into `flags.sign`); the else-of-isinf is the NaN
/// case, with `sign=='-'` meaning IND.
unsafe extern "C" fn pf_output_special_fp<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    v: f64,
    flags: &mut PfFlags,
    locale: *mut c_void,
    legacy_msvcrt_compat: bool,
    three_digit_exp: bool,
) -> i32 {
    let mut pfx = [C::of(0); 16]; // printf.h:287 APICHAR pfx[16]
    let mut sfx = [C::of(0); 8]; //             …, sfx[8]
    let mut len: i32 = 0;

    if !legacy_msvcrt_compat {
        // printf.h:290-304 — the ucrt spelling ("inf", "nan(ind)", …), narrow
        // literals through the cross-width path even in the wide engine.
        let s: &[u8] = if f64_is_inf(&v) {
            if in_set(b"AEFG", flags.format as u32) {
                b"INF\0"
            } else {
                b"inf\0"
            }
        } else if in_set(b"AEFG", flags.format as u32) {
            if flags.sign == b'-' {
                b"NAN(IND)\0"
            } else {
                b"NAN\0"
            }
        } else if flags.sign == b'-' {
            b"nan(ind)\0"
        } else {
            b"nan\0"
        };

        flags.precision = -1;
        flags.pad_zero = false;
        return pf_output_format_str::<C>(pf_puts, puts_ctx, s.as_ptr(), -1, flags, locale);
    }

    // workaround a bug in native implementation (printf.h:306-308)
    if flags.format == b'g' || flags.format == b'G' {
        flags.precision -= 1;
    }

    let pfx0 = pfx.as_mut_ptr();
    let mut p = pfx0;
    if flags.pad_zero && (flags.format == b'a' || flags.format == b'A') {
        // printf.h:311-320
        if flags.sign != 0 {
            *p = C::of(flags.sign);
            p = p.add(1);
        }
        *p = C::of(b'0');
        p = p.add(1);
        *p = C::of(if flags.format == b'a' { b'x' } else { b'X' });
        p = p.add(1);
        let r = pf_puts(puts_ctx, p.offset_from(pfx0) as i32, pfx0);
        if r < 0 {
            return r;
        }
        len += r;

        flags.field_length -= p.offset_from(pfx0) as i32;
    }

    let mut p = pfx0;
    if !flags.pad_zero && (flags.format == b'a' || flags.format == b'A') {
        // printf.h:322-326
        *p = C::of(b'0');
        p = p.add(1);
        *p = C::of(if flags.format == b'a' { b'x' } else { b'X' });
        p = p.add(1);
    }

    // printf.h:328-351 — "1.#INF" / "1.#IND" / "1.#QNAN"
    *p = C::of(b'1');
    p = p.add(1);
    *p = C::of(decimal_point(locale));
    p = p.add(1);
    *p = C::of(b'#');
    p = p.add(1);
    let mut frac_len: i32 = 1;

    if f64_is_inf(&v) {
        *p = C::of(b'I');
        p = p.add(1);
        *p = C::of(b'N');
        p = p.add(1);
        *p = C::of(b'F');
        p = p.add(1);
        frac_len += 3;
    } else if flags.sign == b'-' {
        *p = C::of(b'I');
        p = p.add(1);
        *p = C::of(b'N');
        p = p.add(1);
        *p = C::of(b'D');
        p = p.add(1);
        frac_len += 3;
    } else {
        *p = C::of(b'Q');
        p = p.add(1);
        *p = C::of(b'N');
        p = p.add(1);
        *p = C::of(b'A');
        p = p.add(1);
        *p = C::of(b'N');
        p = p.add(1);
        frac_len += 4;
    }
    *p = C::of(0);
    let pfx_len = p.offset_from(pfx0) as i32;

    if len != 0 {
        flags.sign = 0; // printf.h:353
    }

    // printf.h:355-356 — a truncating precision ROUNDS by bumping the char just
    // before the cut point (negative index back from the NUL); precision 0 bumps
    // the leading '1' itself. Native-compat oddity, kept.
    if flags.precision >= 0 && flags.precision < frac_len {
        let q = p.offset((flags.precision - frac_len - 1) as isize);
        *q = C::from_u32((*q).u().wrapping_add(1));
    }

    // printf.h:358-373 — the exponent suffix ("p+0", "e+000", …).
    let sfx0 = sfx.as_mut_ptr();
    let mut p = sfx0;
    if in_set(b"aAeE", flags.format as u32) {
        if flags.format == b'a' {
            *p = C::of(b'p');
        } else if flags.format == b'A' {
            *p = C::of(b'P');
        } else if flags.format == b'e' {
            *p = C::of(b'e');
        } else {
            *p = C::of(b'E');
        }
        p = p.add(1);

        *p = C::of(b'+');
        p = p.add(1);
        *p = C::of(b'0');
        p = p.add(1);

        if flags.format == b'e' || flags.format == b'E' {
            *p = C::of(b'0');
            p = p.add(1);
            if three_digit_exp {
                *p = C::of(b'0');
                p = p.add(1);
            }
        }
    }
    *p = C::of(0);

    // printf.h:375-387
    let mut sfx_len: i32 = if !flags.alternate && (flags.format == b'g' || flags.format == b'G') {
        frac_len
    } else {
        flags.precision
    };

    if sfx_len == -1 {
        if in_set(b"fFeE", flags.format as u32) {
            sfx_len = 6;
        } else if flags.format == b'a' || flags.format == b'A' {
            sfx_len = 13;
        }
    }
    sfx_len += p.offset_from(sfx0) as i32 - frac_len;

    if sfx_len > 0 {
        flags.field_length -= sfx_len;
    }
    if flags.precision >= 0 {
        if flags.precision == 0 {
            flags.precision -= 1;
        }
        flags.precision += pfx_len - frac_len;
    }

    // printf.h:389-395
    let r = pf_output_format_apichar::<C>(pf_puts, puts_ctx, pfx0, -1, flags, locale);
    if r < 0 {
        return r;
    }
    len += r;

    // printf.h:397-407
    flags.field_length = sfx_len;
    flags.pad_zero = true;
    flags.precision = -1;
    flags.sign = 0;
    let r = pf_output_format_apichar::<C>(pf_puts, puts_ctx, sfx0, -1, flags, locale);
    if r < 0 {
        return r;
    }
    len += r;

    len
}

// ── %a / %A hex float (printf.h:412-540) ──────────────────────────────────────

/// printf.h:412-540 — `pf_output_hex_fp`. Pure bit manipulation on the IEEE
/// double; no float arithmetic at all (matching the C's `*(ULONGLONG*)&v`).
unsafe extern "C" fn pf_output_hex_fp<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    v: f64,
    flags: &mut PfFlags,
    locale: *mut c_void,
    standard_rounding: bool,
) -> i32 {
    // printf.h:415-418 — digits[2][16], selected by Format=='A'.
    const DIG_L: &[u8; 16] = b"0123456789abcdef";
    const DIG_U: &[u8; 16] = b"0123456789ABCDEF";

    let mut pfx = [C::of(0); 4 + (MANT_BITS as usize) / 4 + 1]; // printf.h:420 (18)
    let mut sfx = [C::of(0); 8];
    let mut len: i32 = 0;
    let mut sfx_len: i32 = 0;

    // printf.h:424-427 — strip the sign bit, extract biased exponent + mantissa.
    let mut mant: u64 = f64_bits(&v) << 1;
    let mut exp: i32 = (mant >> MANT_BITS) as i32;
    exp -= (1 << (EXP_BITS - 1)) - 1;
    mant = (mant << EXP_BITS) >> (EXP_BITS + 1);

    let pfx0 = pfx.as_mut_ptr();
    let mut p = pfx0;
    if flags.pad_zero {
        // printf.h:430-440 — sign+0x go out immediately so zero-padding lands
        // between "0x" and the digits.
        if flags.sign != 0 {
            *p = C::of(flags.sign);
            p = p.add(1);
        }
        *p = C::of(b'0');
        p = p.add(1);
        *p = C::of(if flags.format == b'a' { b'x' } else { b'X' });
        p = p.add(1);
        let r = pf_puts(puts_ctx, p.offset_from(pfx0) as i32, pfx0);
        if r < 0 {
            return r;
        }
        len += r;

        flags.field_length -= p.offset_from(pfx0) as i32;
        flags.sign = 0;
        p = pfx0;
    } else {
        // printf.h:441-444
        *p = C::of(b'0');
        p = p.add(1);
        *p = C::of(if flags.format == b'a' { b'x' } else { b'X' });
        p = p.add(1);
    }

    // printf.h:445-451 — subnormal (or zero) leads with '0', normal with '1'.
    if exp == -(1 << (EXP_BITS - 1)) + 1 {
        if mant == 0 {
            exp = 0;
        } else {
            exp += 1;
        }
        *p = C::of(b'0');
        p = p.add(1);
    } else {
        *p = C::of(b'1');
        p = p.add(1);
    }
    *p = C::of(decimal_point(locale));
    p = p.add(1);

    // printf.h:453-456 — 13 hex digits, least-significant nibble first from the
    // bottom of `mant`, written back-to-front. `p` is NOT advanced.
    let digits: &[u8; 16] = if flags.format == b'A' { DIG_U } else { DIG_L };
    let mut r_i: i32 = MANT_BITS / 4 - 1;
    while r_i >= 0 {
        *p.offset(r_i as isize) = C::of(digits[(mant & 15) as usize]);
        mant >>= 4;
        r_i -= 1;
    }

    if flags.precision == 0 {
        // printf.h:457-459 — round-to-integer on the first digit; without '#'
        // the decimal point itself is dropped (p steps back onto it).
        if (*p).u() >= b'8' as u32 {
            let q = p.offset(-2);
            *q = C::from_u32((*q).u().wrapping_add(1));
        }
        if !flags.alternate {
            p = p.offset(-1);
        }
    } else if flags.precision > 0 && flags.precision < MANT_BITS / 4 {
        // printf.h:460-492 — round at `precision` digits.
        let mut round_up = false;

        let cut = (*p.offset(flags.precision as isize)).u();
        if !standard_rounding {
            round_up = cut >= b'8' as u32;
        } else if cut > b'8' as u32 {
            round_up = true;
        } else if cut == b'8' as u32 {
            // sticky scan of the dropped tail (printf.h:466-471)
            let mut r2 = flags.precision + 1;
            while r2 < MANT_BITS / 4 {
                if (*p.offset(r2 as isize)).u() != b'0' as u32 {
                    round_up = true;
                    break;
                }
                r2 += 1;
            }

            if !round_up {
                // ties-to-even on the LAST KEPT digit (printf.h:473-477); note
                // the parity test differs per digit alphabet.
                let pm1 = (*p.offset((flags.precision - 1) as isize)).u();
                if pm1 <= b'9' as u32 {
                    round_up = (pm1 - b'0' as u32) & 1 != 0;
                } else if pm1 <= b'F' as u32 {
                    round_up = (pm1 - b'A' as u32) & 1 != 0;
                } else {
                    round_up = (pm1 - b'a' as u32) & 1 != 0;
                }
            }
        }

        // printf.h:480-490 — carry cascade. NOTE the '9'→'a'/'A' hop (hex!) and
        // that 'f'/'F' wraps to '0' with carry.
        let mut r2 = flags.precision - 1;
        while r2 >= 0 && round_up {
            round_up = false;
            let ch = (*p.offset(r2 as isize)).u();
            if ch == b'f' as u32 || ch == b'F' as u32 {
                *p.offset(r2 as isize) = C::of(b'0');
                round_up = true;
            } else if ch == b'9' as u32 {
                *p.offset(r2 as isize) = C::of(if flags.format == b'a' { b'a' } else { b'A' });
            } else {
                *p.offset(r2 as isize) = C::from_u32(ch + 1);
            }
            r2 -= 1;
        }
        if round_up {
            // carry into the integer digit (p[-1] is the decimal point)
            let q = p.offset(-2);
            *q = C::from_u32((*q).u().wrapping_add(1));
        }
        p = p.offset(flags.precision as isize);
    } else {
        // printf.h:493-496 — keep all digits; excess precision becomes suffix pad.
        p = p.offset((MANT_BITS / 4) as isize);
        if flags.precision > MANT_BITS / 4 {
            sfx_len += flags.precision - MANT_BITS / 4;
        }
    }
    *p = C::of(0);

    // printf.h:499-516 — "p±ddd" suffix. The C writes up to 4 exponent digits
    // right-justified at p[0..3] then compacts left, REUSING `exp` as the copy
    // counter. (If the exponent needed >4 digits the copy would read p[-1] —
    // unreachable for binary64, transcribed as-is.)
    let sfx0 = sfx.as_mut_ptr();
    let mut p = sfx0;
    *p = C::of(if flags.format == b'a' { b'p' } else { b'P' });
    p = p.add(1);
    if exp < 0 {
        *p = C::of(b'-');
        p = p.add(1);
        exp = -exp;
    } else {
        *p = C::of(b'+');
        p = p.add(1);
    }
    let mut r_i: i32 = 3;
    while r_i >= 0 {
        *p.offset(r_i as isize) = C::of((exp % 10) as u8 + b'0');
        exp /= 10;
        if exp == 0 {
            break;
        }
        r_i -= 1;
    }
    // for(exp=0; exp<4-r; exp++) p[exp] = p[exp+r];
    let mut k: i32 = 0;
    while k < 4 - r_i {
        *p.offset(k as isize) = *p.offset((k + r_i) as isize);
        k += 1;
    }
    p = p.offset(k as isize);
    *p = C::of(0);
    sfx_len += p.offset_from(sfx0) as i32;

    // printf.h:518-537
    flags.field_length -= sfx_len;
    flags.precision = -1;
    let r = pf_output_format_apichar::<C>(pf_puts, puts_ctx, pfx0, -1, flags, locale);
    if r < 0 {
        return r;
    }
    len += r;

    flags.field_length = sfx_len;
    flags.pad_zero = true;
    flags.sign = 0;
    let r = pf_output_format_apichar::<C>(pf_puts, puts_ctx, sfx0, -1, flags, locale);
    if r < 0 {
        return r;
    }
    len += r;

    len
}

// ── integer conversion (printf.h:542-601) ─────────────────────────────────────

/// printf.h:542-601 — `pf_integer_conv`: prints `x` to `buf` (digits + precision
/// zero-pad + alternate prefix, NO sign/field chars), then reverses in place.
/// SIDE EFFECTS the caller depends on: `flags.precision` is REWRITTEN to the
/// emitted length; `flags.sign` set to '-' for negative d/i; `flags.alternate`
/// cleared when x == 0.
unsafe fn pf_integer_conv<C: PfChar>(buf: *mut C, flags: &mut PfFlags, mut x: i64) {
    let base: u32 = if flags.format == b'o' {
        8
    } else if flags.format == b'x' || flags.format == b'X' {
        16
    } else {
        10
    };

    let digits: &[u8; 17] = if flags.format == b'X' {
        b"0123456789ABCDEFX"
    } else {
        b"0123456789abcdefx"
    };

    if x < 0 && (flags.format == b'd' || flags.format == b'i') {
        x = x.wrapping_neg(); // C `x = -x`: i64::MIN wraps to itself (UB-in-C that
                              // in practice produces these exact bits; the
                              // (ULONGLONG) casts below then divide unsigned)
        flags.sign = b'-';
    }

    let mut i: i32 = 0;
    if x == 0 {
        flags.alternate = false;
        if flags.precision != 0 {
            *buf.offset(i as isize) = C::of(b'0');
            i += 1;
        }
    } else {
        while x != 0 {
            // printf.h:574-575 — unsigned modulo/divide of the SIGNED value.
            let j = ((x as u64) % base as u64) as usize;
            x = ((x as u64) / base as u64) as i64;
            *buf.offset(i as isize) = C::of(digits[j]);
            i += 1;
        }
    }
    // printf.h:579-581 — precision zero-pad (k may start negative: no pad).
    let mut k = flags.precision - i;
    while k > 0 {
        *buf.offset(i as isize) = C::of(b'0');
        i += 1;
        k -= 1;
    }
    if flags.alternate {
        if base == 16 {
            // reversed later: emits "0x"/"0X" prefix
            *buf.offset(i as isize) = C::of(digits[16]);
            i += 1;
            *buf.offset(i as isize) = C::of(b'0');
            i += 1;
        } else if base == 8 && (*buf.offset((i - 1) as isize)).u() != b'0' as u32 {
            *buf.offset(i as isize) = C::of(b'0');
            i += 1;
        }
    }

    // Adjust precision so pf_fill won't truncate the number later (printf.h:590-591)
    flags.precision = i;

    *buf.offset(i as isize) = C::of(0);
    // printf.h:594-600 — in-place reversal: `while(--i > j)`.
    let mut j: i32 = 0;
    loop {
        i -= 1;
        if i <= j {
            break;
        }
        let tmp = *buf.offset(j as isize);
        *buf.offset(j as isize) = *buf.offset(i as isize);
        *buf.offset(i as isize) = tmp;
        j += 1;
    }
}

// ── %e/%E/%f/%F/%g/%G — the bignum float formatter (printf.h:603-982) ─────────

/// printf.h:603-982 — `pf_output_fp`. Converts the (non-negative, finite) double
/// into base-10 limbs with `crate::bnum` (the SAME engine `strtod` parses with —
/// bit-agreement between the two is the whole reason bnum.rs is shared), rounds
/// at the requested position, then streams digits limb-by-limb through
/// `pf_integer_conv`.
// allow(unused_assignments): the carry-overflow fixups store round_pos/round_limb
// (printf.h:726-731, 738-743) that nothing reads afterwards — the C does the
// same dead store, and we transcribe rather than optimize it away.
#[allow(unused_assignments)]
unsafe extern "C" fn pf_output_fp<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    v: f64,
    flags: &mut PfFlags,
    locale: *mut c_void,
    three_digit_exp: bool,
    standard_rounding: bool,
) -> i32 {
    let mut e2: i32 = 0;
    let mut e10: i32 = 0;
    // printf.h:609-613 — the C stack-allocates FIELD_OFFSET(struct bnum,
    // data[BNUM_PREC64]) bytes; our Bnum borrows the limb buffer directly.
    let mut bnum_data = [0u32; BNUM_PREC64];
    let mut b = Bnum { b: 0, e: 1, size: BNUM_PREC64 as i32, data: &mut bnum_data };
    let mut buf = [C::of(0); LIMB_DIGITS as usize + 1]; // printf.h:614
    let mut trim_tail = false;
    let mut round_up = false;
    // printf.h:616 — `pf_flags f;` is UNINITIALIZED in the C; only Format/PadZero
    // (set below) and Precision (set before every use) are read, and
    // pf_integer_conv's read of f.Alternate is inert under Format='d' (base 10 —
    // the alternate branch tests base 16/8 only). Zero-init is the safe
    // equivalent of that garbage.
    let mut f = PfFlags::default();

    if flags.precision == -1 {
        flags.precision = 6; // printf.h:621-622 (NOTE: a NEGATIVE precision from
                             // "%.*f" that isn't exactly -1 is NOT normalized —
                             // C quirk, kept by not clamping here)
    }

    // printf.h:624-651 — seed the bignum from frexp's mantissa bits.
    let mut v = v;
    v = frexp(v, &mut e2);
    if f64_bits(&v) << 1 != 0 {
        // C: `if(v)` — any nonzero double (sign bit ignored; v >= 0 here anyway)
        let mut m: u64 = 1u64 << (MANT_BITS - 1);
        m |= f64_bits(&v) & ((1u64 << (MANT_BITS - 1)) - 1);
        b.b = 0;
        b.e = 2;
        b.set(0, (m % LIMB_MAX as u64) as u32);
        b.set(1, (m / LIMB_MAX as u64) as u32);
        e2 -= MANT_BITS;

        while e2 > 0 {
            let shift = if e2 > 29 { 29 } else { e2 };
            if b.lshift(shift) {
                e10 += LIMB_DIGITS;
            }
            e2 -= shift;
        }
        while e2 < 0 {
            let shift = if -e2 > 9 { 9 } else { -e2 };
            if b.rshift(shift) {
                e10 -= LIMB_DIGITS;
            }
            e2 += shift;
        }
    } else {
        b.b = 0;
        b.e = 1;
        b.set(0, 0);
        e10 = -LIMB_DIGITS;
    }

    // printf.h:653-657
    let mut first_limb_len: i32 = if b.at(b.e - 1) == 0 {
        1
    } else {
        log10i(b.at(b.e - 1)) as i32 + 1
    };
    let mut radix_pos: i32 = first_limb_len + LIMB_DIGITS + e10;

    // printf.h:659-667 — where (limb + digit-in-limb) rounding happens.
    let mut round_pos: i32 = flags.precision;
    if flags.format == b'f' || flags.format == b'F' {
        round_pos += radix_pos;
    } else if flags.precision == 0 || flags.format == b'e' || flags.format == b'E' {
        round_pos += 1;
    }
    let mut round_limb: i32 = if round_pos <= first_limb_len {
        b.e + (first_limb_len - round_pos) / LIMB_DIGITS - 1
    } else {
        b.e - (round_pos - first_limb_len - 1) / LIMB_DIGITS - 2
    };

    if b.b <= round_limb && round_limb < b.e {
        // printf.h:669-746 — the round position falls INSIDE the stored limbs.
        if round_pos <= first_limb_len {
            round_pos = first_limb_len - round_pos; // printf.h:671
        } else {
            round_pos = LIMB_DIGITS - (round_pos - first_limb_len) % LIMB_DIGITS; // printf.h:673
            if round_pos == LIMB_DIGITS {
                round_pos = 0;
            }
        }

        if round_pos != 0 {
            // printf.h:677-691 — cut inside a limb.
            let l = b.at(round_limb) % P10S[round_pos as usize];
            b.set(round_limb, b.at(round_limb) - l);
            if !standard_rounding {
                round_up = 2 * l >= P10S[round_pos as usize];
            } else if 2 * l > P10S[round_pos as usize] {
                round_up = true;
            } else if 2 * l == P10S[round_pos as usize] {
                let mut r = round_limb - 1;
                while r >= b.b {
                    if b.at(r) != 0 {
                        round_up = true;
                        break;
                    }
                    r -= 1;
                }

                if !round_up {
                    round_up = (b.at(round_limb) / P10S[round_pos as usize]) & 1 != 0;
                }
            }
        } else if round_limb - 1 >= b.b {
            // printf.h:692-705 — cut exactly at a limb boundary.
            if !standard_rounding {
                round_up = 2 * b.at(round_limb - 1) >= LIMB_MAX;
            } else if 2 * b.at(round_limb - 1) > LIMB_MAX {
                round_up = true;
            } else if 2 * b.at(round_limb - 1) == LIMB_MAX {
                let mut r = round_limb - 2;
                while r >= b.b {
                    if b.at(r) != 0 {
                        round_up = true;
                        break;
                    }
                    r -= 1;
                }

                if !round_up {
                    round_up = b.at(round_limb) & 1 != 0;
                }
            }
        }
        b.b = round_limb; // printf.h:706 — drop everything below the round limb

        if round_up {
            // printf.h:708-745 — add the round increment + carry cascade.
            b.set(b.b, b.at(b.b) + P10S[round_pos as usize]);
            let mut i = b.b;
            while i < b.e {
                if b.at(i) < LIMB_MAX {
                    break;
                }
                b.set(i, b.at(i) - LIMB_MAX);
                if i + 1 < b.e {
                    b.set(i + 1, b.at(i + 1) + 1);
                } else {
                    b.set(i + 1, 1); // writes one past e; e bumped in the i==b.e arm
                }
                i += 1;
            }
            if i == b.e - 1 {
                // printf.h:717-732 — top limb may have grown a digit. (The C
                // reuses `i` for the recount; renamed `n` — the else-if below
                // tests the ORIGINAL loop i, which an if/else chain preserves.)
                let n = if b.at(b.e - 1) == 0 {
                    1
                } else {
                    log10i(b.at(b.e - 1)) as i32 + 1
                };
                if n != first_limb_len {
                    first_limb_len = n;
                    radix_pos += 1;

                    round_pos += 1;
                    if round_pos == LIMB_DIGITS {
                        round_pos = 0;
                        round_limb += 1;
                    }
                }
            } else if i == b.e {
                // printf.h:733-744 — carried clean off the top: new limb "1".
                first_limb_len = 1;
                radix_pos += 1;
                b.e += 1;

                round_pos += 1;
                if round_pos == LIMB_DIGITS {
                    round_pos = 0;
                    round_limb += 1;
                }
            }
        }
    } else if b.e <= round_limb {
        // printf.h:747-766 — got 0 or 1 after rounding.
        if b.e == round_limb {
            if !standard_rounding {
                round_up = b.at(b.e - 1) >= LIMB_MAX / 2;
            } else if b.at(b.e - 1) > LIMB_MAX / 2 {
                round_up = true;
            } else if b.at(b.e - 1) == LIMB_MAX / 2 {
                let mut r = b.e - 2;
                while r >= b.b {
                    if b.at(r) != 0 {
                        round_up = true;
                        break;
                    }
                    r -= 1;
                }
                // (no ties-to-even arm here: the kept value would be 0, which is
                // even — the C simply leaves round_up FALSE on a clean tie)
            }
        }

        b.set(round_limb, round_up as u32);
        b.b = round_limb;
        b.e = b.b + 1;
        first_limb_len = 1;
        radix_pos += 1;
    }

    if flags.format == b'g' || flags.format == b'G' {
        // printf.h:768-779 — %g picks %f or %e ('g'-1 / 'g'-2) and rebases the
        // precision to significant digits.
        trim_tail = true;

        if radix_pos >= -3 && radix_pos <= flags.precision {
            flags.format -= 1; // 'g'→'f', 'G'→'F'
            if flags.precision == 0 {
                flags.precision += 1;
            }
            flags.precision -= radix_pos;
        } else {
            flags.format -= 2; // 'g'→'e', 'G'→'E'
            if flags.precision > 0 {
                flags.precision -= 1;
            }
        }
    }

    if trim_tail && !flags.alternate {
        // printf.h:781-818 — %g without '#': trim trailing zero digits by
        // shrinking Precision, limb by limb from the round limb upward.
        let mut i = round_limb;
        while flags.precision > 0 && i < b.e {
            let mut l: u32 = if i >= b.b { b.at(i) } else { 0 };

            let limb_len: i32;
            if i == round_limb {
                let mut r: i32 = if flags.format == b'f' || flags.format == b'F' {
                    radix_pos + flags.precision
                } else {
                    flags.precision + 1
                };
                r = first_limb_len + LIMB_DIGITS * (b.e - 1 - b.b) - r;
                r %= LIMB_DIGITS;
                if r < 0 {
                    r += LIMB_DIGITS;
                }
                l /= P10S[r as usize];
                limb_len = LIMB_DIGITS - r;
            } else {
                limb_len = LIMB_DIGITS;
            }

            if l == 0 {
                flags.precision -= limb_len;
            } else {
                while l % 10 == 0 {
                    flags.precision -= 1;
                    l /= 10;
                }
            }

            if flags.precision <= 0 {
                flags.precision = 0;
                break;
            }
            if l != 0 {
                break;
            }
            i += 1;
        }
    }

    // printf.h:820-833 — total payload length for pf_fill.
    let mut len: i32 = flags.precision;
    if flags.precision != 0 || flags.alternate {
        len += 1; // the decimal point
    }
    if flags.format == b'f' || flags.format == b'F' {
        len += if radix_pos > 0 { radix_pos } else { 1 };
    } else if flags.format == b'e' || flags.format == b'E' {
        radix_pos -= 1; // %e: one digit before the point; radix_pos becomes THE EXPONENT
        if !trim_tail || radix_pos != 0 {
            len += 3; // strlen("1e+")
            if three_digit_exp || radix_pos < -99 || radix_pos > 99 {
                len += 3;
            } else {
                len += 2;
            }
        } else {
            len += 1;
        }
    }

    let mut r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, true);
    if r < 0 {
        return r;
    }
    let mut ret = r;

    f.format = b'd';
    f.pad_zero = true;
    if flags.format == b'f' || flags.format == b'F' {
        // printf.h:841-915 — the %f body.
        if radix_pos <= 0 {
            buf[0] = C::of(b'0');
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
        }

        // integer part, limb by limb (printf.h:849-867)
        let mut limb_len: i32 = LIMB_DIGITS;
        let mut i = b.e - 1;
        while radix_pos > 0 && i >= b.b {
            limb_len = if i == b.e - 1 { first_limb_len } else { LIMB_DIGITS };
            let mut l = b.at(i);
            if limb_len > radix_pos {
                f.precision = radix_pos;
                l /= P10S[(limb_len - radix_pos) as usize];
                limb_len -= radix_pos;
            } else {
                f.precision = limb_len;
                limb_len = LIMB_DIGITS;
            }
            radix_pos -= f.precision;
            pf_integer_conv::<C>(buf.as_mut_ptr(), &mut f, l as i64);

            r = pf_puts(puts_ctx, f.precision, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            i -= 1;
        }

        // trailing integer zeros past the stored limbs (printf.h:869-874)
        buf[0] = C::of(b'0');
        while radix_pos > 0 {
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            radix_pos -= 1;
        }

        if flags.precision != 0 || flags.alternate {
            buf[0] = C::of(decimal_point(locale)); // printf.h:877
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
        }

        // leading fraction zeros for |v| < 1 (printf.h:883-889)
        let mut prec = flags.precision;
        buf[0] = C::of(b'0');
        while prec > 0 && radix_pos + LIMB_DIGITS - first_limb_len < 0 {
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            radix_pos += 1;
            prec -= 1;
        }

        // fraction digits from the remaining limbs (printf.h:891-908). NOTE
        // `limb_len` carries over from the integer loop: a limb split by the
        // decimal point keeps its remaining digit count.
        while prec > 0 && i >= b.b {
            let mut l = b.at(i);
            if limb_len != LIMB_DIGITS {
                l %= P10S[limb_len as usize];
            }
            if limb_len > prec {
                f.precision = prec;
                l /= P10S[(limb_len - prec) as usize];
            } else {
                f.precision = limb_len;
                limb_len = LIMB_DIGITS;
            }
            prec -= f.precision;
            pf_integer_conv::<C>(buf.as_mut_ptr(), &mut f, l as i64);

            r = pf_puts(puts_ctx, f.precision, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            i -= 1;
        }

        // trailing fraction zeros (printf.h:910-915)
        buf[0] = C::of(b'0');
        while prec > 0 {
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            prec -= 1;
        }
    } else {
        // printf.h:916-976 — the %e body.
        let mut l = b.at(b.e - 1);
        l /= P10S[(first_limb_len - 1) as usize];

        buf[0] = C::of(b'0' + l as u8); // the single leading digit
        r = pf_puts(puts_ctx, 1, buf.as_ptr());
        if r < 0 {
            return r;
        }
        ret += r;

        if flags.precision != 0 || flags.alternate {
            buf[0] = C::of(decimal_point(locale)); // printf.h:926
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
        }

        // fraction digits (printf.h:932-954)
        let mut prec = flags.precision;
        let mut limb_len: i32 = LIMB_DIGITS;
        let mut i = b.e - 1;
        while prec > 0 && i >= b.b {
            let mut l = b.at(i);
            if i == b.e - 1 {
                limb_len = first_limb_len - 1;
                l %= P10S[limb_len as usize];
            }

            if limb_len > prec {
                f.precision = prec;
                l /= P10S[(limb_len - prec) as usize];
            } else {
                f.precision = limb_len;
                limb_len = LIMB_DIGITS;
            }
            prec -= f.precision;
            pf_integer_conv::<C>(buf.as_mut_ptr(), &mut f, l as i64);

            r = pf_puts(puts_ctx, f.precision, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            i -= 1;
        }

        // trailing fraction zeros (printf.h:956-961)
        buf[0] = C::of(b'0');
        while prec > 0 {
            r = pf_puts(puts_ctx, 1, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
            prec -= 1;
        }

        // exponent (printf.h:963-975) — suppressed by %g when it is exactly 0.
        if !trim_tail || radix_pos != 0 {
            buf[0] = C::of(flags.format); // 'e' or 'E' (already downshifted from g/G)
            buf[1] = C::of(if radix_pos < 0 { b'-' } else { b'+' });
            r = pf_puts(puts_ctx, 2, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;

            f.precision = if three_digit_exp { 3 } else { 2 };
            pf_integer_conv::<C>(buf.as_mut_ptr(), &mut f, radix_pos as i64);
            r = pf_puts(puts_ctx, f.precision, buf.as_ptr());
            if r < 0 {
                return r;
            }
            ret += r;
        }
    }

    r = pf_fill::<C>(pf_puts, puts_ctx, len, flags, false);
    if r < 0 {
        return r;
    }
    ret += r;
    ret
}

// ── THE ENGINE (printf.h:984-1245) ────────────────────────────────────────────

/// printf.h:984-1245 — `pf_printf`, the format-string state machine. Everything
/// printf-shaped in msvcrt funnels through this (see wcs.c:881-891 for the
/// canonical `_vsnprintf` shell: `pf_printf_a(puts_clbk_str_a, &ctx, format,
/// NULL, 0, arg_clbk_valist, NULL, &valist)` + a manual NUL append).
///
/// Returns chars written, or a negative sink/engine error.
unsafe fn pf_printf<C: PfChar>(
    pf_puts: PutsClbk<C>,
    puts_ctx: *mut c_void,
    fmt: *const C,
    locale: *mut c_void,
    options: u32,
    pf_args: ArgsClbk,
    args_ctx: *mut c_void,
    valist: *mut *mut u8,
) -> i32 {
    let mut p = fmt;
    let mut buf = [C::of(0); 32]; // printf.h:989
    let mut written: i32 = 0;
    let positional_params = options & MSVCRT_PRINTF_POSITIONAL_PARAMS != 0;
    let invoke_invalid_param_handler = options & MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER != 0;
    let ucrt_semantics = options & PRINTF_UCRT_SEMANTICS != 0;

    // printf.h:994-1003 — the ucrt arm reads the option bits; the msvcrt
    // (_MSVCR_VER == 0) arm hardcodes legacy behaviour + consults the process
    // exponent-format switch. Both transcribed; PRINTF_UCRT_SEMANTICS selects,
    // per call rather than per build.
    let legacy_wide: bool;
    let legacy_msvcrt_compat: bool;
    let three_digit_exp: bool;
    let standard_rounding: bool;
    if ucrt_semantics {
        legacy_wide = options & CRT_INTERNAL_PRINTF_LEGACY_WIDE_SPECIFIERS != 0;
        legacy_msvcrt_compat = options & CRT_INTERNAL_PRINTF_LEGACY_MSVCRT_COMPATIBILITY != 0;
        three_digit_exp = options & CRT_INTERNAL_PRINTF_LEGACY_THREE_DIGIT_EXPONENTS != 0;
        standard_rounding = options & CRT_INTERNAL_PRINTF_STANDARD_ROUNDING != 0;
    } else {
        legacy_wide = true;
        legacy_msvcrt_compat = true;
        three_digit_exp = _get_output_format() != TWO_DIGIT_EXPONENT;
        standard_rounding = false;
    }

    // printf.h:1005-1006 — MSVCRT_CHECK_PMT(fmt != NULL)
    if !crate::check_pmt(!fmt.is_null()) {
        return -1;
    }

    while (*p).u() != 0 {
        // output characters before '%' (printf.h:1009-1019)
        let mut q = p;
        while (*q).u() != 0 && (*q).u() != b'%' as u32 {
            q = q.add(1);
        }
        if p != q {
            let i = pf_puts(puts_ctx, q.offset_from(p) as i32, p);
            if i < 0 {
                return i;
            }

            written += i;
            p = q;
            continue;
        }

        // *p == '%' here (printf.h:1021-1022)
        p = p.add(1);

        // output a single '%' character (printf.h:1024-1032)
        if (*p).u() == b'%' as u32 {
            let i = pf_puts(puts_ctx, 1, p);
            p = p.add(1);
            if i < 0 {
                return i;
            }

            written += i;
            continue;
        }

        // check parameter position (printf.h:1034-1038)
        let mut pos: i32 = -1;
        if positional_params {
            let mut posv: i32 = 0;
            let q2 = pf_parse_int(p, &mut posv);
            if (*q2).u() == b'$' as u32 {
                pos = posv;
                p = q2.add(1);
            }
        }

        // parse the flags (printf.h:1040-1056)
        let mut flags = PfFlags::default(); // memset(&flags, 0, sizeof(flags))
        loop {
            let c = (*p).u();
            if c == b'+' as u32 || c == b' ' as u32 {
                if flags.sign != b'+' {
                    flags.sign = c as u8;
                }
            } else if c == b'-' as u32 {
                flags.left_align = true;
            } else if c == b'0' as u32 {
                flags.pad_zero = true;
            } else if c == b'#' as u32 {
                flags.alternate = true;
            } else {
                break; // includes c == 0 (the C's while(*p) guard)
            }

            p = p.add(1);
        }

        // parse the width (printf.h:1058-1081)
        if (*p).u() == b'*' as u32 {
            p = p.add(1);
            let mut i: i32 = -1;
            if positional_params {
                let mut iv: i32 = 0;
                let q2 = pf_parse_int(p, &mut iv);
                if (*q2).u() == b'$' as u32 {
                    i = iv;
                    p = q2.add(1);
                }
            }

            flags.field_length = pf_args(args_ctx, i, VT_INT, valist).get_int;
            if flags.field_length < 0 {
                flags.left_align = true;
                flags.field_length = flags.field_length.wrapping_neg();
            }
        }

        if ucrt_semantics {
            // printf.h:1073-1076 — ucrt resets a '*' width when literal digits
            // follow; msvcrt ACCUMULATES on top of it.
            if (*p).u() >= b'0' as u32 && (*p).u() <= b'9' as u32 {
                flags.field_length = 0;
            }
        }

        while (*p).u() >= b'0' as u32 && (*p).u() <= b'9' as u32 {
            flags.field_length = flags
                .field_length
                .wrapping_mul(10)
                .wrapping_add(((*p).u() - b'0' as u32) as i32);
            p = p.add(1);
        }

        // parse the precision (printf.h:1083-1100). NOTE a negative "%.*"
        // precision is stored AS IS (only ==-1 later means "default") — quirk.
        flags.precision = -1;
        if (*p).u() == b'.' as u32 {
            flags.precision = 0;
            p = p.add(1);
            if (*p).u() == b'*' as u32 {
                p = p.add(1);
                let mut i: i32 = -1;
                if positional_params {
                    let mut iv: i32 = 0;
                    let q2 = pf_parse_int(p, &mut iv);
                    if (*q2).u() == b'$' as u32 {
                        i = iv;
                        p = q2.add(1);
                    }
                }

                flags.precision = pf_args(args_ctx, i, VT_INT, valist).get_int;
            } else {
                while (*p).u() >= b'0' as u32 && (*p).u() <= b'9' as u32 {
                    flags.precision = flags
                        .precision
                        .wrapping_mul(10)
                        .wrapping_add(((*p).u() - b'0' as u32) as i32);
                    p = p.add(1);
                }
            }
        }

        // parse argument size modifier (printf.h:1102-1136)
        loop {
            let c = (*p).u();
            if c == 0 {
                break;
            }
            if c == b'l' as u32 && (*p.add(1)).u() == b'l' as u32 {
                flags.integer_double = true;
                p = p.add(1);
            } else if c == b'l' as u32 {
                flags.integer_length = LEN_LONG;
            } else if c == b'h' as u32 {
                flags.integer_length = LEN_SHORT;
            } else if c == b'I' as u32 {
                let c1 = (*p.add(1)).u();
                if c1 == b'6' as u32 && (*p.add(2)).u() == b'4' as u32 {
                    flags.integer_double = true;
                    p = p.add(2);
                } else if c1 == b'3' as u32 && (*p.add(2)).u() == b'2' as u32 {
                    p = p.add(2);
                } else if c1 != 0 && in_set(b"diouxX", c1) {
                    flags.integer_native = true;
                } else {
                    break; // bare 'I': NOT consumed → re-dispatched as Format
                }
            } else if c == b'w' as u32 {
                flags.wide_string = true;
            } else if (c == b'z' as u32 || c == b't' as u32)
                && {
                    let c1 = (*p.add(1)).u();
                    c1 != 0 && in_set(b"diouxX", c1)
                }
            {
                // printf.h:1123-1125 — `#if _MSVCR_VER == 0 || >= 140`, i.e. present
                // in BOTH versions, so unconditional here.
                flags.integer_native = true;
            } else if c == b'j' as u32 {
                flags.integer_double = true;
            } else if ucrt_semantics && c == b'T' as u32 {
                // printf.h:1129-1131 — ucrt-only
                flags.natural_string = true;
            } else if c != b'L' as u32
                && ((c != b'F' as u32 && c != b'N' as u32) || !legacy_msvcrt_compat)
            {
                break;
            }
            // ('L' — and 'F'/'N' under legacy compat — are consumed as no-ops)
            p = p.add(1);
        }

        // printf.h:1138 — `flags.Format = *p`: Format is a CHAR even in the wide
        // engine, so a wide format char TRUNCATES here (L'%Ť' dispatches as
        // 'd'). Quirk, kept.
        flags.format = (*p).u() as u8;

        let mut i: i32;
        if flags.format == b's' || flags.format == b'S' {
            // printf.h:1140-1143
            i = pf_handle_string::<C>(
                pf_puts,
                puts_ctx,
                pf_args(args_ctx, pos, VT_PTR, valist).get_ptr,
                -1,
                &mut flags,
                locale,
                legacy_wide,
            );
        } else if flags.format == b'c' || flags.format == b'C' {
            // printf.h:1144-1148 — the char is fetched as an INT and handed to
            // the string path as a 1-char buffer (&ch, little-endian).
            let ch: i32 = pf_args(args_ctx, pos, VT_INT, valist).get_int;

            i = pf_handle_string::<C>(
                pf_puts,
                puts_ctx,
                &ch as *const i32 as *const c_void,
                1,
                &mut flags,
                locale,
                legacy_wide,
            );
            if i < 0 {
                i = 0; // ignore conversion error (printf.h:1148)
            }
        } else if flags.format == b'p' {
            // printf.h:1149-1163 — %p = zero-padded pointer-width %X. The C
            // stashes Precision in `i` across the conv; `saved` here.
            flags.format = b'X';
            flags.pad_zero = true;
            let saved = flags.precision;
            flags.precision = 2 * core::mem::size_of::<*mut c_void>() as i32;
            // (ULONG_PTR)get_ptr: zero-extend the address to i64 (u64 bit-cast —
            // an amd64 high-half pointer goes negative, harmless under Format='X')
            pf_integer_conv::<C>(
                buf.as_mut_ptr(),
                &mut flags,
                pf_args(args_ctx, pos, VT_PTR, valist).get_ptr as usize as u64 as i64,
            );
            flags.pad_zero = false;
            flags.precision = saved;

            i = pf_output_format_apichar::<C>(pf_puts, puts_ctx, buf.as_ptr(), -1, &mut flags, locale);
        } else if flags.format == b'n' {
            // printf.h:1164-1174
            if N_FORMAT_ENABLED.load(Ordering::Relaxed) == 0 {
                // MSVCRT_INVALID_PMT("'n' format specifier disabled", EINVAL)
                crate::invalid_pmt(crate::EINVAL);
                return -1;
            }

            let used = pf_args(args_ctx, pos, VT_PTR, valist).get_ptr as *mut i32;
            *used = written;
            i = 0;
        } else if flags.format != 0 && in_set(b"diouxX", flags.format as u32) {
            // printf.h:1175-1210 — the integer path.
            let mut tmp: *mut C = buf.as_mut_ptr();

            // 0 padding is added after '0x' if Alternate flag is in use (printf.h:1179-1182)
            if (flags.format == b'x' || flags.format == b'X')
                && flags.pad_zero
                && flags.alternate
                && !flags.left_align
                && flags.precision < flags.field_length - 2
            {
                flags.precision = flags.field_length - 2;
            }

            let max_len =
                (if flags.field_length > flags.precision { flags.field_length } else { flags.precision }) + 10;
            if max_len > 32 {
                // BUG-KEPT (printf.h:1186): `HeapAlloc(GetProcessHeap(), 0, max_len)`
                // allocates max_len BYTES, not max_len*sizeof(APICHAR) — the WIDE
                // engine under-allocates 2x for a huge width/precision (heap
                // overrun in the C too). Upstream Wine has `* sizeof(APICHAR)`
                // here; THIS tree's copy does not, and we transcribe THIS tree.
                // XXX-VERIFY: if integration ever syncs printf.h with upstream,
                // flip this to `max_len as usize * size_of::<C>()`.
                tmp = HeapAlloc(GetProcessHeap(), 0, max_len as usize) as *mut C;
            }
            if tmp.is_null() {
                return -1;
            }

            if flags.integer_double || (flags.integer_native && core::mem::size_of::<*mut c_void>() == 8) {
                pf_integer_conv::<C>(tmp, &mut flags, pf_args(args_ctx, pos, VT_I8, valist).get_longlong);
            } else if flags.format == b'd' || flags.format == b'i' {
                // printf.h:1193-1197 — the C's ternary evaluates ONE pf_args call;
                // hoisted (borrowck) without reordering any side effect.
                let gi = pf_args(args_ctx, pos, VT_INT, valist).get_int;
                let x: i64 = if flags.integer_length != LEN_SHORT {
                    gi as i64
                } else {
                    gi as i16 as i64 // (short) truncate-then-sign-extend
                };
                pf_integer_conv::<C>(tmp, &mut flags, x);
            } else {
                // printf.h:1198-1202
                let gi = pf_args(args_ctx, pos, VT_INT, valist).get_int;
                let x: i64 = if flags.integer_length != LEN_SHORT {
                    gi as u32 as i64 // (unsigned)get_int — zero-extend
                } else {
                    gi as u16 as i64 // (unsigned short)get_int
                };
                pf_integer_conv::<C>(tmp, &mut flags, x);
            }

            i = pf_output_format_apichar::<C>(pf_puts, puts_ctx, tmp, -1, &mut flags, locale);
            if tmp != buf.as_mut_ptr() {
                HeapFree(GetProcessHeap(), 0, tmp as *mut c_void);
            }
        } else if flags.format != 0 && in_set(b"aAeEfFgG", flags.format as u32) {
            // printf.h:1211-1227 — the float path.
            let mut val: f64 = pf_args(args_ctx, pos, VT_R8, valist).get_double;

            if f64_bits(&val) >> 63 != 0 {
                // signbit(val): move the sign into flags; `val = -val` is exactly
                // a sign-bit flip in IEEE-754 (works for -0.0/-inf/-nan too).
                flags.sign = b'-';
                let bits = f64_bits(&val) & !(1u64 << 63);
                f64_set_bits(&mut val, bits);
            }

            if f64_is_inf(&val) || f64_is_nan(&val) {
                i = pf_output_special_fp::<C>(
                    pf_puts,
                    puts_ctx,
                    val,
                    &mut flags,
                    locale,
                    legacy_msvcrt_compat,
                    three_digit_exp,
                );
            } else if flags.format == b'a' || flags.format == b'A' {
                i = pf_output_hex_fp::<C>(pf_puts, puts_ctx, val, &mut flags, locale, standard_rounding);
            } else {
                i = pf_output_fp::<C>(
                    pf_puts,
                    puts_ctx,
                    val,
                    &mut flags,
                    locale,
                    three_digit_exp,
                    standard_rounding,
                );
            }
        } else {
            // printf.h:1228-1236 — unknown conversion.
            if invoke_invalid_param_handler {
                // NOTE order: handler FIRST, then errno — the OPPOSITE of
                // MSVCRT_INVALID_PMT. Transcribed as written.
                crate::_invalid_parameter(
                    core::ptr::null(),
                    core::ptr::null(),
                    core::ptr::null(),
                    0,
                    0,
                );
                *crate::_errno() = crate::EINVAL;
                return -1;
            }

            continue; // does NOT advance p: the unknown char is re-scanned and
                      // printed as literal text ("%q" prints "q"). Quirk, kept.
        }

        if i < 0 {
            return i;
        }
        written += i;
        p = p.add(1);
    }

    written
}

/// msvcrt.h:390-391 — `int pf_printf_a(puts_clbk_a, void*, const char*,
/// _locale_t, DWORD, args_clbk, void*, va_list*)`.
#[no_mangle]
pub unsafe extern "C" fn pf_printf_a(
    pf_puts: PutsClbk<u8>,
    puts_ctx: *mut c_void,
    fmt: *const u8,
    locale: *mut c_void,
    options: u32,
    pf_args: ArgsClbk,
    args_ctx: *mut c_void,
    valist: *mut *mut u8,
) -> i32 {
    pf_printf::<u8>(pf_puts, puts_ctx, fmt, locale, options, pf_args, args_ctx, valist)
}

/// msvcrt.h:392-393 — the wide twin.
#[no_mangle]
pub unsafe extern "C" fn pf_printf_w(
    pf_puts: PutsClbk<u16>,
    puts_ctx: *mut c_void,
    fmt: *const u16,
    locale: *mut c_void,
    options: u32,
    pf_args: ArgsClbk,
    args_ctx: *mut c_void,
    valist: *mut *mut u8,
) -> i32 {
    pf_printf::<u16>(pf_puts, puts_ctx, fmt, locale, options, pf_args, args_ctx, valist)
}

// ── args callbacks (wcs.c:844-874) ────────────────────────────────────────────

/// wcs.c:847-865 — `arg_clbk_valist`: the sequential (non-positional) argument
/// source; each call does one `va_arg(*valist, T)` on the shared cursor.
#[no_mangle]
pub unsafe extern "C" fn arg_clbk_valist(
    _ctx: *mut c_void,
    _arg_pos: i32,
    ty: i32,
    valist: *mut *mut u8,
) -> PrintfArg {
    // C declares `printf_arg ret;` (uninit) and sets ONE member; zero-init is
    // the defined-behaviour superset of that.
    let mut ret = PrintfArg { get_longlong: 0 };

    if ty == VT_I8 {
        ret.get_longlong = va_next::<i64>(&mut *valist);
    } else if ty == VT_INT {
        ret.get_int = va_next::<i32>(&mut *valist);
    } else if ty == VT_R8 {
        // va_arg(*valist, double) — read as raw bits (u64) into the union: bit-
        // identical to the C, no f64-by-value in any signature (no-SSE rule).
        ret.get_longlong = va_next::<u64>(&mut *valist) as i64;
    } else if ty == VT_PTR {
        ret.get_ptr = va_next::<*mut c_void>(&mut *valist);
    } else {
        // ERR("Incorrect type\n") — debug-channel print only in Wine; no-op here.
        ret.get_int = 0;
    }

    ret
}

/// wcs.c:870-874 — `arg_clbk_positional`: table lookup, NO bounds check. A
/// non-positional '*' width inside a positional format hands pos == -1 and the
/// C reads args[-1] — verbatim (create_positional_ctx normally rejects mixed
/// formats before this can be reached).
#[no_mangle]
pub unsafe extern "C" fn arg_clbk_positional(
    ctx: *mut c_void,
    pos: i32,
    _ty: i32,
    _valist: *mut *mut u8,
) -> PrintfArg {
    let args = ctx as *const PrintfArg;
    *args.offset(pos as isize)
}

// ── positional-parameter machinery (printf.h:1247-1318) ──────────────────────

// printf.h:1248-1253 — enum types_clbk_flags
const TYPE_CLBK_VA_LIST: i32 = 1;
const TYPE_CLBK_POSITIONAL: i32 = 2;
const TYPE_CLBK_ERROR_POS: i32 = 4;
const TYPE_CLBK_ERROR_TYPE: i32 = 8;

/// printf.h:1255-1275 — `arg_clbk_type`: the TYPE-COLLECTION pass. Stores each
/// position's VT_ type into `args[pos]`, and accumulates status flags in
/// `args[0]`. Defined once (the `#ifndef PRINTF_WIDE` guard) and shared by both
/// engine widths, as here.
unsafe extern "C" fn arg_clbk_type(
    ctx: *mut c_void,
    pos: i32,
    ty: i32,
    _valist: *mut *mut u8,
) -> PrintfArg {
    let ret = PrintfArg { get_longlong: 0 }; // static const printf_arg ret (zeroed)
    let args = ctx as *mut PrintfArg;

    if pos == -1 {
        (*args).get_int |= TYPE_CLBK_VA_LIST;
        return ret;
    } else {
        (*args).get_int |= TYPE_CLBK_POSITIONAL;
    }

    if pos < 1 || pos > ARGMAX {
        (*args).get_int |= TYPE_CLBK_ERROR_POS;
    } else if (*args.offset(pos as isize)).get_int != 0 && (*args.offset(pos as isize)).get_int != ty {
        (*args).get_int |= TYPE_CLBK_ERROR_TYPE;
    } else {
        (*args.offset(pos as isize)).get_int = ty;
    }

    ret
}

/// printf.h:1278-1318 — `create_positional_ctx`: dry-run the format with
/// `arg_clbk_type` over a NULL sink, then pull the arguments OUT OF ORDER from
/// the (by-value) va cursor into the table. Returns:
///   * 0   — format is purely sequential (use arg_clbk_valist instead)
///   * <0  — mixed/invalid positional usage
///   * j>0 — table filled through index j-1 (use arg_clbk_positional)
///
/// `args_ctx` must be a zeroed `[PrintfArg; ARGMAX+1]` (the C shells memset it —
/// wcs.c:1200).
unsafe fn create_positional_ctx<C: PfChar>(
    args_ctx: *mut c_void,
    format: *const C,
    valist: *mut u8,
) -> i32 {
    // struct _str_ctx puts_ctx = {INT_MAX, NULL} — a pure counting sink.
    let mut puts_ctx: PfStrCtx<C> = PfStrCtx { len: i32::MAX as usize, buf: core::ptr::null_mut() };
    let args = args_ctx as *mut PrintfArg;

    let i = pf_printf::<C>(
        puts_clbk_str::<C>,
        &mut puts_ctx as *mut PfStrCtx<C> as *mut c_void,
        format,
        core::ptr::null_mut(),
        MSVCRT_PRINTF_POSITIONAL_PARAMS,
        arg_clbk_type,
        args_ctx,
        core::ptr::null_mut(), // the C passes NULL for the va_list* here
    );
    if i < 0 {
        return i;
    }

    if (*args).get_int == 0 || (*args).get_int == TYPE_CLBK_VA_LIST {
        return 0;
    }
    if (*args).get_int != TYPE_CLBK_POSITIONAL {
        return -1;
    }

    // highest used position (printf.h:1294-1296)
    let mut i: i32 = ARGMAX;
    while i > 0 {
        if (*args.offset(i as isize)).get_int != 0 {
            break;
        }
        i -= 1;
    }

    // pull the args in slot order (printf.h:1298-1315). The cursor is BY VALUE
    // (the C takes `va_list valist`), so the caller's copy is not advanced.
    let mut cursor = valist;
    let mut j: i32 = 1;
    while j <= i {
        let slot = args.offset(j as isize);
        match (*slot).get_int {
            VT_I8 => (*slot).get_longlong = va_next::<i64>(&mut cursor),
            VT_INT => (*slot).get_int = va_next::<i32>(&mut cursor),
            // double as raw bits — see arg_clbk_valist.
            VT_R8 => (*slot).get_longlong = va_next::<u64>(&mut cursor) as i64,
            VT_PTR => (*slot).get_ptr = va_next::<*mut c_void>(&mut cursor),
            _ => return -1,
        }
        j += 1;
    }

    j // NOTE: returns j == i+1, and returns 1 when the table ended up empty — verbatim
}

/// msvcrt.h:394 — `int create_positional_ctx_a(void*, const char*, va_list)`.
/// `valist` is the flat byte cursor BY VALUE (a copy — the C's va_list is a
/// `char*` on all three targets).
#[no_mangle]
pub unsafe extern "C" fn create_positional_ctx_a(
    args_ctx: *mut c_void,
    format: *const u8,
    valist: *mut u8,
) -> i32 {
    create_positional_ctx::<u8>(args_ctx, format, valist)
}

/// msvcrt.h:395 — the wide twin.
#[no_mangle]
pub unsafe extern "C" fn create_positional_ctx_w(
    args_ctx: *mut c_void,
    format: *const u16,
    valist: *mut u8,
) -> i32 {
    create_positional_ctx::<u16>(args_ctx, format, valist)
}
