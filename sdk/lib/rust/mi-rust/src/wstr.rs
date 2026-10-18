//! NUL-terminated UTF-16 strings: MI_Char is wchar_t here (MI_CHAR_TYPE 2), as on Windows.

use alloc::vec::Vec;

pub type WString = Vec<u16>;

/// Length of a NUL-terminated wide string, without the NUL.
pub unsafe fn wlen(s: *const u16) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// Copy a NUL-terminated wide string (NULL becomes None).
pub unsafe fn wdup(s: *const u16) -> Option<WString> {
    if s.is_null() {
        return None;
    }
    let n = wlen(s);
    let mut v = Vec::with_capacity(n + 1);
    v.extend_from_slice(core::slice::from_raw_parts(s, n));
    v.push(0);
    Some(v)
}

/// A NUL-terminated copy of `s`.
pub fn wfrom(s: &[u16]) -> WString {
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s);
    v.push(0);
    v
}

/// An ASCII literal as a NUL-terminated wide string.
pub fn wa(s: &str) -> WString {
    let mut v: WString = s.bytes().map(u16::from).collect();
    v.push(0);
    v
}

/// The characters of `s` without its NUL.
pub fn body(s: &[u16]) -> &[u16] {
    match s.iter().position(|&c| c == 0) {
        Some(n) => &s[..n],
        None => s,
    }
}

fn fold(c: u16) -> u16 {
    if (b'a' as u16..=b'z' as u16).contains(&c) {
        c - 32
    } else {
        c
    }
}

/// ASCII case-insensitive equality (CIM names compare that way).
pub fn ieq(a: &[u16], b: &[u16]) -> bool {
    let (a, b) = (body(a), body(b));
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| fold(x) == fold(y))
}

pub unsafe fn ieq_ptr(a: &[u16], b: *const u16) -> bool {
    if b.is_null() {
        return false;
    }
    ieq(a, core::slice::from_raw_parts(b, wlen(b)))
}

pub fn push_str(v: &mut WString, s: &str) {
    if v.last() == Some(&0) {
        v.pop();
    }
    v.extend(s.bytes().map(u16::from));
    v.push(0);
}

pub fn push_w(v: &mut WString, s: &[u16]) {
    if v.last() == Some(&0) {
        v.pop();
    }
    v.extend_from_slice(body(s));
    v.push(0);
}

pub fn push_dec(v: &mut WString, mut n: u64) {
    let mut digits = [0u16; 20];
    let mut i = digits.len();
    loop {
        i -= 1;
        digits[i] = b'0' as u16 + (n % 10) as u16;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    push_w(v, &digits[i..]);
}

/// Parse a decimal integer (optionally signed) from wide characters.
pub fn parse_i128(s: &[u16]) -> Option<i128> {
    let s = body(s);
    let (neg, digits) = match s.first() {
        Some(&c) if c == b'-' as u16 => (true, &s[1..]),
        Some(&c) if c == b'+' as u16 => (false, &s[1..]),
        _ => (false, s),
    };
    if digits.is_empty() {
        return None;
    }
    let mut n: i128 = 0;
    for &c in digits {
        if !(b'0' as u16..=b'9' as u16).contains(&c) {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((c - b'0' as u16) as i128)?;
    }
    Some(if neg { -n } else { n })
}
