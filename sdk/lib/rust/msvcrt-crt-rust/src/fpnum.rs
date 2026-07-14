//! Wave 13 — msvcrt's **bignum float parser**, in Rust.
//!
//! This is the module wave 5 said we could not have.
//!
//! # Why this is a TRANSLATION and not an assembly
//!
//! Every other wave in this lane has been *assembly*: cnt-rust already had the
//! algorithm, and the work was to bolt msvcrt's contract onto it. Wave 5 probed
//! whether that trick reached `strtod` and got a hard NO — cnt-rust's parser is DJ
//! Delorie's naive `r = r*10 + d` accumulator (its own doc admits it "was never IEEE
//! correctly-rounded"), while msvcrt's is Wine's base-10 **bignum**. They disagreed by
//! **14 ULPs** on the very first hard-to-round input:
//!
//! ```text
//! "0.0000000000000000087519423665610e-32"
//!   cnt-rust (DJ Delorie):  0x35c05f590315f1e3
//!   msvcrt   (bignum):      0x35c05f590315f1d5
//! ```
//!
//! They were never the same function. So the honest path — the one the plan recorded —
//! is to port **msvcrt's own machinery**: `bnum.h` plus `string.c`'s `fpnum_*`. That is
//! what this file is. `strtod` is not a wrapper around someone else's parser here; it is
//! msvcrt's parser.
//!
//! The wave-5 probe stays in `classify` as an executable DO-NOT-FLIP record for the
//! cnt-rust route. It should still DIVERGE. What changes is that `strtod` no longer
//! *takes* that route.
//!
//! # The oracle
//!
//! Bit-exact. `classify` compares the raw `u64` bit pattern of the returned `double`
//! against the genuine `ref_strtod` (msvcrt's own `string.c`, compiled for the host),
//! plus the `endptr` displacement and `errno`. There is no "close enough" — a 1-ULP
//! disagreement is a DIVERGE. This is the strongest oracle the lane has ever had, and
//! it is why porting a bignum by hand is a sane thing to attempt at all.
//!
//! # `_MSVCR_VER == 0` — what is deliberately ABSENT
//!
//! msvcrt.dll is built `-D_MSVCR_VER=0` (`dll/win32/msvcrt/CMakeLists.txt:180`), and the
//! harness compiles the reference with the same define. Under it, three blocks of
//! `string.c` **do not exist**, and so they do not exist here either:
//!
//!   * `fpnum_parse16` — hex-float (`0x1.8p3`) parsing        (`#if _MSVCR_VER >= 140`)
//!   * the `infinity` / `nan` word matcher in `fpnum_parse_bnum`  (same guard)
//!   * `strtof` / `_strtof_l`                                  (`#if _MSVCR_VER >= 120`)
//!
//! So in *this* CRT, `strtod("inf")` is 0.0 and `strtod("0x10")` parses the `0` and stops
//! at the `x`. That is not a shortcut — it is the contract, and `classify` pins it with
//! explicit rows. If anyone ever raises `_MSVCR_VER`, the reference grows those branches,
//! our Rust does not, and those rows go DIVERGE **loudly**. The guard is structural, not
//! a comment.

use crate::sys;

// ── bnum.h ────────────────────────────────────────────────────────────────────
//
// A base-10 bignum with a fixed decimal point, held in a POWER-OF-TWO circular
// buffer of 9-digit limbs. `b` is the least-significant limb index and `e` the
// most-significant + 1 — and note that `b` *decreases* as the number grows, wrapping
// negative through `bnum_idx`'s mask. (`-1 & (size-1) == size-1` in two's complement,
// in C and in Rust alike, which is what makes the wrap work.)

const LIMB_DIGITS: i32 = 9;
const LIMB_MAX: u32 = 1_000_000_000; // 10^9

/// Data buffer size in DWORDs — enough to hold a 64-bit double. Power of two.
const BNUM_PREC64: usize = 128;
/// …and an 80-bit long double. Also a power of two. 8 KiB of stack, exactly as the C
/// spends it (`BYTE bnum_data[FIELD_OFFSET(struct bnum, data[BNUM_PREC80])]`).
const BNUM_PREC80: usize = 2048;

const P10S: [u32; 10] = [
    1, 10, 100, 1000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000,
];

const EXP_BITS: i32 = 11;
const MANT_BITS: i32 = 53;
const LDBL_EXP_BITS: i32 = 15;
const LDBL_MANT_BITS: i32 = 64;

// <float.h>, for the exponent-range early-outs. Pinned by `_Static_assert` in
// dll/win32/msvcrt/rust_layout_assert.c so a header change cannot silently move them.
const DBL_MAX_10_EXP: i32 = 308;
const DBL_MIN_10_EXP: i32 = -307;
const DBL_DIG: i32 = 15;
// msvcrt.h:72-73
const DBL80_MAX_10_EXP: i32 = 4932;
const DBL80_MIN_10_EXP: i32 = -4951;

/// `struct bnum`, with the flexible array turned into a borrowed slice. The C stack-
/// allocates the buffer in `fpnum_parse` and hands the struct down; we do the same, and
/// the slice length carries `size` so the two can never disagree.
struct Bnum<'a> {
    /// least significant limb position (decreases as limbs are prepended)
    b: i32,
    /// most significant limb position + 1
    e: i32,
    /// buffer size in limbs — a power of two
    size: i32,
    data: &'a mut [u32],
}

