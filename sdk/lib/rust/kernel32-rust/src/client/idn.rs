//! kernel32 IdnToAscii / IdnToUnicode / IdnToNameprepUnicode (RFC 3490 IDNA + RFC 3492
//! punycode), following Wine's ntdll `RtlIdnTo*` for the return protocol and error mapping.
//!
//! DIVERGENCE FROM WINDOWS/WINE (nameprep is approximated): Windows/Wine run RFC 3454
//! nameprep with NFKC normalization and the unassigned-code-point table (A.1) read from NLS
//! data. We have neither NFKC tables (RtlNormalizeString is a stub) nor A.1. So for input
//! containing any non-ASCII character this implementation:
//!   * maps RFC 3454 table B.1 ("map to nothing": U+00AD U+034F U+1806 U+180B-D U+200B-D
//!     U+2060 U+FE00-F U+FEFF) to nothing;
//!   * folds case with ntdll RtlDowncaseUnicodeChar (approximates B.2; BMP only) plus the
//!     two common multi-char folds U+00DF -> "ss" and U+0130 -> "i" U+0307;
//!   * maps the IDNA label separators U+3002, U+FF0E, U+FF61 to '.';
//!   * rejects the RFC 3454 C.1.2 C.2 C.3 C.4 C.5 C.6 C.7 C.8 C.9 prohibited ranges and lone
//!     surrogates with ERROR_INVALID_NAME.
//!   It does NOT apply NFKC: input needing compatibility mapping (fullwidth letters,
//!   ligatures, ...) or canonical recomposition is encoded un-normalized, and unassigned code
//!   points are never rejected (IDN_ALLOW_UNASSIGNED is accepted but has no effect). The
//!   bidi check (RFC 3454 section 6) is not performed. Pure-ASCII input skips nameprep
//!   (RFC 3490 section 4.1 step 2) and is exact. IDN_USE_STD3_ASCII_RULES restricts only the
//!   ASCII range to letters/digits/hyphen (RFC 3490 step 3), and forbids leading/trailing
//!   hyphens.
//!
//! Return protocol (as Wine): the output includes the terminating NUL iff the input did
//! (cchUnicodeChar == -1 counts the NUL); cchDest == 0 returns the required length; a too
//! small buffer gives ERROR_INSUFFICIENT_BUFFER; invalid names ERROR_INVALID_NAME; flags
//! other than IDN_ALLOW_UNASSIGNED|IDN_USE_STD3_ASCII_RULES ERROR_INVALID_FLAGS; NULL source
//! or length < -1 or bad destination ERROR_INVALID_PARAMETER. All of these return 0.
use ntdll_sys::nt::*;
use crate::prelude::*;

const ERROR_INSUFFICIENT_BUFFER_: u32 = 122;
const ERROR_INVALID_NAME_: u32 = 123;
const ERROR_INVALID_PARAMETER_: u32 = 87;
const ERROR_INVALID_FLAGS_: u32 = 1004;

const IDN_ALLOW_UNASSIGNED_: u32 = 0x01;
const IDN_USE_STD3_ASCII_RULES_: u32 = 0x02;

/// Capacity of the nameprep / ToASCII working buffers (UTF-16 units), as Wine.
const MAX_NAME: usize = 256;
/// Capacity for the code points of one label.
const MAX_CPS: usize = 256;

// Punycode parameters (RFC 3492 section 5).
const BASE: i64 = 36;
const TMIN: i64 = 1;
const TMAX: i64 = 26;
const SKEW: i64 = 38;
const DAMP: i64 = 700;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum IdnErr {
    InvalidName,
    BufferTooSmall,
}

type R<T> = Result<T, IdnErr>;

// ---------------------------------------------------------------------------------------
// UTF-16 helpers

/// Decode one code point at the start of `s`; returns (cp, units). Lone surrogates are
/// RFC 3454 C.5 (prohibited).
fn get_utf16(s: &[u16]) -> R<(u32, usize)> {
    let c = s[0] as u32;
    if (0xd800..0xdc00).contains(&c) {
        if s.len() > 1 && (0xdc00..0xe000).contains(&(s[1] as u32)) {
            return Ok((0x10000 + ((c - 0xd800) << 10) + (s[1] as u32 - 0xdc00), 2));
        }
        return Err(IdnErr::InvalidName);
    }
    if (0xdc00..0xe000).contains(&c) {
        return Err(IdnErr::InvalidName);
    }
    Ok((c, 1))
}

