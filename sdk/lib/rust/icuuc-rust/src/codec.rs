//! One-character decode/encode steps. No state; the streaming logic lives in engine.rs.
use crate::names::{Entry, Kind};
use crate::plat::{Platform, Plat as P};
use crate::{U_ILLEGAL_CHAR_FOUND, U_INVALID_CHAR_FOUND, U_TRUNCATED_CHAR_FOUND};

pub enum Dec {
    Char { used: usize, units: [u16; 2], n: usize },
    /// Valid prefix of a longer sequence and no more input (and not flushing).
    More,
    Bad { used: usize, code: i32 },
}

pub enum Enc {
    Bytes { buf: [u8; 8], n: usize },
    /// Unmappable (U_INVALID_CHAR_FOUND / UCNV_UNASSIGNED).
    Unmapped,
}

fn incomplete(v: &[u8], flush: bool) -> Dec {
    if flush { Dec::Bad { used: v.len(), code: U_TRUNCATED_CHAR_FOUND } } else { Dec::More }
}

fn one(u: u32) -> Dec {
    if u >= 0x10000 {
        let c = u - 0x10000;
        Dec::Char { used: 0, units: [0xD800 + (c >> 10) as u16, 0xDC00 + (c & 0x3FF) as u16], n: 2 }
    } else {
        Dec::Char { used: 0, units: [u as u16, 0], n: 1 }
    }
}

fn with_used(d: Dec, used: usize) -> Dec {
    match d {
        Dec::Char { units, n, .. } => Dec::Char { used, units, n },
        o => o,
    }
}

/// `v` is non-empty and holds min(4, bytes available) bytes.
pub fn decode(e: &Entry, v: &[u8], flush: bool) -> Dec {
    let b0 = v[0];
    match e.kind {
        Kind::Ascii => if b0 < 0x80 { with_used(one(b0 as u32), 1) } else { Dec::Bad { used: 1, code: U_ILLEGAL_CHAR_FOUND } },
        Kind::Latin1 => with_used(one(b0 as u32), 1),
        Kind::Utf8 => utf8(v, flush),
        Kind::Utf16Le | Kind::Utf16Be => utf16(v, flush, e.kind == Kind::Utf16Be),
        Kind::Cp => cp_decode(e.cp, v, flush),
    }
}

