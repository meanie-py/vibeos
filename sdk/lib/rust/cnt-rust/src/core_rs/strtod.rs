//! `strtod`/`atof` core — faithful port of ReactOS libcntpr
//! `string/strtod.c` (the DJ Delorie parser). Pure, `no_std`, no
//! `unsafe`, no raw pointers: scans a `&[u8]` and returns the parsed
//! value plus the byte offset where parsing stopped (the C `*sret`).
//!
//! **Documented i386 deviation (deliberate, "same thing on every arch"
//! directive).** The genuine C accumulates `r`/`d` in `long double`,
//! which is 80-bit x87 on i386 but `== double == f64` on amd64/arm64.
//! Rust stable has no f80, so this port accumulates in `f64` on all 3
//! arches. That makes it **bit-identical to the amd64/arm64 C** (and to
//! a host C reference built `-mlong-double-64`, which cnt-difftest pins)
//! but it can differ from the OLD i386 C by up to a last-ULP on
//! hard-to-round decimal inputs. The algorithm was never IEEE
//! correctly-rounded anyway (a naive `r = r*10 + d`, then `r *= 10^e`),
//! so this trades i386's incidental f80 extra precision for one uniform
//! Rust impl. The exact DJ Delorie control flow — including its quirks —
//! is preserved (see the sign notes below).

/// Parse a leading floating-point number from `s` (faithful to
/// `strtod.c`). Returns `(value, consumed)` where `consumed` is the
/// offset of the first unconsumed byte (what the C writes to `*sret`).
///
/// Quirks preserved verbatim from the C:
/// * `*sret` (here `consumed`) is set AFTER skipping leading
///   whitespace + the optional sign (NOT the original start), matching
///   DJ Delorie — not the ISO "set to nptr on failure" behavior.
/// * the no-digits case returns `0.0` with sign discarded;
/// * the `e`/`E`-with-no-exponent-digit case returns `r` **without**
///   applying the sign (a genuine C quirk: `return r;`, not `r*sign`).
pub fn strtod_bytes(s: &[u8]) -> (f64, usize) {
    let n = s.len();
    let mut r = 0.0_f64; // result
    let mut e: i32 = 0; // exponent magnitude
    let mut sign: i32 = 1;
    let mut esign: i32 = 1;
    let mut flags: u32 = 0;
    let mut i = 0usize;

    // `while ((*s == ' ') || (*s == '\t')) s++;`
    while i < n && (s[i] == b' ' || s[i] == b'\t') {
        i += 1;
    }

    // sign
    if i < n && s[i] == b'+' {
        i += 1;
    } else if i < n && s[i] == b'-' {
        sign = -1;
        i += 1;
    }

    // integer digits: `r *= 10.0; r += *s - '0';`
    while i < n && s[i].is_ascii_digit() {
        flags |= 1;
        r *= 10.0;
        r += (s[i] - b'0') as f64;
        i += 1;
    }

    // fraction: `d = 0.1L; r += d * digit; d *= 0.1L;`
    if i < n && s[i] == b'.' {
        let mut d = 0.1_f64;
        i += 1;
        while i < n && s[i].is_ascii_digit() {
            flags |= 2;
            r += d * ((s[i] - b'0') as f64);
            i += 1;
            d *= 0.1;
        }
    }

    // `if (flags == 0) { *sret = s; return 0; }`
    if flags == 0 {
        return (0.0, i);
    }

    // exponent
    if i < n && (s[i] == b'e' || s[i] == b'E') {
        i += 1;
        if i < n && s[i] == b'+' {
            i += 1;
        } else if i < n && s[i] == b'-' {
            i += 1;
            esign = -1;
        }
        // `if ((*s < '0') || (*s > '9')) { *sret = s; return r; }`
        // NOTE: returns `r` WITHOUT the sign — faithful to the C.
        if !(i < n && s[i].is_ascii_digit()) {
            return (r, i);
        }
        while i < n && s[i].is_ascii_digit() {
            e *= 10;
            e += (s[i] - b'0') as i32;
            i += 1;
        }
    }

    // apply exponent: `for (i=1;i<=e;i++) r *= 0.1L | 10.0;`
    if esign < 0 {
        let mut k = 1;
        while k <= e {
            r *= 0.1;
            k += 1;
        }
    } else {
        let mut k = 1;
        while k <= e {
            r *= 10.0;
            k += 1;
        }
    }

    // `return r * sign;`
    (r * sign as f64, i)
}