impl Bnum<'_> {
    /// `bnum_idx` — the circular-buffer mask. `idx` is routinely NEGATIVE (`b->b-1`);
    /// masking a negative i32 with `size-1` yields the wrapped slot, in Rust exactly as
    /// in C, because both are two's complement.
    #[inline]
    fn idx(&self, i: i32) -> usize {
        (i & (self.size - 1)) as usize
    }

    #[inline]
    fn at(&self, i: i32) -> u32 {
        self.data[self.idx(i)]
    }

    #[inline]
    fn set(&mut self, i: i32, v: u32) {
        let k = self.idx(i);
        self.data[k] = v;
    }

    /// `bnum_lshift`. Returns true if a new most-significant limb was added.
    ///
    /// Invariant (the C `assert`s it, and it is compiled out under NDEBUG): `shift <= 29`,
    /// so the limb count can grow by at most 1.
    fn lshift(&mut self, shift: i32) -> bool {
        let mut rest: u32 = 0;
        let mut i = self.b;
        while i < self.e {
            let tmp = ((self.at(i) as u64) << shift) + rest as u64;
            rest = (tmp / LIMB_MAX as u64) as u32;
            self.set(i, (tmp % LIMB_MAX as u64) as u32);

            if i == self.b && self.at(i) == 0 {
                self.b += 1;
            }
            i += 1;
        }

        if rest != 0 {
            self.set(self.e, rest);
            self.e += 1;

            if self.idx(self.b) == self.idx(self.e) {
                if self.at(self.b) != 0 {
                    let v = self.at(self.b + 1) | 1;
                    self.set(self.b + 1, v);
                }
                self.b += 1;
            }
            return true;
        }
        false
    }

    /// `bnum_rshift`. Returns true if the most-significant limb was removed.
    ///
    /// Invariant: `shift <= 9` (so `LIMB_MAX >> shift` loses no accuracy).
    fn rshift(&mut self, shift: i32) -> bool {
        let mut rest: u32 = 0;
        let mut ret = false;

        let mut i = self.e - 1;
        while i >= self.b {
            let tmp = self.at(i) & ((1u32 << shift) - 1);
            self.set(i, (self.at(i) >> shift) + rest);
            rest = (LIMB_MAX >> shift) * tmp;
            if i == self.e - 1 && self.at(i) == 0 {
                self.e -= 1;
                ret = true;
            }
            i -= 1;
        }

        if rest != 0 {
            if self.idx(self.b - 1) == self.idx(self.e) {
                // no room: fold the lost limb into a sticky bit
                let v = self.at(self.b) | 1;
                self.set(self.b, v);
            } else {
                self.b -= 1;
                self.set(self.b, rest);
            }
        }
        ret
    }

    /// `bnum_mult`. Invariant: `mult <= LIMB_MAX`.
    fn mult(&mut self, mult: u32) {
        let mut rest: u32 = 0;
        let mut i = self.b;
        while i < self.e {
            let tmp = (self.at(i) as u64) * mult as u64 + rest as u64;
            rest = (tmp / LIMB_MAX as u64) as u32;
            self.set(i, (tmp % LIMB_MAX as u64) as u32);

            if i == self.b && self.at(i) == 0 {
                self.b += 1;
            }
            i += 1;
        }

        if rest != 0 {
            self.set(self.e, rest);
            self.e += 1;

            if self.idx(self.b) == self.idx(self.e) {
                if self.at(self.b) != 0 {
                    let v = self.at(self.b + 1) | 1;
                    self.set(self.b + 1, v);
                }
                self.b += 1;
            }
        }
    }

    /// `bnum_to_mant` — the top three limbs as a u64. `None` on overflow.
    fn to_mant(&self) -> Option<u64> {
        const LM: u64 = LIMB_MAX as u64;

        if u64::MAX / LM / LM < self.at(self.e - 1) as u64 {
            return None;
        }
        let mut m = self.at(self.e - 1) as u64 * LM * LM;
        if self.b == self.e - 1 {
            return Some(m);
        }

        if u64::MAX - m < self.at(self.e - 2) as u64 * LM {
            return None;
        }
        m += self.at(self.e - 2) as u64 * LM;
        if self.b == self.e - 2 {
            return Some(m);
        }

        if u64::MAX - m < self.at(self.e - 3) as u64 {
            return None;
        }
        m += self.at(self.e - 3) as u64;
        Some(m)
    }
}

// ── struct fpnum / enum fpmod (msvcrt.h:350-364) ──────────────────────────────

/// `enum fpmod` — how the digits BELOW the retained mantissa should round it.
/// The discriminants are the C's, and `fpnum_double` compares them by value.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum FpMod {
    /// only used when the dropped part contains only zeros
    RoundZero = 0,
    RoundDown = 1,
    RoundEven = 2,
    RoundUp = 3,
    ValInfinity = 4,
    ValNan = 5,
}
use FpMod::*;

#[derive(Clone, Copy)]
struct FpNum {
    sign: i32,
    exp: i32,
    m: u64,
    md: FpMod,
}

#[inline]
fn fpnum(sign: i32, exp: i32, m: u64, md: FpMod) -> FpNum {
    FpNum { sign, exp, m, md }
}

// ── the character source ──────────────────────────────────────────────────────
//
// The C passes `wchar_t get(void*)` / `void unget(void*)` function pointers over a
// `const char**` (or `const wchar_t**`) cursor. A trait is the same thing without the
// indirect call.

/// `WEOF`. msvcrt's `wint_t` is **16-bit**, so this is 0xFFFF — see wave 11.
const WEOF: u16 = 0xFFFF;

trait CharSrc {
    /// # Safety
    /// The cursor must point into a NUL-terminated string.
    unsafe fn get(&mut self) -> u16;
    /// # Safety
    /// Only legal after a matching `get`.
    unsafe fn unget(&mut self);
}

