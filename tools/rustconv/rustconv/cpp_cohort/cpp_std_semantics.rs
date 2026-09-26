//! Bounded byte semantics for C++ std::stol and std::string::substr.
//!
//! This is a host pilot component, not an automatic rewrite. Callers must
//! preserve the original argument expressions and own the target's long width.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericError {
    InvalidArgument,
    OutOfRange,
    UnsupportedInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubstrError {
    OutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StolAbi {
    pub long_bits: u32,
    pub binary_prefix: bool,
}

fn digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'z' => Some((byte - b'a') as u32 + 10),
        b'A'..=b'Z' => Some((byte - b'A') as u32 + 10),
        _ => None,
    }
}

fn c_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Model std::stol(text, &pos, base) in the C locale for ASCII bytes.
/// Supports base 0 or 2..=36 and a 32- or 64-bit C++ long. `binary_prefix`
/// must be obtained from the target C runtime; some strtol implementations
/// accept 0b while others do not. NUL terminates
/// conversion, as std::stol forwards `text.c_str()` to strtol.
pub fn stol_bytes(text: &[u8], base: u32, abi: StolAbi) -> Result<(i64, usize), NumericError> {
    let long_bits = abi.long_bits;
    if !(base == 0 || (2..=36).contains(&base)) || !(long_bits == 32 || long_bits == 64) {
        return Err(NumericError::UnsupportedInput);
    }
    if text.iter().any(|byte| *byte >= 128) {
        return Err(NumericError::UnsupportedInput);
    }
    let end = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
    let mut at = 0usize;
    while at < end && c_space(text[at]) {
        at += 1;
    }
    let negative = if at < end && matches!(text[at], b'+' | b'-') {
        let sign = text[at] == b'-';
        at += 1;
        sign
    } else {
        false
    };
    let mut radix = base;
    if radix == 0 {
        radix = if at < end && text[at] == b'0' { 8 } else { 10 };
    }
    if radix == 16
        && at + 2 < end
        && text[at] == b'0'
        && matches!(text[at + 1], b'x' | b'X')
        && digit(text[at + 2]).is_some_and(|d| d < 16)
    {
        at += 2;
    } else if base == 0
        && at + 2 < end
        && text[at] == b'0'
        && matches!(text[at + 1], b'x' | b'X')
        && digit(text[at + 2]).is_some_and(|d| d < 16)
    {
        radix = 16;
        at += 2;
    } else if abi.binary_prefix
        && (base == 0 || base == 2)
        && at + 2 < end
        && text[at] == b'0'
        && matches!(text[at + 1], b'b' | b'B')
        && digit(text[at + 2]).is_some_and(|d| d < 2)
    {
        radix = 2;
        at += 2;
    }
    let digits_start = at;
    let limit = if negative { 1u128 << (long_bits - 1) } else { (1u128 << (long_bits - 1)) - 1 };
    let mut magnitude = 0u128;
    let mut overflow = false;
    while at < end {
        let Some(value) = digit(text[at]) else { break };
        if value >= radix {
            break;
        }
        magnitude = match magnitude.checked_mul(radix as u128).and_then(|n| n.checked_add(value as u128)) {
            Some(n) => n,
            None => {
                overflow = true;
                magnitude
            }
        };
        if magnitude > limit {
            overflow = true;
        }
        at += 1;
    }
    if at == digits_start {
        return Err(NumericError::InvalidArgument);
    }
    if overflow {
        return Err(NumericError::OutOfRange);
    }
    let signed = if negative { -(magnitude as i128) } else { magnitude as i128 };
    Ok((signed as i64, at))
}

/// Model std::string::substr(pos, count) on bytes. `None` is npos.
/// The caller supplies *count*, not an end offset; this matters for real code
/// that passes an absolute end position as the second argument.
pub fn substr_bytes(text: &[u8], pos: usize, count: Option<usize>) -> Result<Vec<u8>, SubstrError> {
    if pos > text.len() {
        return Err(SubstrError::OutOfRange);
    }
    let end = count.map_or(text.len(), |count| pos.saturating_add(count).min(text.len()));
    Ok(text[pos..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABI: StolAbi = StolAbi { long_bits: 64, binary_prefix: true };

    #[test]
    fn numeric_prefix_and_consumed_position() {
        assert_eq!(stol_bytes(b"  -0x2a!", 0, ABI), Ok((-42, 7)));
        assert_eq!(stol_bytes(b"0779", 0, ABI), Ok((63, 3)));
        assert_eq!(stol_bytes(b"123\0ignored", 10, ABI), Ok((123, 3)));
    }

    #[test]
    fn runtime_policy_and_bounds() {
        assert_eq!(stol_bytes(b"0b101", 0, ABI), Ok((5, 5)));
        assert_eq!(stol_bytes(b"0b101", 0, StolAbi { binary_prefix: false, ..ABI }), Ok((0, 1)));
        assert_eq!(stol_bytes(b"2147483648", 10, StolAbi { long_bits: 32, ..ABI }), Err(NumericError::OutOfRange));
        assert_eq!(stol_bytes(b"no digits", 0, ABI), Err(NumericError::InvalidArgument));
        assert_eq!(stol_bytes(b"\xff12", 10, ABI), Err(NumericError::UnsupportedInput));
    }

    #[test]
    fn substring_count_and_bounds() {
        assert_eq!(substr_bytes(b"  CODEPAGE 1252", 2, Some(10)), Ok(b"CODEPAGE 1".to_vec()));
        assert_eq!(substr_bytes(b"abc", 3, Some(9)), Ok(Vec::new()));
        assert_eq!(substr_bytes(b"abc", 4, None), Err(SubstrError::OutOfRange));
    }
}