fn encode_utf16(cp: u32, out: &mut [u16; 2]) -> usize {
    if cp >= 0x10000 {
        let v = cp - 0x10000;
        out[0] = 0xd800 + (v >> 10) as u16;
        out[1] = 0xdc00 + (v & 0x3ff) as u16;
        2
    } else {
        out[0] = cp as u16;
        1
    }
}

/// Bounded output sink; `count_only` measures without storing (cchDest == 0).
struct Out<'a> {
    buf: &'a mut [u16],
    count_only: bool,
    n: usize,
}

impl<'a> Out<'a> {
    fn put(&mut self, w: u16) -> R<()> {
        if !self.count_only {
            if self.n >= self.buf.len() {
                return Err(IdnErr::BufferTooSmall);
            }
            self.buf[self.n] = w;
        }
        self.n += 1;
        Ok(())
    }
    fn put_cp(&mut self, cp: u32) -> R<()> {
        let mut u = [0u16; 2];
        let k = encode_utf16(cp, &mut u);
        for i in 0..k {
            self.put(u[i])?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------
// RFC 3454 tables (hardcoded subsets)

/// Table B.1: mapped to nothing.
fn map_to_nothing(c: u32) -> bool {
    matches!(c, 0x00ad | 0x034f | 0x1806 | 0x180b..=0x180d | 0x200b..=0x200d | 0x2060
        | 0xfe00..=0xfe0f | 0xfeff)
}

/// Prohibited output: C.1.2, C.2.1, C.2.2, C.3, C.4, C.6, C.7, C.8, C.9 (surrogates, C.5,
/// are rejected while decoding UTF-16; they cannot reach here as code points).
fn prohibited(c: u32) -> bool {
    match c {
        0x0000..=0x001f | 0x007f => true,                  // C.2.1
        0x0080..=0x009f => true,                           // C.2.2
        0x00a0 | 0x1680 | 0x2000..=0x200b | 0x202f | 0x205f | 0x3000 => true, // C.1.2
        0x06dd | 0x070f | 0x180e | 0x200c | 0x200d | 0x2028 | 0x2029 => true, // C.2.2
        0x2060..=0x2063 | 0x206a..=0x206f | 0xfeff => true, // C.2.2 / C.8
        0xfff9..=0xfffd => true,                           // C.2.2 / C.6
        0x1d173..=0x1d17a => true,                         // C.2.2
        0x0340 | 0x0341 | 0x200e | 0x200f | 0x202a..=0x202e => true, // C.8
        0x2ff0..=0x2ffb => true,                           // C.7
        0xe000..=0xf8ff | 0xf0000..=0xffffd | 0x100000..=0x10fffd => true, // C.3
        0xfdd0..=0xfdef => true,                           // C.4
        0xe0001 | 0xe0020..=0xe007f => true,               // C.9
        0xd800..=0xdfff => true,                           // C.5
        _ => (c & 0xfffe) == 0xfffe && c <= 0x10ffff,      // C.4 plane ends
    }
}

fn is_dot(c: u32) -> bool {
    matches!(c, 0x2e | 0x3002 | 0xff0e | 0xff61)
}

fn is_ldh(c: u32) -> bool {
    (c >= 'a' as u32 && c <= 'z' as u32)
        || (c >= 'A' as u32 && c <= 'Z' as u32)
        || (c >= '0' as u32 && c <= '9' as u32)
        || c == '-' as u32
}

// ---------------------------------------------------------------------------------------
// Nameprep (approximate, see module docs)

/// `src` is the full source (including a trailing NUL if the caller counted it). Output in
/// `out`; returns its length in units (with the NUL if present).
fn nameprep(flags: u32, src: &[u16], lower: fn(u16) -> u16, out: &mut [u16; MAX_NAME]) -> R<usize> {
    let n = src.len();
    if n > MAX_NAME {
        return Err(IdnErr::InvalidName);
    }
    let ascii_only = src
        .iter()
        .enumerate()
        .all(|(i, &c)| (c >= 0x20 && c < 0x7f) || (c == 0 && i == n - 1));
    let mut len = 0usize;
    if ascii_only {
        out[..n].copy_from_slice(src);
        len = n;
    } else {
        let mut i = 0;
        while i < n {
            if src[i] == 0 && i != n - 1 {
                return Err(IdnErr::InvalidName);
            }
            let (cp, l) = get_utf16(&src[i..])?;
            i += l;
            if map_to_nothing(cp) {
                continue;
            }
            // Fold to at most two code points.
            let mut mapped = [cp, 0];
            let mut cnt = 1;
            if is_dot(cp) {
                mapped[0] = 0x2e;
            } else if cp == 0xdf {
                mapped = [0x73, 0x73];
                cnt = 2;
            } else if cp == 0x130 {
                mapped = [0x69, 0x307];
                cnt = 2;
            } else if cp < 0x10000 {
                mapped[0] = lower(cp as u16) as u32;
            }
            for k in 0..cnt {
                let m = mapped[k];
                if prohibited(m) && !(m == 0 && i == n) {
                    return Err(IdnErr::InvalidName);
                }
                let mut u = [0u16; 2];
                let ul = encode_utf16(m, &mut u);
                if len + ul > MAX_NAME {
                    return Err(IdnErr::InvalidName);
                }
                out[len..len + ul].copy_from_slice(&u[..ul]);
                len += ul;
            }
        }
    }

    // Label validation (Wine RtlIdnToNameprepUnicode).
    let std3 = flags & IDN_USE_STD3_ASCII_RULES_ != 0;
    let mut i = 0usize;
    let mut start = 0usize;
    while i < len {
        let (ch, l) = get_utf16(&out[i..len])?;
        if ch == 0 {
            break;
        }
        if ch == 0x2e {
            if start == i {
                return Err(IdnErr::InvalidName);
            }
            if i - start > 63 {
                return Err(IdnErr::InvalidName);
            }
            if std3 && (out[start] == 0x2d || out[i - 1] == 0x2d) {
                return Err(IdnErr::InvalidName);
            }
            start = i + 1;
            i += l;
            continue;
        }
        if std3 && ch < 0x80 && !is_ldh(ch) {
            return Err(IdnErr::InvalidName);
        }
        i += l;
    }
    if i == 0 || i - start > 63 {
        return Err(IdnErr::InvalidName);
    }
    if std3 && (out[start] == 0x2d || out[i - 1] == 0x2d) {
        return Err(IdnErr::InvalidName);
    }
    Ok(len)
}

// ---------------------------------------------------------------------------------------
// Punycode

fn adapt(mut delta: i64, numpoints: i64, first: bool) -> i64 {
    delta /= if first { DAMP } else { 2 };
    delta += delta / numpoints;
    let mut k = 0;
    while delta > ((BASE - TMIN) * TMAX) / 2 {
        delta /= BASE - TMIN;
        k += BASE;
    }
    k + ((BASE - TMIN + 1) * delta) / (delta + SKEW)
}

fn digit_char(d: i64) -> u16 {
    if d <= 25 { b'a' as u16 + d as u16 } else { b'0' as u16 + (d - 26) as u16 }
}

/// RFC 3492 encode of one label (no "xn--" prefix): appends basic code points, '-' if any,
/// then the deltas, to `res[*out..]`.
fn puny_encode(cps: &[u32], res: &mut [u16; MAX_NAME], out: &mut usize) -> R<()> {
    let b = cps.iter().filter(|&&c| c < 0x80).count();
    if *out + b + 1 > MAX_NAME {
        return Err(IdnErr::InvalidName);
    }
    for &c in cps {
        if c < 0x80 {
            res[*out] = c as u16;
            *out += 1;
        }
    }
    if b > 0 {
        res[*out] = b'-' as u16;
        *out += 1;
    }
    let (mut n, mut bias, mut delta, mut h) = (0x80i64, 72i64, 0i64, b);
    while h < cps.len() {
        let mut m = 0x10ffffi64;
        for &c in cps {
            if (c as i64) >= n && m > c as i64 {
                m = c as i64;
            }
        }
        delta += (m - n) * (h as i64 + 1);
        n = m;
        for &c in cps {
            let c = c as i64;
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = if k <= bias { TMIN } else if k >= bias + TMAX { TMAX } else { k - bias };
                    let disp = if q < t { q } else { t + (q - t) % (BASE - t) };
                    if *out >= MAX_NAME {
                        return Err(IdnErr::InvalidName);
                    }
                    res[*out] = digit_char(disp);
                    *out += 1;
                    if q < t {
                        break;
                    }
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                bias = adapt(delta, h as i64 + 1, h == b);
                delta = 0;
                h += 1;
            } else if c < n {
                delta += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Ok(())
}

/// Wine's check_invalid_chars on a label's code points (no combining-class tables: the
/// ZWJ/ZWNJ-after-virama exception is moot because C.2.2 prohibits both here).
fn invalid_label(flags: u32, cps: &[u32]) -> bool {
    for &c in cps {
        if prohibited(c) {
            return true;
        }
        if matches!(c, 0x2260 | 0x226e | 0x226f) && flags & IDN_USE_STD3_ASCII_RULES_ != 0 {
            return true;
        }
    }
    flags & IDN_USE_STD3_ASCII_RULES_ != 0
        && !cps.is_empty()
        && (cps[0] == 0x2d || cps[cps.len() - 1] == 0x2d)
}

/// RtlIdnToAscii equivalent. Output has the NUL iff `src` ended in one.
fn to_ascii(flags: u32, src: &[u16], lower: fn(u16) -> u16, res: &mut [u16; MAX_NAME])
    -> R<usize>
{
    let mut norm = [0u16; MAX_NAME];
    let normlen = nameprep(flags, src, lower, &mut norm)?;
    let mut out = 0usize;
    let mut start = 0usize;
    while start < normlen {
        let out_label = out;
        let mut cps = [0u32; 64];
        let mut buflen = 0usize;
        let mut b = 0usize;
        let mut i = start;
        while i < normlen {
            let (ch, l) = get_utf16(&norm[i..normlen])?;
            if ch == 0 || ch == 0x2e {
                break;
            }
            if ch < 0x80 {
                b += 1;
            }
            if buflen >= cps.len() {
                return Err(IdnErr::InvalidName);
            }
            cps[buflen] = ch;
            buflen += 1;
            i += l;
        }
        let end = i;

        if b == end - start {
            // all-ASCII label (plus its delimiter) is copied verbatim
            if end < normlen {
                b += 1;
            }
            if out + b > MAX_NAME {
                return Err(IdnErr::InvalidName);
            }
            res[out..out + b].copy_from_slice(&norm[start..start + b]);
            out += b;
            start = end + 1;
            continue;
        }

        let cps = &cps[..buflen];
        if buflen >= 4 && cps[2] == 0x2d && cps[3] == 0x2d {
            return Err(IdnErr::InvalidName);
        }
        if invalid_label(flags, cps) {
            return Err(IdnErr::InvalidName);
        }
        if out + 4 > MAX_NAME {
            return Err(IdnErr::InvalidName);
        }
        for &c in b"xn--" {
            res[out] = c as u16;
            out += 1;
        }
        puny_encode(cps, res, &mut out)?;
        if out - out_label > 63 {
            return Err(IdnErr::InvalidName);
        }
        if end < normlen {
            if out >= MAX_NAME {
                return Err(IdnErr::InvalidName);
            }
            res[out] = norm[end];
            out += 1;
        }
        start = end + 1;
    }
    Ok(out)
}

/// RtlIdnToUnicode equivalent. Writes into `o`; the returned length is `o.n`.
fn to_unicode(flags: u32, src: &[u16], o: &mut Out) -> R<()> {
    let srclen = src.len();
    let std3 = flags & IDN_USE_STD3_ASCII_RULES_ != 0;
    let mut start = 0usize;
    while start < srclen {
        let (mut n, mut bias, mut pos) = (0x80i64, 72i64, 0i64);
        let mut delim = 0usize;
        let mut ch = 0u16;
        let out_label = o.n;
        let mut i = start;
        while i < srclen {
            ch = src[i];
            if ch > 0x7f || (i != srclen - 1 && ch == 0) {
                return Err(IdnErr::InvalidName);
            }
            if ch == 0 || ch == b'.' as u16 {
                break;
            }
            if ch == b'-' as u16 {
                delim = i;
            }
            if std3 && !is_ldh(ch as u32) {
                return Err(IdnErr::InvalidName);
            }
            i += 1;
        }
        let mut end = i;
        // last label may be empty
        if start == end && ch != 0 {
            return Err(IdnErr::InvalidName);
        }

        let lc = |c: u16| c | 0x20;
        if end - start < 4
            || lc(src[start]) != b'x' as u16
            || lc(src[start + 1]) != b'n' as u16
            || src[start + 2] != b'-' as u16
            || src[start + 3] != b'-' as u16
        {
            if end - start > 63 {
                return Err(IdnErr::InvalidName);
            }
            if std3 && end > start && (src[start] == b'-' as u16 || src[end - 1] == b'-' as u16) {
                return Err(IdnErr::InvalidName);
            }
            if end < srclen {
                end += 1;
            }
            for k in start..end {
                o.put(src[k])?;
            }
            start = end;
            continue;
        }

        if delim == start + 3 {
            delim += 1;
        }
        let mut cps = [0u32; MAX_CPS];
        let mut buflen = 0usize;
        i = start + 4;
        while i < delim {
            if buflen >= MAX_CPS - 1 {
                return Err(IdnErr::InvalidName);
            }
            cps[buflen] = src[i] as u32;
            buflen += 1;
            i += 1;
        }
        if buflen > 0 {
            i += 1;
        }
        while i < end {
            let old_pos = pos;
            let mut w = 1i64;
            let mut k = BASE;
            loop {
                if i >= end {
                    return Err(IdnErr::InvalidName);
                }
                let c = src[i];
                i += 1;
                let digit = match c {
                    0x61..=0x7a => (c - 0x61) as i64,
                    0x41..=0x5a => (c - 0x41) as i64,
                    0x30..=0x39 => (c - 0x30) as i64 + 26,
                    _ => return Err(IdnErr::InvalidName),
                };
                pos += digit * w;
                if pos > (1i64 << 40) {
                    return Err(IdnErr::InvalidName);
                }
                let t = if k <= bias { TMIN } else if k >= bias + TMAX { TMAX } else { k - bias };
                if digit < t {
                    break;
                }
                w *= BASE - t;
                if w > (1i64 << 40) {
                    return Err(IdnErr::InvalidName);
                }
                k += BASE;
            }
            bias = adapt(pos - old_pos, buflen as i64 + 1, old_pos == 0);
            n += pos / (buflen as i64 + 1);
            pos %= buflen as i64 + 1;
            if n > 0x10ffff || buflen >= MAX_CPS - 1 {
                return Err(IdnErr::InvalidName);
            }
            let p = pos as usize;
            cps.copy_within(p..buflen, p + 1);
            cps[p] = n as u32;
            pos += 1;
            buflen += 1;
        }
        if invalid_label(flags, &cps[..buflen]) {
            return Err(IdnErr::InvalidName);
        }
        for k in 0..buflen {
            o.put_cp(cps[k])?;
        }
        if o.n - out_label > 63 {
            return Err(IdnErr::InvalidName);
        }
        if end < srclen {
            o.put(src[end])?;
        }
        start = end + 1;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------
// Win32 entry points

fn lower_char(c: u16) -> u16 {
    // SAFETY: RtlDowncaseUnicodeChar is a pure table lookup.
    unsafe { RtlDowncaseUnicodeChar(c) }
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    ToAscii,
    ToUnicode,
    Nameprep,
}

unsafe fn idn_common(flags: u32, src: *const u16, srclen: i32, dst: *mut u16, dstlen: i32, op: Op) -> i32 {
    if flags & !(IDN_ALLOW_UNASSIGNED_ | IDN_USE_STD3_ASCII_RULES_) != 0 {
        SetLastError(ERROR_INVALID_FLAGS_);
        return 0;
    }
    if src.is_null() || srclen < -1 || dstlen < 0 || (dstlen > 0 && dst.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return 0;
    }
    let n = if srclen == -1 {
        let mut k = 0usize;
        while *src.add(k) != 0 {
            k += 1;
        }
        k + 1
    } else {
        srclen as usize
    };
    let s = core::slice::from_raw_parts(src, n);

    let result: R<usize> = match op {
        Op::ToUnicode => {
            let out_slice: &mut [u16] = if dstlen > 0 {
                core::slice::from_raw_parts_mut(dst, dstlen as usize)
            } else {
                &mut []
            };
            let mut o = Out { buf: out_slice, count_only: dstlen == 0, n: 0 };
            to_unicode(flags, s, &mut o).map(|_| o.n)
        }
        _ => {
            let mut tmp = [0u16; MAX_NAME];
            let r = if op == Op::ToAscii {
                to_ascii(flags, s, lower_char, &mut tmp)
            } else {
                nameprep(flags, s, lower_char, &mut tmp)
            };
            match r {
                Ok(len) if dstlen == 0 => Ok(len),
                Ok(len) if len > dstlen as usize => Err(IdnErr::BufferTooSmall),
                Ok(len) => {
                    core::ptr::copy_nonoverlapping(tmp.as_ptr(), dst, len);
                    Ok(len)
                }
                Err(e) => Err(e),
            }
        }
    };
    match result {
        Ok(len) => len as i32,
        Err(IdnErr::InvalidName) => {
            SetLastError(ERROR_INVALID_NAME_);
            0
        }
        Err(IdnErr::BufferTooSmall) => {
            SetLastError(ERROR_INSUFFICIENT_BUFFER_);
            0
        }
    }
}

/// IdnToAscii: RFC 3490 ToASCII over a whole domain name. Last error: see module docs.
#[no_mangle]
pub unsafe extern "system" fn IdnToAscii(
    dwFlags: u32,
    lpUnicodeCharStr: *const u16,
    cchUnicodeChar: i32,
    lpASCIICharStr: *mut u16,
    cchASCIIChar: i32,
) -> i32 {
    idn_common(dwFlags, lpUnicodeCharStr, cchUnicodeChar, lpASCIICharStr, cchASCIIChar, Op::ToAscii)
}

/// IdnToUnicode: RFC 3490 ToUnicode over a whole domain name.
#[no_mangle]
pub unsafe extern "system" fn IdnToUnicode(
    dwFlags: u32,
    lpASCIICharStr: *const u16,
    cchASCIIChar: i32,
    lpUnicodeCharStr: *mut u16,
    cchUnicodeChar: i32,
) -> i32 {
    idn_common(dwFlags, lpASCIICharStr, cchASCIIChar, lpUnicodeCharStr, cchUnicodeChar, Op::ToUnicode)
}

/// IdnToNameprepUnicode: the (approximate) nameprep step alone.
#[no_mangle]
pub unsafe extern "system" fn IdnToNameprepUnicode(
    dwFlags: u32,
    lpUnicodeCharStr: *const u16,
    cchUnicodeChar: i32,
    lpNameprepCharStr: *mut u16,
    cchNameprepChar: i32,
) -> i32 {
    idn_common(dwFlags, lpUnicodeCharStr, cchUnicodeChar, lpNameprepCharStr, cchNameprepChar, Op::Nameprep)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::String;
    use std::vec::Vec;

    fn lw(c: u16) -> u16 {
        match char::from_u32(c as u32) {
            Some(ch) => ch.to_lowercase().next().map(|l| l as u32).filter(|&l| l < 0x10000).unwrap_or(c as u32) as u16,
            None => c,
        }
    }
    fn w(s: &str) -> Vec<u16> { s.encode_utf16().collect() }
    fn wn(s: &str) -> Vec<u16> { let mut v = w(s); v.push(0); v }
    fn ascii(flags: u32, s: &[u16]) -> Result<String, IdnErr> {
        let mut r = [0u16; MAX_NAME];
        let n = to_ascii(flags, s, lw, &mut r)?;
        Ok(String::from_utf16(&r[..n]).unwrap())
    }
    fn uni(flags: u32, s: &[u16]) -> Result<Vec<u16>, IdnErr> {
        let mut buf = [0u16; 1024];
        let mut o = Out { buf: &mut buf, count_only: false, n: 0 };
        to_unicode(flags, s, &mut o)?;
        let n = o.n;
        Ok(buf[..n].to_vec())
    }
    fn prep(flags: u32, s: &[u16]) -> Result<Vec<u16>, IdnErr> {
        let mut r = [0u16; MAX_NAME];
        let n = nameprep(flags, s, lw, &mut r)?;
        Ok(r[..n].to_vec())
    }
    const E: IdnErr = IdnErr::InvalidName;

    #[test]
    fn rfc3492_samples() {
        let enc = |cps: &[u32]| {
            let mut r = [0u16; MAX_NAME];
            let mut o = 0;
            puny_encode(cps, &mut r, &mut o).unwrap();
            String::from_utf16(&r[..o]).unwrap()
        };
        assert_eq!(enc(&[0x644,0x64a,0x647,0x645,0x627,0x628,0x62a,0x643,0x644,0x645,0x648,0x634,0x639,0x631,0x628,0x64a,0x61f]), "egbpdaj6bu4bxfgehfvwxn");
        assert_eq!(enc(&[0x4ed6,0x4eec,0x4e3a,0x4ec0,0x4e48,0x4e0d,0x8bf4,0x4e2d,0x6587]), "ihqwcrb4cv8a8dqg056pqjye");
        assert_eq!(enc(&[0x33,0x5e74,0x42,0x7d44,0x91d1,0x516b,0x5148,0x751f]), "3B-ww4c5e180e575a65lsy2b");
        let all: Vec<u32> = "-> $1.00 <-".chars().map(|c| c as u32).collect();
        assert_eq!(enc(&all), "-> $1.00 <--");
        // decode round trip
        let u = uni(0, &w("xn--egbpdaj6bu4bxfgehfvwxn")).unwrap();
        assert_eq!(u.len(), 17);
        assert_eq!(u[0], 0x644);
        let u = uni(0, &w("xn--ihqwcrb4cv8a8dqg056pqjye")).unwrap();
        assert_eq!(u, [0x4ed6,0x4eec,0x4e3a,0x4ec0,0x4e48,0x4e0d,0x8bf4,0x4e2d,0x6587]);
    }

    #[test]
    fn wine_nameprep() {
        assert_eq!(prep(0, &wn("test")).unwrap(), wn("test"));
        assert_eq!(prep(0, &[b't' as u16, 0, b'e' as u16, 0]), Err(E));
        assert_eq!(prep(0, &wn("T")).unwrap(), wn("T"));
        assert_eq!(prep(0, &[0]), Err(E));
        assert_eq!(prep(0, &[]), Err(E));
        assert_eq!(prep(0, &wn(" -/[]")).unwrap(), wn(" -/[]"));
        assert!(prep(2, &w("a-a")).is_ok());
        assert_eq!(prep(2, &w("aa-")), Err(E));
        assert_eq!(prep(0, &wn("a..a")[..4]), Err(E));
        assert_eq!(prep(0, &w("a.")).unwrap(), w("a."));
        assert_eq!(prep(0, &w("T.\u{105}.A")).unwrap(), w("t.\u{105}.a"));
        assert_eq!(prep(0, &w("T.*.A")).unwrap(), w("T.*.A"));
        assert_eq!(prep(0, &w("X\u{ff0e}.Z")), Err(E));
        assert!(prep(0, &w(&"a".repeat(63))).is_ok());
        assert_eq!(prep(0, &w(&"a".repeat(64))), Err(E));
        // vector 8 minus NFKC-only parts (U+37A, U+6A U+30C compose)
        assert_eq!(prep(0, &w("T\u{df}\u{130}\u{143}")).unwrap(), w("tssi\u{307}\u{144}"));
        // B.1 mapped away, ASCII control/private use prohibited
        assert_eq!(prep(0, &w("t\u{ad}\u{34f}\u{180b}\u{180c}\u{180d}\u{200b}\u{200c}\u{200d}")).unwrap(), w("t"));
        assert_eq!(prep(0, &w("a\u{e111}z")), Err(E));
        assert_eq!(prep(0, &w("a\u{0}\u{105}")), Err(E));
        assert_eq!(prep(0, &w("a\u{1}\u{105}")), Err(E));
        assert_eq!(prep(0, &[0xd800, 0x61]), Err(E));
    }

    #[test]
    fn wine_to_ascii() {
        let ok = |f, s: &[u16], o: &str| assert_eq!(ascii(f, s).unwrap(), o, "{:?}", s);
        ok(0, &wn("Test"), "Test\0");
        ok(0, &w("Te\u{17c}st"), "xn--test-cbb");
        ok(0, &w("te\u{105}st.te\u{17c}st"), "xn--test-cta.xn--test-cbb");
        ok(0, &wn("\u{105}."), "xn--2da.\0");
        ok(0, &w("http://t\u{106}"), "xn--http://t-78a");
        ok(0, &wn("\u{4e3a}\u{8bf4}\u{4e0d}\u{4ed6}\u{5011}\u{10d}\u{11b}\u{305c}\u{306a}"), "xn--bea2a1631avbav44tyha32b91egs2t\0");
        ok(0, &w(&"a".repeat(63)), &"a".repeat(63));
        assert_eq!(ascii(0, &w(&"a".repeat(64))), Err(E));
        ok(0, &wn("\u{e4}z123456789012345678901234567890123456789012345678901234"), "xn--z123456789012345678901234567890123456789012345678901234-9te\0");
        let mut v: Vec<u16> = std::vec![0xd803, 0xde78, 0x46b5, b'-' as u16, 0xa861, b'.' as u16, 0x2e87, 0];
        ok(0, &v, "xn----bm3an932a1l5d.xn--xvj\0");
        ok(0, &wn("\u{6ef}\u{6ef}"), "xn--cmba\0");
        // U+FF61 is a label separator; leading '-' allowed without STD3
        v = std::vec![b'-' as u16, 0x7e1, 0xff61, 0x2184, 0];
        ok(0, &v, "xn----8cd.xn--r5g\0");
        assert_eq!(ascii(2, &v), Err(E));
        assert_eq!(ascii(0, &w("ab--\u{105}")), Err(E));
        assert_eq!(ascii(0, &w("a\u{3002}\u{3002}b")), Err(E));
    }

    #[test]
    fn wine_to_unicode() {
        let ok = |f, s: &[u16], o: &[u16]| assert_eq!(uni(f, s).unwrap(), o);
        ok(0, &w("Tes."), &w("Tes."));
        assert_eq!(uni(0, &w("\u{105}")), Err(E));
        ok(0, &w("xn--4dbcagdahymbxekheh6e0a7fei0b"), &w("\u{5dc}\u{5de}\u{5d4}\u{5d4}\u{5dd}\u{5e4}\u{5e9}\u{5d5}\u{5d8}\u{5dc}\u{5d0}\u{5de}\u{5d3}\u{5d1}\u{5e8}\u{5d9}\u{5dd}\u{5e2}\u{5d1}\u{5e8}\u{5d9}\u{5ea}"));
        ok(0, &w("test.xn--kda9ag5e9jnfsj.xn--pz-fna"), &w("test.\u{105}\u{119}\u{15b}\u{107}\u{142}\u{f3}\u{17c}.p\u{119}z"));
        ok(0, &w(&"a".repeat(63)), &w(&"a".repeat(63)));
        assert_eq!(uni(0, &w(&"a".repeat(64))), Err(E));
        let mut v = w("xn----bm3an932a1l5d.xn--xvj");
        v.push(0);
        let mut e: Vec<u16> = std::vec![0xd803, 0xde78, 0x46b5, b'-' as u16, 0xa861, b'.' as u16, 0x2e87, 0];
        ok(0, &v, &e);
        e = w("\u{e4}z123456789012345678901234567890123456789012345678901234");
        ok(0, &w("xn--z123456789012345678901234567890123456789012345678901234-9te"), &e);
        ok(0, &wn("foo.bar"), &wn("foo.bar"));
        ok(0, &wn("d.xn----dha"), &wn("d.\u{fc}-"));
        // too-small destination
        let mut small = [0u16; 2];
        let mut o = Out { buf: &mut small, count_only: false, n: 0 };
        assert_eq!(to_unicode(0, &w("foo.bar"), &mut o), Err(IdnErr::BufferTooSmall));
        // count-only mode
        let mut o = Out { buf: &mut [], count_only: true, n: 0 };
        to_unicode(0, &w("xn--ihqwcrb4cv8a8dqg056pqjye"), &mut o).unwrap();
        assert_eq!(o.n, 9);
    }
}