/// `strtod_str_get` / `strtod_str_unget` (string.c:1011-1021).
///
/// **The sign-extension is load-bearing.** The C is
///
/// ```c
/// static wchar_t strtod_str_get(void *ctx) {
///     const char **p = ctx;
///     if (!**p) return WEOF;
///     return *(*p)++;          /* <-- a SIGNED char, widened to unsigned short */
/// }
/// ```
///
/// `char` is signed on all three of our targets (asserted in `rust_layout_assert.c`), so
/// a byte of 0x80 arrives as 0xFF80 — and a byte of **0xFF arrives as 0xFFFF, which *is*
/// `WEOF`**. So `strtod("1\xFF")` sees end-of-input at the 0xFF, yet the cursor has
/// ALREADY advanced past it, and `if (nch != WEOF) unget(ctx)` therefore does not put it
/// back. The endptr lands one byte too far. That is msvcrt's behaviour; `classify` has a
/// row for it. Do not "fix" it.
struct StrSrc {
    p: *const u8,
}

impl CharSrc for StrSrc {
    #[inline]
    unsafe fn get(&mut self) -> u16 {
        if *self.p == 0 {
            return WEOF;
        }
        let c = *self.p as i8 as i16 as u16;
        self.p = self.p.add(1);
        c
    }
    #[inline]
    unsafe fn unget(&mut self) {
        self.p = self.p.sub(1);
    }
}

/// `strtod_wstr_get` / `strtod_wstr_unget` (wcs.c:558-569). `wchar_t` is already
/// unsigned, so there is no sign-extension trap on this side — but a literal 0xFFFF
/// character still reads as WEOF, symmetrically.
struct WStrSrc {
    p: *const u16,
}

impl CharSrc for WStrSrc {
    #[inline]
    unsafe fn get(&mut self) -> u16 {
        if *self.p == 0 {
            return WEOF;
        }
        let c = *self.p;
        self.p = self.p.add(1);
        c
    }
    #[inline]
    unsafe fn unget(&mut self) {
        self.p = self.p.sub(1);
    }
}

// ── fpnum_double (string.c:365-477) ───────────────────────────────────────────

/// Assemble an IEEE-754 binary64 out of a parsed `fpnum`, applying the round mode.
/// Returns 0, or `ERANGE` on overflow/underflow.
fn fpnum_double(fp: &mut FpNum, d: &mut f64) -> i32 {
    let mut bits: u64 = 0;

    if fp.md == ValInfinity {
        *d = fp.sign as f64 * f64::INFINITY;
        return 0;
    }
    if fp.md == ValNan {
        bits = !0;
        if fp.sign == 1 {
            bits &= !(1u64 << (MANT_BITS + EXP_BITS - 1));
        }
        unsafe { store_bits(d, bits) };
        return 0;
    }
    if fp.m == 0 {
        // `fp->sign * 0.0` — signed zero. -1 * 0.0 is -0.0, and the harness compares
        // BIT PATTERNS, so this is not a distinction without a difference.
        *d = fp.sign as f64 * 0.0;
        return 0;
    }

    // make sure that we don't overflow modifying exponent
    if fp.exp > 1 << EXP_BITS {
        *d = fp.sign as f64 * f64::INFINITY;
        return crate::ERANGE;
    }
    if fp.exp < -(1 << EXP_BITS) {
        *d = fp.sign as f64 * 0.0;
        return crate::ERANGE;
    }
    fp.exp += MANT_BITS - 1;

    // normalize mantissa
    while fp.m < 1u64 << (MANT_BITS - 1) {
        fp.m <<= 1;
        fp.exp -= 1;
    }
    while fp.m >= 1u64 << MANT_BITS {
        if fp.m & 1 != 0 || fp.md != RoundZero {
            if fp.m & 1 == 0 {
                fp.md = RoundDown;
            } else if fp.md == RoundZero {
                fp.md = RoundEven;
            } else {
                fp.md = RoundUp;
            }
        }
        fp.m >>= 1;
        fp.exp += 1;
    }
    fp.exp += (1 << (EXP_BITS - 1)) - 1;

    // handle subnormals
    if fp.exp <= 0 {
        if fp.m & 1 != 0 && fp.md == RoundZero {
            fp.md = RoundEven;
        } else if fp.m & 1 != 0 {
            fp.md = RoundUp;
        } else if fp.md != RoundZero {
            fp.md = RoundDown;
        }
        fp.m >>= 1;
    }
    while fp.m != 0 && fp.exp < 0 {
        if fp.m & 1 != 0 && fp.md == RoundZero {
            fp.md = RoundEven;
        } else if fp.m & 1 != 0 {
            fp.md = RoundUp;
        } else if fp.md != RoundZero {
            fp.md = RoundDown;
        }
        fp.m >>= 1;
        fp.exp += 1;
    }

    // round mantissa
    if fp.md == RoundUp || (fp.md == RoundEven && fp.m & 1 != 0) {
        fp.m += 1;

        // handle subnormal that falls into regular range due to rounding
        if fp.m == 1u64 << (MANT_BITS - 1) {
            fp.exp += 1;
        } else if fp.m >= 1u64 << MANT_BITS {
            fp.exp += 1;
            fp.m >>= 1;
        }
    }

    if fp.exp >= (1 << EXP_BITS) - 1 {
        *d = fp.sign as f64 * f64::INFINITY;
        return crate::ERANGE;
    }
    if fp.m == 0 || fp.exp < 0 {
        *d = fp.sign as f64 * 0.0;
        return crate::ERANGE;
    }

    if fp.sign == -1 {
        bits |= 1u64 << (MANT_BITS + EXP_BITS - 1);
    }
    bits |= (fp.exp as u64) << (MANT_BITS - 1);
    bits |= fp.m & ((1u64 << (MANT_BITS - 1)) - 1);

    unsafe { store_bits(d, bits) };
    0
}

// ── fpnum_ldouble (string.c:482-607) ──────────────────────────────────────────

