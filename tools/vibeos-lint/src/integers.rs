use regex::Regex;
use std::sync::OnceLock;

// Python accepts unbounded decimal integers, including Unicode decimal digits.
pub fn decimal(raw: &str) -> Result<String, String> {
    static DIGIT: OnceLock<Regex> = OnceLock::new();
    let digit = DIGIT.get_or_init(|| Regex::new(r"^\p{Nd}$").unwrap());
    let mut result = String::new();
    for ch in raw.chars() {
        let value = if ch.is_ascii_digit() {
            ch as u32 - '0' as u32
        } else {
            let mut buf = [0; 4];
            if !digit.is_match(ch.encode_utf8(&mut buf)) {
                return Err(format!("invalid decimal alias suffix {raw}"));
            }
            // Unicode Nd runs contain consecutive groups of ten decimal digits.
            let mut start = ch as u32;
            while start > 0
                && char::from_u32(start - 1)
                    .is_some_and(|prior| digit.is_match(prior.encode_utf8(&mut buf)))
            {
                start -= 1;
            }
            (ch as u32 - start) % 10
        };
        if value != 0 || !result.is_empty() {
            result.push(char::from(b'0' + value as u8));
        }
    }
    Ok(if result.is_empty() {
        "0".into()
    } else {
        result
    })
}

/// Python int(raw, 0) for unsigned literal captures, rendered without a width limit.
pub fn unsigned_base0(raw: &str) -> Result<String, String> {
    let clean = raw.replace('_', "");
    if let Some(hex) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        if hex.is_empty() {
            return Err(format!("invalid integer literal {raw}"));
        }
        let mut digits = vec![0u8];
        for ch in hex.chars() {
            let mut carry = ch
                .to_digit(16)
                .ok_or_else(|| format!("invalid integer literal {raw}"))?
                as u16;
            for digit in &mut digits {
                let value = *digit as u16 * 16 + carry;
                *digit = (value % 10) as u8;
                carry = value / 10;
            }
            while carry > 0 {
                digits.push((carry % 10) as u8);
                carry /= 10;
            }
        }
        Ok(digits
            .into_iter()
            .rev()
            .map(|n| char::from(b'0' + n))
            .collect())
    } else {
        if clean.is_empty() {
            return Err(format!("invalid integer literal {raw}"));
        }
        let value = decimal(&clean)?;
        if value != "0" && decimal(&clean.chars().next().unwrap().to_string())? == "0" {
            return Err(format!("invalid base-0 integer literal {raw}"));
        }
        Ok(value)
    }
}
