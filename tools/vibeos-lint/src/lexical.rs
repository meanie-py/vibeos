//! Legacy Rust token masking shared by source rules.
pub fn strip_noise(text: &str) -> String {
    // Keep one character buffer for the legacy character-indexed scanner and
    // mask it in place. The previous implementation cloned this allocation,
    // making an ASCII input occupy two 4-byte-per-character buffers before
    // building the returned String.
    let mut chars: Vec<char> = text.chars().collect();
    let (mut i, n) = (0usize, chars.len());
    let starts = |chars: &[char], at: usize, pat: &str| -> bool {
        pat.chars()
            .enumerate()
            .all(|(offset, ch)| chars.get(at + offset) == Some(&ch))
    };
    let blank = |chars: &mut [char], start: usize, end: usize| {
        for c in &mut chars[start..end] {
            if *c != '\n' {
                *c = ' ';
            }
        }
    };
    while i < n {
        if !matches!(chars[i], 'r' | 'b' | '/' | '"' | '\'') {
            i += 1;
            continue;
        }
        let start = i;
        // Match Python's b?r(#*)" raw-string opener at the current token.
        let raw_prefix = if starts(&chars, i, "br") {
            Some(i + 2)
        } else if chars[i] == 'r' {
            Some(i + 1)
        } else {
            None
        };
        if let Some(mut j) = raw_prefix {
            while j < n && chars[j] == '#' {
                j += 1;
            }
            if j < n && chars[j] == '"' {
                let hashes = j - raw_prefix.unwrap();
                j += 1;
                let mut end = n;
                while j + hashes + 1 <= n {
                    if chars[j] == '"' && chars[j + 1..j + hashes + 1].iter().all(|&c| c == '#') {
                        end = j + hashes + 1;
                        break;
                    }
                    j += 1;
                }
                blank(&mut chars, start, end);
                i = end;
                continue;
            }
        }
        if starts(&chars, i, "//") {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(n, |d| i + d);
            blank(&mut chars, i, end);
            i = end;
            continue;
        }
        if starts(&chars, i, "/*") {
            let (mut depth, mut j) = (1usize, i + 2);
            while j < n && depth > 0 {
                if starts(&chars, j, "/*") {
                    depth += 1;
                    j += 2;
                } else if starts(&chars, j, "*/") {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut chars, i, j);
            i = j;
            continue;
        }
        let q = if chars[i] == 'b' && i + 1 < n && matches!(chars[i + 1], '"' | '\'') {
            i + 1
        } else {
            i
        };
        if chars[q] == '"' {
            let mut j = q + 1;
            while j < n {
                if chars[j] == '\\' {
                    j += 2;
                } else if chars[j] == '"' {
                    j += 1;
                    break;
                } else {
                    j += 1;
                }
            }
            blank(&mut chars, i, j.min(n));
            i = j.min(n);
            continue;
        }
        if chars[q] == '\'' {
            let mut j = q + 1;
            if j < n {
                if chars[j] == '\\' {
                    j += 1;
                    if j < n && chars[j] == '\n' {
                        j = n + 1;
                    } else if j < n && chars[j] == 'x' {
                        j += 1;
                        if j + 2 <= n && chars[j..j + 2].iter().all(|c| c.is_ascii_hexdigit()) {
                            j += 2;
                        } else {
                            j = n + 1;
                        }
                    } else if j < n && chars[j] == 'u' && j + 1 < n && chars[j + 1] == '{' {
                        j += 2;
                        let digits = j;
                        while j < n && chars[j].is_ascii_hexdigit() {
                            j += 1;
                        }
                        if j == digits || j >= n || chars[j] != '}' {
                            j = n + 1;
                        } else {
                            j += 1;
                        }
                    } else if j < n {
                        j += 1;
                    }
                } else if chars[j] != '\'' && chars[j] != '\\' {
                    j += 1;
                }
                if j < n && chars[j] == '\'' {
                    j += 1;
                    blank(&mut chars, i, j);
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
    chars.into_iter().collect()
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
