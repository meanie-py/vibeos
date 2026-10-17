//! Arbitrary-width Python integer semantics for const scanner literals.
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Big {
    negative: bool,
    magnitude: String,
}
impl Ord for Big {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self
                .magnitude
                .len()
                .cmp(&other.magnitude.len())
                .then_with(|| self.magnitude.cmp(&other.magnitude)),
            (true, true) => other
                .magnitude
                .len()
                .cmp(&self.magnitude.len())
                .then_with(|| other.magnitude.cmp(&self.magnitude)),
        }
    }
}
impl PartialOrd for Big {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Big {
    pub fn zero() -> Self {
        Self {
            negative: false,
            magnitude: "0".into(),
        }
    }
    pub fn is_negative(&self) -> bool {
        self.negative
    }
    pub fn parse(raw: &str) -> Option<Self> {
        let clean = raw.replace('_', "");
        let (negative, body) = if let Some(x) = clean.strip_prefix('-') {
            (true, x)
        } else {
            (false, clean.as_str())
        };
        let magnitude = crate::integers::unsigned_base0(body).ok()?;
        Some(Self {
            negative: negative && magnitude != "0",
            magnitude,
        })
    }
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut d = vec![0u8];
        for b in bytes {
            let mut carry = *b as u16;
            for digit in &mut d {
                let n = *digit as u16 * 256 + carry;
                *digit = (n % 10) as u8;
                carry = n / 10;
            }
            while carry > 0 {
                d.push((carry % 10) as u8);
                carry /= 10;
            }
        }
        let magnitude = d
            .into_iter()
            .rev()
            .map(|x| char::from(b'0' + x))
            .collect::<String>();
        Self {
            negative: false,
            magnitude,
        }
    }
    fn add_one(s: &str) -> String {
        let mut d = s.bytes().map(|b| b - b'0').collect::<Vec<_>>();
        let mut i = d.len();
        loop {
            if i == 0 {
                d.insert(0, 1);
                break;
            }
            i -= 1;
            if d[i] < 9 {
                d[i] += 1;
                break;
            }
            d[i] = 0;
        }
        d.into_iter().map(|x| char::from(b'0' + x)).collect()
    }
    fn sub_one(s: &str) -> String {
        if s == "0" {
            return "0".into();
        }
        let mut d = s.bytes().map(|b| b - b'0').collect::<Vec<_>>();
        let mut i = d.len();
        loop {
            i -= 1;
            if d[i] > 0 {
                d[i] -= 1;
                break;
            }
            d[i] = 9;
        }
        while d.len() > 1 && d[0] == 0 {
            d.remove(0);
        }
        d.into_iter().map(|x| char::from(b'0' + x)).collect()
    }
    pub fn bitnot(self) -> Self {
        if self.negative {
            let magnitude = Self::sub_one(&self.magnitude);
            Self {
                negative: false,
                magnitude,
            }
        } else {
            Self {
                negative: true,
                magnitude: Self::add_one(&self.magnitude),
            }
        }
    }
    fn rem_bits(&self, bits: u32) -> u64 {
        let modulus = 1u128 << bits;
        let mut rem = 0u128;
        for b in self.magnitude.bytes() {
            rem = (rem * 10 + (b - b'0') as u128) % modulus;
        }
        if self.negative && rem != 0 {
            (modulus - rem) as u64
        } else {
            rem as u64
        }
    }
    pub fn same_bits(&self, o: &Self, bits: u32) -> bool {
        self.rem_bits(bits) == o.rem_bits(bits)
    }
    pub fn small_nonnegative(&self, max: u32) -> bool {
        !self.negative && self.magnitude.parse::<u32>().is_ok_and(|x| x <= max)
    }
    pub fn render(&self) -> String {
        if self.negative {
            format!("!0x{}", Self::decimal_hex(&Self::sub_one(&self.magnitude)))
        } else {
            format!("0x{}", Self::decimal_hex(&self.magnitude))
        }
    }
    fn decimal_hex(s: &str) -> String {
        if s == "0" {
            return "0".into();
        }
        let mut digits = s.bytes().map(|b| (b - b'0') as u16).collect::<Vec<_>>();
        let mut hex = Vec::new();
        while digits.iter().any(|x| *x != 0) {
            let mut carry = 0u16;
            for d in &mut digits {
                let n = carry * 10 + *d;
                *d = n / 16;
                carry = n % 16;
            }
            hex.push(
                char::from_digit(carry as u32, 16)
                    .unwrap()
                    .to_ascii_uppercase(),
            );
            while digits.len() > 1 && digits[0] == 0 {
                digits.remove(0);
            }
        }
        hex.iter().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handles_unbounded_values_and_masks() {
        let x = Big::parse("999999999999999999999999999999999999999999").unwrap();
        assert_eq!(x.render(), "0xB7ABC627050305ADF14A3D9E3FFFFFFFFFF");
        assert_eq!(x.rem_bits(32), 0xFFFF_FFFF);
    }
    #[test]
    fn complements_match_python() {
        assert_eq!(Big::parse("255").unwrap().bitnot().render(), "!0xFF");
        assert_eq!(Big::parse("-256").unwrap().bitnot().render(), "0xFF");
    }
}