fn utf8(v: &[u8], flush: bool) -> Dec {
    let b0 = v[0];
    if b0 < 0x80 {
        return with_used(one(b0 as u32), 1);
    }
    let (need, lo, hi) = match b0 {
        0xC2..=0xDF => (2, 0x80, 0xBF),
        0xE0 => (3, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE | 0xEF => (3, 0x80, 0xBF),
        0xED => (3, 0x80, 0x9F),
        0xF0 => (4, 0x90, 0xBF),
        0xF1..=0xF3 => (4, 0x80, 0xBF),
        0xF4 => (4, 0x80, 0x8F),
        _ => return Dec::Bad { used: 1, code: U_ILLEGAL_CHAR_FOUND },
    };
    let mut cp = (b0 as u32) & (0xFF >> (need + 1));
    for i in 1..need {
        if i >= v.len() {
            return incomplete(v, flush);
        }
        let b = v[i];
        let (l, h) = if i == 1 { (lo, hi) } else { (0x80, 0xBF) };
        if b < l || b > h {
            return Dec::Bad { used: i, code: U_ILLEGAL_CHAR_FOUND };
        }
        cp = (cp << 6) | (b as u32 & 0x3F);
    }
    with_used(one(cp), need)
}

fn utf16(v: &[u8], flush: bool, be: bool) -> Dec {
    let rd = |i: usize| -> u16 {
        if be { ((v[i] as u16) << 8) | v[i + 1] as u16 } else { ((v[i + 1] as u16) << 8) | v[i] as u16 }
    };
    if v.len() < 2 {
        return incomplete(v, flush);
    }
    let u = rd(0);
    if (0xD800..0xDC00).contains(&u) {
        if v.len() < 4 {
            return incomplete(v, flush);
        }
        let l = rd(2);
        if !(0xDC00..0xE000).contains(&l) {
            return Dec::Bad { used: 2, code: U_ILLEGAL_CHAR_FOUND };
        }
        return Dec::Char { used: 4, units: [u, l], n: 2 };
    }
    if (0xDC00..0xE000).contains(&u) {
        return Dec::Bad { used: 2, code: U_ILLEGAL_CHAR_FOUND };
    }
    Dec::Char { used: 2, units: [u, 0], n: 1 }
}

/// Lead-byte test with overrides for code pages whose lead bytes kernel32 does not report fully.
fn is_lead(cp: u32, b: u8) -> bool {
    match cp {
        51932 => (0x8E..=0xFE).contains(&b),
        54936 => (0x81..=0xFE).contains(&b),
        _ => P::is_lead(cp, b),
    }
}

fn seq_need(cp: u32, b0: u8, b1: u8) -> usize {
    if cp == 54936 && (0x30..=0x39).contains(&b1) {
        4
    } else if cp == 51932 && b0 == 0x8F {
        3
    } else {
        2
    }
}

fn cp_decode(cp: u32, v: &[u8], flush: bool) -> Dec {
    let b0 = v[0];
    let mut w = [0u16; 4];
    if !is_lead(cp, b0) {
        return match P::mb_to_wc(cp, &v[..1], &mut w) {
            Some(n) if n >= 1 => Dec::Char { used: 1, units: [w[0], if n > 1 { w[1] } else { 0 }], n: n.min(2) },
            _ => Dec::Bad { used: 1, code: U_INVALID_CHAR_FOUND },
        };
    }
    if v.len() < 2 {
        return incomplete(v, flush);
    }
    let need = seq_need(cp, b0, v[1]);
    if v.len() < need {
        return incomplete(v, flush);
    }
    match P::mb_to_wc(cp, &v[..need], &mut w) {
        Some(n) if n >= 1 => Dec::Char { used: need, units: [w[0], if n > 1 { w[1] } else { 0 }], n: n.min(2) },
        // A second byte below 0x40 cannot be a trail byte: report only the lead as illegal and
        // let the next byte be decoded on its own (this is what ICU's MBCS converter does).
        _ if need == 2 && v[1] < 0x40 => Dec::Bad { used: 1, code: U_ILLEGAL_CHAR_FOUND },
        _ => Dec::Bad { used: need, code: U_INVALID_CHAR_FOUND },
    }
}

pub fn encode(e: &Entry, cp: u32) -> Enc {
    let mut buf = [0u8; 8];
    match e.kind {
        Kind::Ascii => if cp < 0x80 { buf[0] = cp as u8; Enc::Bytes { buf, n: 1 } } else { Enc::Unmapped },
        Kind::Latin1 => if cp < 0x100 { buf[0] = cp as u8; Enc::Bytes { buf, n: 1 } } else { Enc::Unmapped },
        Kind::Utf8 => {
            let n = if cp < 0x80 {
                buf[0] = cp as u8; 1
            } else if cp < 0x800 {
                buf[0] = 0xC0 | (cp >> 6) as u8; buf[1] = 0x80 | (cp & 0x3F) as u8; 2
            } else if cp < 0x10000 {
                buf[0] = 0xE0 | (cp >> 12) as u8; buf[1] = 0x80 | ((cp >> 6) & 0x3F) as u8;
                buf[2] = 0x80 | (cp & 0x3F) as u8; 3
            } else {
                buf[0] = 0xF0 | (cp >> 18) as u8; buf[1] = 0x80 | ((cp >> 12) & 0x3F) as u8;
                buf[2] = 0x80 | ((cp >> 6) & 0x3F) as u8; buf[3] = 0x80 | (cp & 0x3F) as u8; 4
            };
            Enc::Bytes { buf, n }
        }
        Kind::Utf16Le | Kind::Utf16Be => {
            let be = e.kind == Kind::Utf16Be;
            let (u, n) = split(cp);
            let mut k = 0;
            for i in 0..n {
                let (hi, lo) = ((u[i] >> 8) as u8, u[i] as u8);
                if be { buf[k] = hi; buf[k + 1] = lo; } else { buf[k] = lo; buf[k + 1] = hi; }
                k += 2;
            }
            Enc::Bytes { buf, n: k }
        }
        Kind::Cp => {
            let (u, n) = split(cp);
            match P::wc_to_mb(e.cp, &u[..n], &mut buf) {
                Some(k) if k >= 1 && k <= 8 => Enc::Bytes { buf, n: k },
                _ => Enc::Unmapped,
            }
        }
    }
}

fn split(cp: u32) -> ([u16; 2], usize) {
    if cp >= 0x10000 {
        let c = cp - 0x10000;
        ([0xD800 + (c >> 10) as u16, 0xDC00 + (c & 0x3FF) as u16], 2)
    } else {
        ([cp as u16, 0], 1)
    }
}

/// Substitution bytes used by the built-in fromUnicode substitute callback.
/// ICU uses 0x1A (SUB) for ASCII/Latin-1 and U+FFFD for the Unicode encodings; for Windows code
/// pages we use the code page's own DefaultChar (usually '?').
pub fn sub_bytes(e: &Entry) -> ([u8; 4], usize) {
    match e.kind {
        Kind::Ascii | Kind::Latin1 => ([0x1A, 0, 0, 0], 1),
        Kind::Utf8 => ([0xEF, 0xBF, 0xBD, 0], 3),
        Kind::Utf16Le => ([0xFD, 0xFF, 0, 0], 2),
        Kind::Utf16Be => ([0xFF, 0xFD, 0, 0], 2),
        Kind::Cp => ([P::cp_info(e.cp).map_or(b'?', |i| i.default_char), 0, 0, 0], 1),
    }
}