/// `WINE_BROKEN_LDOUBLE` — `struct { ULONG x80[3]; }`, 12 bytes. This is the *internal*
/// shape the C computes into. The value it finally hands the caller is the 10-byte
/// `MSVCRT__LDOUBLE`; see `__STRINGTOLD_L` for the truncating copy that bridges them.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WineBrokenLdouble {
    x80: [u32; 3],
}

impl WineBrokenLdouble {
    #[inline]
    fn inf(sign: i32) -> Self {
        let mut d = Self {
            x80: [0, 0x8000_0000, (1 << LDBL_EXP_BITS) - 1],
        };
        if sign == -1 {
            d.x80[2] |= 1 << LDBL_EXP_BITS;
        }
        d
    }
    #[inline]
    fn zero(sign: i32) -> Self {
        let mut d = Self { x80: [0, 0, 0] };
        if sign == -1 {
            d.x80[2] |= 1 << LDBL_EXP_BITS;
        }
        d
    }
}

fn fpnum_ldouble(fp: &mut FpNum, d: &mut WineBrokenLdouble) -> i32 {
    if fp.md == ValInfinity {
        *d = WineBrokenLdouble::inf(fp.sign);
        return 0;
    }
    if fp.md == ValNan {
        let mut v = WineBrokenLdouble {
            x80: [!0, !0, (1 << LDBL_EXP_BITS) - 1],
        };
        if fp.sign == -1 {
            v.x80[2] |= 1 << LDBL_EXP_BITS;
        }
        *d = v;
        return 0;
    }
    if fp.m == 0 {
        *d = WineBrokenLdouble::zero(fp.sign);
        return 0;
    }

    if fp.exp > 1 << LDBL_EXP_BITS {
        *d = WineBrokenLdouble::inf(fp.sign);
        return crate::ERANGE;
    }
    if fp.exp < -(1 << LDBL_EXP_BITS) {
        *d = WineBrokenLdouble::zero(fp.sign);
        return crate::ERANGE;
    }
    fp.exp += LDBL_MANT_BITS - 1;

    // normalize mantissa. NOTE the asymmetry with fpnum_double: there is no
    // "while m >= 1<<MANT_BITS" loop here, because an 80-bit mantissa is a full u64 and
    // cannot exceed it. Transcribed, not inferred.
    while fp.m < 1u64 << (LDBL_MANT_BITS - 1) {
        fp.m <<= 1;
        fp.exp -= 1;
    }
    fp.exp += (1 << (LDBL_EXP_BITS - 1)) - 1;

    // handle subnormals
    if fp.exp <= 0 {
        if fp.m & 1 != 0 && fp.md == RoundZero {
            fp.md = RoundEven;
        } else if fp.m & 1 != 0 {
            fp.md = RoundUp;
        } else if fp.md != RoundZero {
            fp.md = RoundDown;
        }
        fp.m >>= 1;
    }
    while fp.m != 0 && fp.exp < 0 {
        if fp.m & 1 != 0 && fp.md == RoundZero {
            fp.md = RoundEven;
        } else if fp.m & 1 != 0 {
            fp.md = RoundUp;
        } else if fp.md != RoundZero {
            fp.md = RoundDown;
        }
        fp.m >>= 1;
        fp.exp += 1;
    }

    // round mantissa
    if fp.md == RoundUp || (fp.md == RoundEven && fp.m & 1 != 0) {
        if fp.m == u64::MAX {
            fp.m = 1u64 << (LDBL_MANT_BITS - 1);
            fp.exp += 1;
        } else {
            fp.m += 1;
            // handle subnormal that falls into regular range due to rounding
            if (fp.m ^ (fp.m - 1)) & (1u64 << (LDBL_MANT_BITS - 1)) != 0 {
                fp.exp += 1;
            }
        }
    }

    if fp.exp >= (1 << LDBL_EXP_BITS) - 1 {
        *d = WineBrokenLdouble::inf(fp.sign);
        return crate::ERANGE;
    }
    if fp.m == 0 || fp.exp < 0 {
        *d = WineBrokenLdouble::zero(fp.sign);
        return crate::ERANGE;
    }

    d.x80[0] = fp.m as u32;
    d.x80[1] = (fp.m >> 32) as u32;
    d.x80[2] = fp.exp as u32;
    if fp.sign == -1 {
        d.x80[2] |= 1 << LDBL_EXP_BITS;
    }
    0
}

// ── fpnum_parse_bnum (string.c:753-983) ───────────────────────────────────────

