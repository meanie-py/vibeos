//! Arbitrary-width signed integer operations used by verify_tables' Python evaluator.
use crate::const_values::Big;
use std::cmp::Ordering;

fn parts(v: &Big) -> (bool, Vec<u8>) {
    let raw = v.decimal();
    let (neg, digits) = raw
        .strip_prefix('-')
        .map_or((false, raw.as_str()), |x| (true, x));
    (neg, digits.bytes().map(|x| x - b'0').collect())
}
fn from_parts(negative: bool, digits: &[u8]) -> Big {
    let mut s: String = digits.iter().map(|d| char::from(b'0' + *d)).collect();
    let trimmed = s.trim_start_matches('0').to_owned();
    s = if trimmed.is_empty() {
        "0".into()
    } else {
        trimmed
    };
    if negative && s != "0" {
        s.insert(0, '-');
    }
    Big::parse(&s).unwrap()
}
fn cmp_digits(a: &[u8], b: &[u8]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
fn add_abs(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let (mut i, mut carry) = (0usize, 0u8);
    while i < a.len() || i < b.len() || carry != 0 {
        let n = a.get(a.len().wrapping_sub(i + 1)).copied().unwrap_or(0)
            + b.get(b.len().wrapping_sub(i + 1)).copied().unwrap_or(0)
            + carry;
        out.push(n % 10);
        carry = n / 10;
        i += 1;
    }
    out.reverse();
    out
}
fn sub_abs(a: &[u8], b: &[u8]) -> Vec<u8> {
    debug_assert!(cmp_digits(a, b) != Ordering::Less);
    let mut out = Vec::with_capacity(a.len());
    let (mut i, mut borrow) = (0usize, 0i8);
    while i < a.len() {
        let ai = a[a.len() - i - 1] as i8 - borrow;
        let bi = b.get(b.len().wrapping_sub(i + 1)).copied().unwrap_or(0) as i8;
        if ai < bi {
            out.push((ai + 10 - bi) as u8);
            borrow = 1;
        } else {
            out.push((ai - bi) as u8);
            borrow = 0;
        }
        i += 1;
    }
    while out.len() > 1 && out.last() == Some(&0) {
        out.pop();
    }
    out.reverse();
    out
}
pub fn add(a: &Big, b: &Big) -> Big {
    let (an, ad) = parts(a);
    let (bn, bd) = parts(b);
    if an == bn {
        return from_parts(an, &add_abs(&ad, &bd));
    }
    match cmp_digits(&ad, &bd) {
        Ordering::Equal => Big::zero(),
        Ordering::Greater => from_parts(an, &sub_abs(&ad, &bd)),
        Ordering::Less => from_parts(bn, &sub_abs(&bd, &ad)),
    }
}
pub fn neg(a: &Big) -> Big {
    let (n, d) = parts(a);
    from_parts(!n, &d)
}
pub fn sub(a: &Big, b: &Big) -> Big {
    add(a, &neg(b))
}
pub fn mul(a: &Big, b: &Big) -> Big {
    let (an, ad) = parts(a);
    let (bn, bd) = parts(b);
    let mut out = vec![0u32; ad.len() + bd.len()];
    for (i, x) in ad.iter().rev().enumerate() {
        for (j, y) in bd.iter().rev().enumerate() {
            let at = out.len() - 1 - i - j;
            out[at] += (*x as u32) * (*y as u32);
        }
    }
    for i in (1..out.len()).rev() {
        let carry = out[i] / 10;
        out[i] %= 10;
        out[i - 1] += carry;
    }
    let digits = out.into_iter().map(|x| (x % 10) as u8).collect::<Vec<_>>();
    from_parts(an ^ bn, &digits)
}

fn div_small(d: &mut Vec<u8>, divisor: u8) -> u8 {
    let mut rem = 0u16;
    for x in d.iter_mut() {
        let n = rem * 10 + *x as u16;
        *x = (n / divisor as u16) as u8;
        rem = n % divisor as u16;
    }
    while d.len() > 1 && d[0] == 0 {
        d.remove(0);
    }
    rem as u8
}
fn decimal_to_bits(digits: &[u8]) -> Vec<bool> {
    if digits.iter().all(|d| *d == 0) {
        return vec![false];
    }
    let mut d = digits.to_vec();
    let mut bits = Vec::new();
    while d.iter().any(|x| *x != 0) {
        bits.push(div_small(&mut d, 2) != 0);
    }
    bits
}
fn bits_to_decimal(bits: &[bool]) -> Vec<u8> {
    let mut digits = vec![0u8];
    for bit in bits.iter().rev() {
        let mut carry = *bit as u8;
        for d in digits.iter_mut().rev() {
            let n = *d * 2 + carry;
            *d = n % 10;
            carry = n / 10;
        }
        if carry != 0 {
            digits.insert(0, carry);
        }
    }
    digits
}
fn twos_bits(negative: bool, magnitude: &[bool], width: usize) -> Vec<bool> {
    let mut bits = vec![false; width];
    for (i, b) in magnitude.iter().enumerate().take(width) {
        bits[i] = *b;
    }
    if negative {
        for bit in &mut bits {
            *bit = !*bit;
        }
        let mut carry = true;
        for bit in &mut bits {
            let old = *bit;
            *bit = old ^ carry;
            carry &= old;
        }
    }
    bits
}
pub fn bitor(a: &Big, b: &Big) -> Big {
    let (an, ad) = parts(a);
    let (bn, bd) = parts(b);
    let ab = decimal_to_bits(&ad);
    let bb = decimal_to_bits(&bd);
    let width = ab.len().max(bb.len()) + 1;
    let aa = twos_bits(an, &ab, width);
    let bb = twos_bits(bn, &bb, width);
    let mut out = aa
        .iter()
        .zip(bb.iter())
        .map(|(x, y)| *x | *y)
        .collect::<Vec<_>>();
    let negative = out[width - 1];
    if negative {
        for bit in &mut out {
            *bit = !*bit;
        }
        let mut carry = true;
        for bit in &mut out {
            let old = *bit;
            *bit = old ^ carry;
            carry &= old;
        }
    }
    from_parts(negative, &bits_to_decimal(&out))
}
pub fn shl(a: &Big, count: &Big) -> Result<Big, String> {
    let raw = count.decimal();
    let n = raw
        .parse::<usize>()
        .map_err(|_| format!("invalid or negative shift count: {raw}"))?;
    let (negative, mut digits) = parts(a);
    let reserve = n / 3 + 1;
    digits
        .try_reserve(reserve)
        .map_err(|e| format!("cannot allocate left-shift result for count {n}: {e}"))?;
    for _ in 0..n {
        let mut carry = 0u8;
        for d in digits.iter_mut().rev() {
            let v = *d * 2 + carry;
            *d = v % 10;
            carry = v / 10;
        }
        if carry != 0 {
            digits.insert(0, carry);
        }
    }
    Ok(from_parts(negative, &digits))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn b(s: &str) -> Big {
        Big::parse(s).unwrap()
    }
    #[test]
    fn exact_signed_arithmetic_and_bitwise_or() {
        assert_eq!(
            add(&b("999999999999999999999"), &b("1")).decimal(),
            "1000000000000000000000"
        );
        assert_eq!(sub(&b("1"), &b("2")).decimal(), "-1");
        assert_eq!(
            mul(&b("123456789012345"), &b("987654321")).decimal(),
            "121932631124827861592745"
        );
        assert_eq!(bitor(&b("-2"), &b("1")).decimal(), "-1");
        assert_eq!(bitor(&b("-5"), &b("6")).decimal(), "-1");
        assert_eq!(
            bitor(&b("18446744073709551616"), &b("3")).decimal(),
            "18446744073709551619"
        );
    }
    #[test]
    fn exact_large_left_shift_and_invalid_shift() {
        assert_eq!(
            shl(&b("3"), &b("70")).unwrap().decimal(),
            "3541774862152233910272"
        );
        assert!(shl(&b("1"), &b("-1")).is_err());
    }
}
