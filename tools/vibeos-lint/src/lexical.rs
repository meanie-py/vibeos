//! Legacy Rust token masking shared by source rules.
pub fn strip_noise(text: &str) -> String {
    if text.is_ascii() {
        // Every token pattern and delimiter is ASCII, so byte offsets preserve
        // the legacy character offsets while using one byte per input byte.
        let bytes = scan(text.as_bytes().to_vec());
        // The source was ASCII and the scanner only substitutes ASCII spaces.
        String::from_utf8(bytes).expect("ASCII masking remains valid UTF-8")
    } else {
        scan(text.chars().collect()).into_iter().collect()
    }
}

trait ScanUnit: Copy + Eq {
    fn ascii(byte: u8) -> Self;
    fn is_ascii_hexdigit(self) -> bool;
}

impl ScanUnit for u8 {
    fn ascii(byte: u8) -> Self {
        byte
    }

    fn is_ascii_hexdigit(self) -> bool {
        matches!(self, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
    }
}

impl ScanUnit for char {
    fn ascii(byte: u8) -> Self {
        byte as char
    }

    fn is_ascii_hexdigit(self) -> bool {
        matches!(self, '0'..='9' | 'a'..='f' | 'A'..='F')
    }
}

fn scan<T: ScanUnit>(mut units: Vec<T>) -> Vec<T> {
    let (mut i, n) = (0usize, units.len());
    let starts = |units: &[T], at: usize, pat: &[u8]| -> bool {
        pat.iter()
            .enumerate()
            .all(|(offset, &byte)| units.get(at + offset) == Some(&T::ascii(byte)))
    };
    let blank = |units: &mut [T], start: usize, end: usize| {
        for c in &mut units[start..end] {
            if *c != T::ascii(b'\n') {
                *c = T::ascii(b' ');
            }
        }
    };
    while i < n {
        if ![b'r', b'b', b'/', b'"', b'\'']
            .into_iter()
            .any(|byte| units[i] == T::ascii(byte))
        {
            i += 1;
            continue;
        }
        let start = i;
        // Match Python's b?r(#*)" raw-string opener at the current token.
        let raw_prefix = if starts(&units, i, b"br") {
            Some(i + 2)
        } else if units[i] == T::ascii(b'r') {
            Some(i + 1)
        } else {
            None
        };
        if let Some(mut j) = raw_prefix {
            while j < n && units[j] == T::ascii(b'#') {
                j += 1;
            }
            if j < n && units[j] == T::ascii(b'"') {
                let hashes = j - raw_prefix.unwrap();
                j += 1;
                let mut end = n;
                while j + hashes + 1 <= n {
                    if units[j] == T::ascii(b'"')
                        && units[j + 1..j + hashes + 1]
                            .iter()
                            .all(|&c| c == T::ascii(b'#'))
                    {
                        end = j + hashes + 1;
                        break;
                    }
                    j += 1;
                }
                blank(&mut units, start, end);
                i = end;
                continue;
            }
        }
        if starts(&units, i, b"//") {
            let end = units[i..]
                .iter()
                .position(|&c| c == T::ascii(b'\n'))
                .map_or(n, |d| i + d);
            blank(&mut units, i, end);
            i = end;
            continue;
        }
        if starts(&units, i, b"/*") {
            let (mut depth, mut j) = (1usize, i + 2);
            while j < n && depth > 0 {
                if starts(&units, j, b"/*") {
                    depth += 1;
                    j += 2;
                } else if starts(&units, j, b"*/") {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut units, i, j);
            i = j;
            continue;
        }
        let q = if units[i] == T::ascii(b'b')
            && i + 1 < n
            && (units[i + 1] == T::ascii(b'"') || units[i + 1] == T::ascii(b'\''))
        {
            i + 1
        } else {
            i
        };
        if units[q] == T::ascii(b'"') {
            let mut j = q + 1;
            while j < n {
                if units[j] == T::ascii(b'\\') {
                    j += 2;
                } else if units[j] == T::ascii(b'"') {
                    j += 1;
                    break;
                } else {
                    j += 1;
                }
            }
            blank(&mut units, i, j.min(n));
            i = j.min(n);
            continue;
        }
        if units[q] == T::ascii(b'\'') {
            let mut j = q + 1;
            if j < n {
                if units[j] == T::ascii(b'\\') {
                    j += 1;
                    if j < n && units[j] == T::ascii(b'\n') {
                        j = n + 1;
                    } else if j < n && units[j] == T::ascii(b'x') {
                        j += 1;
                        if j + 2 <= n && units[j..j + 2].iter().all(|c| c.is_ascii_hexdigit()) {
                            j += 2;
                        } else {
                            j = n + 1;
                        }
                    } else if j < n
                        && units[j] == T::ascii(b'u')
                        && j + 1 < n
                        && units[j + 1] == T::ascii(b'{')
                    {
                        j += 2;
                        let digits = j;
                        while j < n && units[j].is_ascii_hexdigit() {
                            j += 1;
                        }
                        if j == digits || j >= n || units[j] != T::ascii(b'}') {
                            j = n + 1;
                        } else {
                            j += 1;
                        }
                    } else if j < n {
                        j += 1;
                    }
                } else if units[j] != T::ascii(b'\'') && units[j] != T::ascii(b'\\') {
                    j += 1;
                }
                if j < n && units[j] == T::ascii(b'\'') {
                    j += 1;
                    blank(&mut units, i, j);
                    i = j;
                    continue;
                }
            }
            // An unmatched tick is a lifetime; preserve it and scan the following text.
            i += 1;
            continue;
        }
        i += 1;
    }
    units
}

#[cfg(test)]
mod tests {
    use super::strip_noise;

    #[test]
    fn masks_unicode_by_character_and_keeps_newlines() {
        let input = "α/*é\n猫*/β";
        assert_eq!(strip_noise(input), "α   \n   β");
    }

    #[test]
    fn nested_comments_raw_strings_byte_chars_and_lifetimes() {
        let input =
            "let a = r##\"/* text */\"##; /* x /* y */ z */ let b = b'Q'; let r: &'static str;";
        let expected = format!(
            "let a = {}; {} let b = {}; let r: &'static str;",
            " ".repeat(17),
            " ".repeat(17),
            " ".repeat(4),
        );
        assert_eq!(strip_noise(input), expected);
    }

    #[test]
    fn malformed_literals_keep_the_legacy_scan_boundaries() {
        assert_eq!(
            strip_noise("let x = \"unterminated\ncode;"),
            format!("let x = {}\n     ", " ".repeat(13))
        );
        assert_eq!(strip_noise("'life + value"), "'life + value");
        assert_eq!(strip_noise("b'\\xGG' + live"), "b'\\xGG' + live");
    }
}
