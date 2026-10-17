//! Legacy Rust token masking shared by source rules.
pub fn strip_noise(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = chars.clone();
    let (mut i, n) = (0usize, chars.len());
    let starts = |at: usize, pat: &str| -> bool {
        pat.chars()
            .enumerate()
            .all(|(offset, ch)| chars.get(at + offset) == Some(&ch))
    };
    let blank = |out: &mut [char], start: usize, end: usize| {
        for c in &mut out[start..end] {
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
        let raw_prefix = if starts(i, "br") {
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
                let close: Vec<char> = std::iter::once('"')
                    .chain(std::iter::repeat('#').take(hashes))
                    .collect();
                j += 1;
                let mut end = n;
                while j + close.len() <= n {
                    if chars[j..j + close.len()] == close {
                        end = j + close.len();
                        break;
                    }
                    j += 1;
                }
                blank(&mut out, start, end);
                i = end;
                continue;
            }
        }
        if starts(i, "//") {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(n, |d| i + d);
            blank(&mut out, i, end);
            i = end;
            continue;
        }
        if starts(i, "/*") {
            let (mut depth, mut j) = (1usize, i + 2);
            while j < n && depth > 0 {
                if starts(j, "/*") {
                    depth += 1;
                    j += 2;
                } else if starts(j, "*/") {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            blank(&mut out, i, j);
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
            blank(&mut out, i, j.min(n));
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
                    blank(&mut out, i, j);
                    i = j;
                    continue;
                }
            }
            // An unmatched tick is a lifetime; preserve it and scan the following text.
            out[i] = chars[i];
            i += 1;
            continue;
        }
        i += 1;
    }
    out.into_iter().collect()
}
