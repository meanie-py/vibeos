//! `bnum.h` — msvcrt's base-10 **bignum**, shared by the float PARSER (`fpnum.rs`) and
//! the float FORMATTER (`printf.rs`).
//!
//! A real number with a fixed decimal point, held in a POWER-OF-TWO circular buffer of
//! 9-digit limbs. `b` is the least-significant limb index and `e` the most-significant
//! + 1 — and `b` *decreases* as the number grows, wrapping negative through `idx`'s mask
//! (`-1 & (size-1) == size-1` in two's complement, identical in C and Rust).
//!
//! Extracted verbatim from `fpnum.rs` (wave 13) so the printf `%e/%f/%g` formatter runs
//! the exact same limb arithmetic the parser does — the two must agree bit-for-bit, and
//! the only way to guarantee that is one implementation. The C achieves it the same way:
//! `bnum.h` is `#include`d by both `string.c` (parse) and `printf.h` (format).

pub(crate) const LIMB_DIGITS: i32 = 9;
pub(crate) const LIMB_MAX: u32 = 1_000_000_000; // 10^9

/// Data buffer size in DWORDs — enough to hold a 64-bit double. Power of two.
pub(crate) const BNUM_PREC64: usize = 128;
/// …and an 80-bit long double. Also a power of two.
pub(crate) const BNUM_PREC80: usize = 2048;

pub(crate) const P10S: [u32; 10] = [
    1, 10, 100, 1000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000,
];

/// `bnum.h::log2i` — floor(log2(x)) via a bit-scan. `x` must be non-zero.
#[inline]
pub(crate) fn log2i(x: u32) -> u32 {
    31 - x.leading_zeros()
}

/// `bnum.h::log10i` — floor(log10(x)). The C's exact form (`(log2i+1)*1233/4096`, then a
/// `p10s` correction) so the digit counts match byte-for-byte. `x` must be non-zero.
#[inline]
pub(crate) fn log10i(x: u32) -> u32 {
    let t = ((log2i(x) + 1) * 1233) / 4096;
    t - (x < P10S[t as usize]) as u32
}

/// `struct bnum`, with the flexible array turned into a borrowed slice. The caller
/// stack-allocates the buffer and the slice length carries `size`, so the two can never
/// disagree. Fields are `pub(crate)`: the formatter (`printf.rs`) seeds `data`/`b`/`e`
/// directly from a mantissa, exactly as `pf_output_fp` does in the C.
pub(crate) struct Bnum<'a> {
    /// least significant limb position (decreases as limbs are prepended)
    pub(crate) b: i32,
    /// most significant limb position + 1
    pub(crate) e: i32,
    /// buffer size in limbs — a power of two
    pub(crate) size: i32,
    pub(crate) data: &'a mut [u32],
}

impl Bnum<'_> {
    /// `bnum_idx` — the circular-buffer mask. `i` is routinely NEGATIVE (`b-1`); masking a
    /// negative i32 with `size-1` yields the wrapped slot, in Rust exactly as in C.
    #[inline]
    pub(crate) fn idx(&self, i: i32) -> usize {
        (i & (self.size - 1)) as usize
    }

    #[inline]
    pub(crate) fn at(&self, i: i32) -> u32 {
        self.data[self.idx(i)]
    }

    #[inline]
    pub(crate) fn set(&mut self, i: i32, v: u32) {
        let k = self.idx(i);
        self.data[k] = v;
    }

    /// `bnum_lshift`. Returns true if a new most-significant limb was added.
    ///
    /// Invariant (the C `assert`s it, compiled out under NDEBUG): `shift <= 29`.
    pub(crate) fn lshift(&mut self, shift: i32) -> bool {
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
    pub(crate) fn rshift(&mut self, shift: i32) -> bool {
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
    pub(crate) fn mult(&mut self, mult: u32) {
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

    /// `bnum_to_mant` — the top three limbs as a u64. `None` on overflow. (Parser-only,
    /// but lives here with the rest of the engine.)
    pub(crate) fn to_mant(&self) -> Option<u64> {
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