/// The parser proper. `_MSVCR_VER == 0`, so no inf/nan words and no hex floats — see the
/// module docs.
unsafe fn fpnum_parse_bnum<S: CharSrc>(
    s: &mut S,
    locinfo: *mut sys::threadlocaleinfostruct,
    ldouble: bool,
    b: &mut Bnum,
) -> FpNum {
    let mut found_digit = false;
    let mut found_dp = false;
    let mut found_sign = false;
    let mut e2: i32 = 0;
    let mut dp: i32 = 0;
    let mut sign: i32 = 1;
    let mut limb_digits: i32 = 0;
    let mut round = RoundZero;

    // The locale's radix character. `lconv->decimal_point` is a `char*`; the C compares
    // the (wide) input character against `*decimal_point` directly.
    let decimal_point = *(*(*locinfo).lconv).decimal_point as u8 as u16;

    let mut nch = s.get();
    if nch == b'-' as u16 {
        found_sign = true;
        sign = -1;
        nch = s.get();
    } else if nch == b'+' as u16 {
        found_sign = true;
        nch = s.get();
    }

    // [ _MSVCR_VER >= 140: the "infinity"/"nan" word matcher and the 0x hex-float
    //   hand-off to fpnum_parse16 would go here. Absent by contract — see module docs. ]

    while nch == b'0' as u16 {
        found_digit = true;
        nch = s.get();
    }

    b.b = 0;
    b.e = 1;
    b.data[0] = 0;

    // integer digits
    while nch >= b'0' as u16 && nch <= b'9' as u16 {
        found_digit = true;
        if limb_digits == LIMB_DIGITS {
            if b.idx(b.b - 1) == b.idx(b.e) {
                break; // buffer full
            }
            b.b -= 1;
            b.set(b.b, 0);
            limb_digits = 0;
        }

        let v = b.at(b.b) * 10 + (nch - b'0' as u16) as u32;
        b.set(b.b, v);
        limb_digits += 1;
        nch = s.get();
        dp += 1;
    }
    // ...and the ones that no longer fit: fold them into a sticky bit.
    while nch >= b'0' as u16 && nch <= b'9' as u16 {
        if nch != b'0' as u16 {
            let v = b.at(b.b) | 1;
            b.set(b.b, v);
        }
        nch = s.get();
        dp += 1;
    }

    if nch == decimal_point {
        found_dp = true;
        nch = s.get();
    }

    // skip leading '0'
    if nch == b'0' as u16 && limb_digits == 0 && b.b == 0 {
        found_digit = true;
        while nch == b'0' as u16 {
            nch = s.get();
            dp -= 1;
        }
    }

    // fraction digits
    while nch >= b'0' as u16 && nch <= b'9' as u16 {
        found_digit = true;
        if limb_digits == LIMB_DIGITS {
            if b.idx(b.b - 1) == b.idx(b.e) {
                break;
            }
            b.b -= 1;
            b.set(b.b, 0);
            limb_digits = 0;
        }

        let v = b.at(b.b) * 10 + (nch - b'0' as u16) as u32;
        b.set(b.b, v);
        limb_digits += 1;
        nch = s.get();
    }
    while nch >= b'0' as u16 && nch <= b'9' as u16 {
        if nch != b'0' as u16 {
            let v = b.at(b.b) | 1;
            b.set(b.b, v);
        }
        nch = s.get();
    }

    if !found_digit {
        if nch != WEOF {
            s.unget();
        }
        if found_dp {
            s.unget();
        }
        if found_sign {
            s.unget();
        }
        return fpnum(0, 0, 0, RoundZero);
    }

    // exponent. NOTE msvcrt accepts 'd'/'D' as well as 'e'/'E' — a Fortran-ism it has
    // never dropped.
    if nch == b'e' as u16 || nch == b'E' as u16 || nch == b'd' as u16 || nch == b'D' as u16 {
        let mut e: i32 = 0;
        let mut sg: i32 = 1;

        nch = s.get();
        if nch == b'-' as u16 {
            found_sign = true;
            sg = -1;
            nch = s.get();
        } else if nch == b'+' as u16 {
            found_sign = true;
            nch = s.get();
        } else {
            // NB: this CLEARS found_sign — so a mantissa sign is forgotten here, and the
            // "no exponent digits" rollback below ungets one character fewer. Verbatim.
            found_sign = false;
        }

        if nch >= b'0' as u16 && nch <= b'9' as u16 {
            while nch >= b'0' as u16 && nch <= b'9' as u16 {
                if e > i32::MAX / 10 || e * 10 > i32::MAX - (nch as i32) + b'0' as i32 {
                    e = i32::MAX;
                } else {
                    e = e * 10 + nch as i32 - b'0' as i32;
                }
                nch = s.get();
            }
            if nch != WEOF {
                s.unget();
            }
            e = e.wrapping_mul(sg);

            if e < 0 && dp < i32::MIN - e {
                dp = i32::MIN;
            } else if e > 0 && dp > i32::MAX - e {
                dp = i32::MAX;
            } else {
                dp += e;
            }
        } else {
            if nch != WEOF {
                s.unget();
            }
            if found_sign {
                s.unget();
            }
            s.unget();
        }
    } else if nch != WEOF {
        s.unget();
    }

    if b.at(b.e - 1) == 0 {
        return fpnum(sign, 0, 0, RoundZero);
    }

    // Fill last limb with 0 if needed
    if b.b + 1 != b.e {
        while limb_digits != LIMB_DIGITS {
            let v = b.at(b.b) * 10;
            b.set(b.b, v);
            limb_digits += 1;
        }
    }
    while b.idx(b.b) < b.idx(b.e) {
        if b.at(b.b) != 0 {
            break;
        }
        b.b += 1;
    }

    // move decimal point to limb boundary
    if limb_digits == dp && b.b == b.e - 1 {
        return fpnum(sign, 0, b.at(b.e - 1) as u64, RoundZero);
    }
    // C's `%` truncates toward zero, and so does Rust's — `(-1) % 9 == -1` in both.
    let mut off = (dp - limb_digits) % LIMB_DIGITS;
    if off < 0 {
        off += LIMB_DIGITS;
    }
    if off != 0 {
        b.mult(P10S[off as usize]);
    }

    if dp - 1
        > if ldouble {
            DBL80_MAX_10_EXP
        } else {
            DBL_MAX_10_EXP
        }
    {
        return fpnum(sign, i32::MAX, 1, RoundZero);
    }
    // Count part of exponent stored in denormalized mantissa.
    // Increase exponent range to handle subnormals.
    if dp - 1
        < if ldouble {
            DBL80_MIN_10_EXP
        } else {
            DBL_MIN_10_EXP - DBL_DIG - 18
        }
    {
        return fpnum(sign, i32::MIN, 1, RoundZero);
    }

    while dp > 3 * LIMB_DIGITS {
        if b.rshift(9) {
            dp -= LIMB_DIGITS;
        }
        e2 += 9;
    }
    while dp <= 2 * LIMB_DIGITS {
        if b.lshift(29) {
            dp += LIMB_DIGITS;
        }
        e2 -= 29;
    }
    // Make sure most significant mantissa bit will be set
    while b.at(b.e - 1) <= 9 {
        b.lshift(1);
        e2 -= 1;
    }
    let m = loop {
        match b.to_mant() {
            Some(m) => break m,
            None => {
                b.rshift(1);
                e2 += 1;
            }
        }
    };

    if b.e - 4 >= b.b && b.at(b.e - 4) != 0 {
        round = if b.at(b.e - 4) > LIMB_MAX / 2 {
            RoundUp
        } else if b.at(b.e - 4) == LIMB_MAX / 2 {
            RoundEven
        } else {
            RoundDown
        };
    }
    if round == RoundZero || round == RoundEven {
        // ⚠ VERBATIM, INCLUDING A WINE BUG THAT TURNS OUT TO BE INERT. The C is:
        //
        //     for(i=b->e-5; i>=b->b; i--) {
        //         if(!b->data[bnum_idx(b, b->b)]) continue;   /* <-- b->b, not i */
        //         if(round == FP_ROUND_EVEN) round = FP_ROUND_UP;
        //         else round = FP_ROUND_DOWN;
        //     }
        //
        // `i` is the loop variable, but the body inspects the LEAST-significant limb
        // (`b->b`) every iteration instead of the i'th. It reads like a plain bug — a
        // sticky-bit scan of the tail that in fact re-reads one limb N times.
        //
        // I transcribed it verbatim, then tried to CATCH it with a negative control
        // ("fix" b.b -> i and demand a DIVERGE). The control did not bite, which means one
        // of two things: either the corpus never reaches the state where the two spellings
        // differ, or the bug is genuinely inert. Instrumenting the reachability settled it
        // — it is INERT, and structurally so:
        //
        //   * The scan only matters when it runs (range b.b ..= e-5 non-empty, i.e. the
        //     mantissa spans >= 5 limbs) AND `round` is not already decided by limb e-4.
        //   * A `500000000` (== LIMB_MAX/2) limb — the ONLY value that leaves `round`
        //     at FP_ROUND_EVEN rather than UP/DOWN — is produced solely by `bnum_rshift(1)`
        //     on an odd limb, whose `rest = (LIMB_MAX>>1)*1` lands in a freshly-prepended
        //     BOTTOM limb. So FP_ROUND_EVEN implies that half-limb sits at `b`, i.e. at
        //     `e-4` only when `b == e-4`, and then the scan range `b ..= e-5` is EMPTY.
        //   * Whenever the scan body does run, `round` is therefore FP_ROUND_ZERO, and the
        //     strip loop above guarantees `data[b] != 0` (measured: 0 violations in 43,712
        //     parses), so buggy `data[b]` and correct `data[i]` both take the non-zero arm
        //     and both compute FP_ROUND_DOWN. Same answer, every time.
        //
        // Reproducing it is still correct — the oracle is msvcrt, not IEEE, and matching
        // its bytes is the whole contract — but the reason the bit-exact rows stay green is
        // not luck. Kept b.b to match; the tie-corpus in classify (gen_tie) exists to keep
        // hunting the discriminating state should a future bignum change ever create one.
        let mut i = b.e - 5;
        while i >= b.b {
            if b.at(b.b) == 0 {
                i -= 1;
                continue;
            }
            round = if round == RoundEven { RoundUp } else { RoundDown };
            i -= 1;
        }
    }

    fpnum(sign, e2, m, round)
}

/// `fpnum_parse` — pick the bignum precision, then parse. The C stack-allocates the
/// buffer here and hands a pointer down; we hand down a slice.
unsafe fn fpnum_parse<S: CharSrc>(
    s: &mut S,
    locinfo: *mut sys::threadlocaleinfostruct,
    ldouble: bool,
) -> FpNum {
    if !ldouble {
        let mut data = [0u32; BNUM_PREC64];
        let mut b = Bnum {
            b: 0,
            e: 1,
            size: BNUM_PREC64 as i32,
            data: &mut data,
        };
        fpnum_parse_bnum(s, locinfo, ldouble, &mut b)
    } else {
        let mut data = [0u32; BNUM_PREC80];
        let mut b = Bnum {
            b: 0,
            e: 1,
            size: BNUM_PREC80 as i32,
            data: &mut data,
        };
        fpnum_parse_bnum(s, locinfo, ldouble, &mut b)
    }
}

// ── the public entry points ───────────────────────────────────────────────────
//
// NOTE we deliberately do NOT `#[no_mangle]` the engine itself. `fpnum_parse` is a real
// cross-TU symbol in msvcrt (scanf's `%f` and the still-C `_atoldbl` path call it), and
// exporting a Rust one would drag `struct fpnum` across the language boundary by value
// for no benefit. The C engine stays, serves its C callers, and the two are graded
// against each other by `classify` through the public doors below.

use core::ffi::c_void;

/// `_CRT_FLOAT` / `_CRT_DOUBLE` (`sdk/include/crt/stdlib.h`) — the out-params of the
/// `_atoflt`/`_atodbl` family. Single-member structs, so no layout subtlety.
#[repr(C)]
pub struct CrtFloat {
    f: f32,
}
#[repr(C)]
pub struct CrtDouble {
    x: f64,
}

const OVERFLOW_: i32 = 3; // _OVERFLOW  (sdk/include/crt/math.h:41)
const UNDERFLOW_: i32 = 4; // _UNDERFLOW (sdk/include/crt/math.h:42)

/// `x.is_infinite()` without SSE.
///
/// The std `f64::is_infinite` / `f32::is_infinite` methods take their operand by value in
/// the Rust ABI, which hard-errors on no-SSE i386 exactly like a Rust-ABI `-> f64` return.
/// So read the operand's bits straight out of memory (a plain integer load — no float
/// register, no ABI) and test the IEEE exponent/mantissa. Same bit form as cnt-rust's
/// `core_rs/floatcvt.rs::is_finite`.
///
/// # Safety
/// `p` must point to a valid `f64`.
#[inline]
unsafe fn f64_is_inf(p: *const f64) -> bool {
    let bits = *(p as *const u64);
    (bits & 0x7FFF_FFFF_FFFF_FFFF) == 0x7FF0_0000_0000_0000
}
/// f32 twin of [`f64_is_inf`].
///
/// # Safety
/// `p` must point to a valid `f32`.
#[inline]
unsafe fn f32_is_inf(p: *const f32) -> bool {
    let bits = *(p as *const u32);
    (bits & 0x7FFF_FFFF) == 0x7F80_0000
}

/// The store-side twin of `f64::from_bits`, without SSE.
///
/// `f64::from_bits(u64) -> f64` returns an `f64` by value in the Rust ABI, so it hard-errors
/// on no-SSE i386 like every other Rust-ABI `-> f64`. But we only ever want the bits *in a
/// destination `f64`* — so write them straight into its storage as a `u64` (a plain integer
/// store, no float register). Bit-identical to `*d = f64::from_bits(bits)`.
///
/// # Safety
/// `d` must point to a writable `f64`.
#[inline]
unsafe fn store_bits(d: *mut f64, bits: u64) {
    *(d as *mut u64) = bits;
}

/// `strtod_helper` (string.c:1024). The shared body of the whole narrow family.
///
/// `perr` is the `_atoflt`/`_atodbl` out-of-band error channel: when it is non-NULL the
/// ERANGE goes there INSTEAD of to errno. And when it is NULL, note the `#if _MSVCR_VER
/// == 0` arm — this CRT **zeroes errno up front**, which the wide `_wcstod_l` does not.
/// That asymmetry is real and `classify` grades it.
/// `extern "C"`, NOT the default Rust ABI, ON PURPOSE. A Rust-ABI `-> f64` returns in
/// `xmm0` and so hard-errors on the no-SSE i386 target ("SIMD vector type `f64` … requires
/// the `sse` target feature"); an `extern "C"` `-> f64` returns in the x87 `ST(0)`
/// register, needs no SSE, and is the i386 C ABI. This is the same technique cnt-rust's
/// `cabi/strtod.rs` and `core_rs/floatcvt.rs::fl` use. On amd64/arm64 it is a no-op
/// (their C ABI returns f64 in xmm0/d0 anyway).
#[inline]
unsafe extern "C" fn strtod_helper(
    str_: *const u8,
    end: *mut *mut u8,
    locale: *mut c_void,
    perr: *mut i32,
) -> f64 {
    if !perr.is_null() {
        *perr = 0;
    } else {
        // #if _MSVCR_VER == 0
        *crate::_errno() = 0;
    }

    if !crate::check_pmt(!str_.is_null()) {
        if !end.is_null() {
            *end = core::ptr::null_mut();
        }
        return 0.0;
    }

    let li = crate::locinfo_of(locale);

    // ONE get_locinfo() for the whole call — the C re-resolves it per whitespace
    // character via `_isspace_l(c, locale)`; hoisting is behaviour-preserving and keeps
    // `hot_path` green. (See §9.76: this is the cost no perf ratio can see.)
    let mut p = str_;
    while crate::isctype_li(*p as i32, sys::_SPACE as i32, li) != 0 {
        p = p.add(1);
    }
    let beg = p;

    let mut src = StrSrc { p };
    let mut fp = fpnum_parse(&mut src, li, false);
    p = src.p;

    if !end.is_null() {
        *end = if p == beg { str_ as *mut u8 } else { p as *mut u8 };
    }

    let mut ret = 0.0f64;
    let err = fpnum_double(&mut fp, &mut ret);
    if !perr.is_null() {
        *perr = err;
    } else if err != 0 {
        *crate::_errno() = err;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _strtod_l(str_: *const u8, end: *mut *mut u8, locale: *mut c_void) -> f64 {
    strtod_helper(str_, end, locale, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn strtod(str_: *const u8, end: *mut *mut u8) -> f64 {
    _strtod_l(str_, end, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn atof(str_: *const u8) -> f64 {
    _strtod_l(str_, core::ptr::null_mut(), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _atof_l(str_: *const u8, locale: *mut c_void) -> f64 {
    _strtod_l(str_, core::ptr::null_mut(), locale)
}

/// `_atoflt_l`. Note it reports UNDERFLOW off the *float* (`value->f`, after the
/// double→float narrowing), not off the double — so a value that is fine as a double but
/// flushes to zero as a float is an `_UNDERFLOW`.
#[no_mangle]
pub unsafe extern "C" fn _atoflt_l(
    value: *mut CrtFloat,
    str_: *mut u8,
    locale: *mut c_void,
) -> i32 {
    let mut err: i32 = 0;
    let d = strtod_helper(str_, core::ptr::null_mut(), locale, &mut err);
    (*value).f = d as f32;

    if f32_is_inf(core::ptr::addr_of!((*value).f)) {
        return OVERFLOW_;
    }
    if (d != 0.0 || err != 0) && (*value).f > -f32::MIN_POSITIVE && (*value).f < f32::MIN_POSITIVE {
        return UNDERFLOW_;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _atodbl_l(
    value: *mut CrtDouble,
    str_: *mut u8,
    locale: *mut c_void,
) -> i32 {
    let mut err: i32 = 0;
    (*value).x = strtod_helper(str_, core::ptr::null_mut(), locale, &mut err);

    if f64_is_inf(core::ptr::addr_of!((*value).x)) {
        return OVERFLOW_;
    }
    if ((*value).x != 0.0 || err != 0)
        && (*value).x > -f64::MIN_POSITIVE
        && (*value).x < f64::MIN_POSITIVE
    {
        return UNDERFLOW_;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _atodbl(value: *mut CrtDouble, str_: *mut u8) -> i32 {
    _atodbl_l(value, str_, core::ptr::null_mut())
}

// ── the wide family (wcs.c:558-604) ───────────────────────────────────────────

/// `_wcstod_l`. Deliberately NOT a call to `strtod_helper`-with-a-wide-source: the C
/// wrote this one out separately and it differs — **there is no `*_errno() = 0` up
/// front.** Transcribe the file you are replacing, not the one next to it.
#[no_mangle]
pub unsafe extern "C" fn _wcstod_l(
    str_: *const u16,
    end: *mut *mut u16,
    locale: *mut c_void,
) -> f64 {
    if !crate::check_pmt(!str_.is_null()) {
        if !end.is_null() {
            *end = core::ptr::null_mut();
        }
        return 0.0;
    }

    let li = crate::locinfo_of(locale);

    let mut p = str_;
    while _iswspace_l_shim(*p as u32, locale) != 0 {
        p = p.add(1);
    }
    let beg = p;

    let mut src = WStrSrc { p };
    let mut fp = fpnum_parse(&mut src, li, false);
    p = src.p;

    if !end.is_null() {
        *end = if p == beg { str_ as *mut u16 } else { p as *mut u16 };
    }

    let mut ret = 0.0f64;
    let err = fpnum_double(&mut fp, &mut ret);
    if err != 0 {
        *crate::_errno() = err;
    }
    ret
}

/// `_iswspace_l` is already ours (wave 9) and ignores `locale` entirely, so this costs no
/// environment lookup per character and `hot_path` stays flat.
#[inline]
unsafe fn _iswspace_l_shim(wc: u32, locale: *mut c_void) -> i32 {
    crate::_iswspace_l(wc, locale)
}

#[no_mangle]
pub unsafe extern "C" fn wcstod(str_: *const u16, end: *mut *mut u16) -> f64 {
    _wcstod_l(str_, end, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wtof(str_: *const u16) -> f64 {
    _wcstod_l(str_, core::ptr::null_mut(), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wtof_l(str_: *const u16, locale: *mut c_void) -> f64 {
    _wcstod_l(str_, core::ptr::null_mut(), locale)
}

// ── the 80-bit family (string.c:1517-1587) ────────────────────────────────────

/// `MSVCRT__LDOUBLE` — ReactOS's **10-byte** x87 extended-precision struct
/// (`msvcrt.h:84`). Wine's own is 12 bytes (`WINE_BROKEN_LDOUBLE`, above) and ReactOS
/// calls that out in a comment: *"The wine version is wrong and causes buffer overruns"*.
///
/// So `__STRINGTOLD_L` computes into the 12-byte one and then `memcpy`s only
/// `sizeof(MSVCRT__LDOUBLE)` = **10** bytes out. That drops the high 2 bytes of `x80[2]`,
/// which are zero anyway — the x87 format is an 8-byte mantissa plus a 2-byte
/// sign/exponent. Getting this wrong writes 2 bytes past the caller's buffer, which is
/// precisely the overrun ReactOS patched.
///
/// A bare `[u8; 10]` is already size 10 / align 1, so plain `#[repr(C)]` matches the C
/// exactly — no `packed` needed (and `packed` would only invite unaligned-reference
/// errors on the field access below for nothing).
#[repr(C)]
pub struct MsvcrtLdouble {
    ld: [u8; 10],
}

/// `__STRINGTOLD_L`. Return code is its own little language: 0 ok, 1 underflow,
/// 2 overflow, 4 nothing-parsed.
#[no_mangle]
pub unsafe extern "C" fn __STRINGTOLD_L(
    value: *mut MsvcrtLdouble,
    endptr: *mut *mut u8,
    str_: *const u8,
    _flags: i32,
    locale: *mut c_void,
) -> i32 {
    // The C FIXME()s on non-zero flags and otherwise ignores them. So do we.
    let mut ret = 0i32;

    let li = crate::locinfo_of(locale);

    let mut p = str_;
    while crate::isctype_li(*p as i32, sys::_SPACE as i32, li) != 0 {
        p = p.add(1);
    }
    let beg = p;

    let mut src = StrSrc { p };
    let mut fp = fpnum_parse(&mut src, li, true);
    p = src.p;

    if !endptr.is_null() {
        *endptr = if p == beg { str_ as *mut u8 } else { p as *mut u8 };
    }
    if p == beg {
        ret = 4;
    }

    let mut v = WineBrokenLdouble::default();
    let err = fpnum_ldouble(&mut fp, &mut v);
    if err != 0 {
        ret = if v.x80[2] & 0x7fff != 0 { 2 } else { 1 };
    }

    // memcpy(value_, value, sizeof(*value_)) — 10 of the 12 bytes. See the type docs.
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!(v.x80) as *const u8,
        core::ptr::addr_of_mut!((*value).ld) as *mut u8,
        10,
    );
    ret
}

#[no_mangle]
pub unsafe extern "C" fn __STRINGTOLD(
    value: *mut MsvcrtLdouble,
    endptr: *mut *mut u8,
    str_: *const u8,
    flags: i32,
) -> i32 {
    __STRINGTOLD_L(value, endptr, str_, flags, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _atoldbl_l(
    value: *mut MsvcrtLdouble,
    str_: *mut u8,
    locale: *mut c_void,
) -> i32 {
    let mut endptr: *mut u8 = core::ptr::null_mut();
    match __STRINGTOLD_L(value, &mut endptr, str_, 0, locale) {
        1 => UNDERFLOW_,
        2 => OVERFLOW_,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn _atoldbl(value: *mut MsvcrtLdouble, str_: *mut u8) -> i32 {
    _atoldbl_l(value, str_, core::ptr::null_mut())
}
