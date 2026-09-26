//! Wave ENV-1 — pure string-walk core for `sdk/lib/rtl/env.c`.
//!
//! Two functions here:
//! - `find_env_var` (the algorithmic body of `RtlQueryEnvironmentVariable_U`)
//! - `expand_env_strings` (the algorithmic body of `RtlExpandEnvironmentStrings_U`)
//!
//! Both work on `&[u16]` slices — the cabi shims build the slices
//! from the caller's `PWSTR Environment` (double-null terminated)
//! and from the `UNICODE_STRING Source / Destination` pair. PEB /
//! `Nt*VirtualMemory` plumbing stays in the cabi layer.
//!
//! Faithful to ReactOS env.c (Eric Kohl). The fiddly bit is the
//! C `wcs++ ; wcschr(wcs, L'=')` pattern: name search starts at
//! `i + 1` so a hidden `=DRIVE:` cwd entry doesn't terminate its
//! own name. We preserve that exactly.
//!
//! No allocation, no `unsafe`.

use super::nls;

pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;
pub const STATUS_VARIABLE_NOT_FOUND: i32 = 0xC000_0100u32 as i32;
pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;
/// Upstream 2026-09 overflow guard added to both `_U` wrappers.
pub const STATUS_UNSUCCESSFUL: i32 = 0xC000_0001u32 as i32;
/// `ntdef.h`'s `UNICODE_STRING_MAX_CHARS` (32767) — the overflow
/// threshold the 2026-09 rewrite checks `ResultLength`/`ReturnLength`
/// against before writing it into a `UNICODE_STRING::Length` (USHORT).
pub const UNICODE_STRING_MAX_CHARS: usize = 32767;

/// Result of `find_env_var`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvQuery {
    /// No entry whose case-insensitive name matches.
    NotFound,
    /// Found at `env[value_start .. value_start + value_len]`.
    Found { value_start: usize, value_len: usize },
}

/// `wcslen` on a `&[u16]` slice, bounded by the slice length.
#[inline]
pub fn wstrlen(s: &[u16]) -> usize {
    let mut n = 0;
    while n < s.len() && s[n] != 0 {
        n += 1;
    }
    n
}

/// Look up a variable by name in an environment block. Case-insensitive.
///
/// Faithful port of the C walk inside `RtlQueryEnvironmentVariable_U`:
/// for each entry, start the `=` search at `i + 1` (so the first char
/// may itself be `=`, which is how Windows hides per-drive cwds like
/// `=C:=C:\foo`).
pub fn find_env_var(env: &[u16], name: &[u16]) -> EnvQuery {
    let mut i = 0;
    while i < env.len() && env[i] != 0 {
        // Find the '=' starting from i+1.
        let mut j = i + 1;
        while j < env.len() && env[j] != b'=' as u16 && env[j] != 0 {
            j += 1;
        }
        if j < env.len() && env[j] == b'=' as u16 {
            // var name spans env[i..j], value starts at env[j+1]
            let val_start = j + 1;
            let mut wcs = val_start;
            while wcs < env.len() && env[wcs] != 0 {
                wcs += 1;
            }
            let var_name = &env[i..j];
            if nls::compare_unicode_strings(var_name, name, true) == 0 {
                return EnvQuery::Found {
                    value_start: val_start,
                    value_len: wcs - val_start,
                };
            }
            i = wcs + 1; // past the value's null terminator
        } else {
            // No '=' found — corrupt entry, skip it.
            i = j + 1;
        }
    }
    EnvQuery::NotFound
}

/// Result of `expand_env_strings`.
pub struct ExpandResult {
    /// `STATUS_SUCCESS` or `STATUS_BUFFER_TOO_SMALL`.
    pub status: i32,
    /// Total chars that *would* be written if `dest` were unbounded,
    /// including the trailing NUL. Faithful to the C
    /// `TotalLength = ... * sizeof(WCHAR)` (we report chars; cabi
    /// scales to bytes for `*Length`).
    pub total_chars: u32,
    /// Chars actually written to `dest`, *excluding* the trailing
    /// NUL. Faithful to `Destination->Length = (DestBuffer -
    /// Destination->Buffer) * sizeof(WCHAR)`.
    pub dest_chars_written: u32,
}

/// Expand `%VAR%` references in `source` against `env`, writing into
/// `dest`. Faithful port of `RtlExpandEnvironmentStrings_U`.
///
/// On `%VAR%`, looks up VAR in env. If found, substitutes the value.
/// If not found, copies `%VAR%` literally. Unfinished `%foo` (no
/// closing `%`) is copied verbatim.
///
/// Always NUL-terminates `dest` if there's room, otherwise reports
/// `STATUS_BUFFER_TOO_SMALL`.
pub fn expand_env_strings(env: &[u16], source: &[u16], dest: &mut [u16]) -> ExpandResult {
    let percent = b'%' as u16;
    let mut src = 0usize;
    let mut src_len = source.len();
    let mut d = 0usize;
    let mut d_max = dest.len();
    let mut total_chars: u32 = 1; // trailing NUL
    let mut ret = STATUS_SUCCESS;

    while src_len != 0 {
        // copy_slice + copy_len describe one chunk to commit to dest.
        let copy_start;
        let copy_len;

        if source[src] != percent {
            // Plain run up to next '%' or end.
            copy_start = src;
            let mut n = 0;
            while src_len != 0 && source[src] != percent {
                src += 1;
                n += 1;
                src_len -= 1;
            }
            copy_len = n;
        } else {
            // `%VAR%` candidate. Search for closing '%'.
            let var_start = src + 1;
            let mut var_end = var_start;
            let mut tail = src_len - 1;
            while var_end < source.len() && source[var_end] != percent && tail != 0 {
                var_end += 1;
                tail -= 1;
            }

            if tail != 0 {
                // Closing '%' found at source[var_end].
                let var_name = &source[var_start..var_end];
                match find_env_var(env, var_name) {
                    EnvQuery::Found { value_start, value_len } => {
                        let val = &env[value_start..value_start + value_len];
                        src = var_end + 1;
                        src_len = tail - 1;
                        total_chars += value_len as u32;
                        // Faithful to C: the var substitution path
                        // delegates the copy to RtlQueryEnvironmentVariable_U,
                        // which is ALL-OR-NOTHING — on too-small it
                        // doesn't partial-write (unlike the plain-run
                        // copy below).
                        if d_max != 0 {
                            if value_len <= d_max {
                                dest[d..d + value_len].copy_from_slice(val);
                                d += value_len;
                                d_max -= value_len;
                            } else {
                                ret = STATUS_BUFFER_TOO_SMALL;
                                d_max = 0;
                            }
                        }
                        continue;
                    }
                    EnvQuery::NotFound => {
                        // Faithful to C:
                        //   CopyBuffer = SourceBuffer;
                        //   CopyLength = SourceLength - Tail + 1;
                        copy_start = src;
                        copy_len = src_len - tail + 1;
                        src_len -= copy_len;
                        src += copy_len;
                    }
                }
            } else {
                // Unfinished `%foo` — copy rest verbatim.
                copy_start = src;
                copy_len = src_len;
                src_len = 0;
            }
        }

        total_chars += copy_len as u32;
        if d_max != 0 {
            let n = core::cmp::min(copy_len, d_max);
            if n < copy_len {
                ret = STATUS_BUFFER_TOO_SMALL;
            }
            dest[d..d + n].copy_from_slice(&source[copy_start..copy_start + n]);
            d += n;
            d_max -= n;
        }
    }

    // NUL-terminate or report too-small.
    if d_max != 0 {
        dest[d] = 0;
    } else {
        ret = STATUS_BUFFER_TOO_SMALL;
    }

    ExpandResult {
        status: ret,
        total_chars,
        dest_chars_written: d as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    // --- wstrlen ---

    #[test]
    fn wstrlen_empty_slice() {
        assert_eq!(wstrlen(&[]), 0);
    }
    #[test]
    fn wstrlen_no_nul_returns_full() {
        assert_eq!(wstrlen(&w("hello")), 5);
    }
    #[test]
    fn wstrlen_stops_at_nul() {
        let s: Vec<u16> = "ab".encode_utf16().chain([0, b'c' as u16]).collect();
        assert_eq!(wstrlen(&s), 2);
    }
    #[test]
    fn wstrlen_starts_with_nul_returns_zero() {
        assert_eq!(wstrlen(&[0u16, b'a' as u16]), 0);
    }

    // --- find_env_var ---

    /// Build a NUL-separated env block from `(name, value)` pairs +
    /// final double-NUL terminator.
    fn mk_env(pairs: &[(&str, &str)]) -> Vec<u16> {
        let mut v = Vec::new();
        for (n, val) in pairs {
            v.extend(n.encode_utf16());
            v.push(b'=' as u16);
            v.extend(val.encode_utf16());
            v.push(0);
        }
        v.push(0);
        v
    }

    #[test]
    fn find_env_var_simple_hit() {
        let env = mk_env(&[("PATH", "C:\\Windows"), ("FOO", "bar")]);
        let r = find_env_var(&env, &w("FOO"));
        match r {
            EnvQuery::Found {
                value_start,
                value_len,
            } => {
                assert_eq!(value_len, 3);
                let slice = &env[value_start..value_start + value_len];
                assert_eq!(slice, &w("bar")[..]);
            }
            _ => panic!("expected Found"),
        }
    }

    #[test]
    fn find_env_var_case_insensitive() {
        let env = mk_env(&[("Path", "C:\\Windows")]);
        let r = find_env_var(&env, &w("PATH"));
        assert!(matches!(r, EnvQuery::Found { .. }));
        let r2 = find_env_var(&env, &w("path"));
        assert!(matches!(r2, EnvQuery::Found { .. }));
    }

    #[test]
    fn find_env_var_miss() {
        let env = mk_env(&[("FOO", "bar")]);
        assert_eq!(find_env_var(&env, &w("BAZ")), EnvQuery::NotFound);
    }

    #[test]
    fn find_env_var_empty_env() {
        // Just the double-NUL terminator.
        let env = vec![0u16];
        assert_eq!(find_env_var(&env, &w("ANY")), EnvQuery::NotFound);
    }

    #[test]
    fn find_env_var_empty_value() {
        let env = mk_env(&[("EMPTY", "")]);
        let r = find_env_var(&env, &w("EMPTY"));
        match r {
            EnvQuery::Found { value_len, .. } => {
                assert_eq!(value_len, 0);
            }
            _ => panic!("expected Found"),
        }
    }

    #[test]
    fn find_env_var_hidden_per_drive_cwd() {
        // Windows stores per-drive CWDs as `=C:=C:\foo\0`. The C walk
        // and our port skip the leading '=' by starting the '=' search
        // at i+1, so the name is `=C:` and the value is `C:\foo`.
        let env = mk_env(&[("=C:", "C:\\foo"), ("PATH", "ok")]);
        let r = find_env_var(&env, &w("=C:"));
        match r {
            EnvQuery::Found {
                value_start,
                value_len,
            } => {
                assert_eq!(&env[value_start..value_start + value_len], &w("C:\\foo")[..]);
            }
            _ => panic!("expected Found"),
        }
        // PATH still findable after the hidden entry.
        let r2 = find_env_var(&env, &w("PATH"));
        assert!(matches!(r2, EnvQuery::Found { .. }));
    }

    // --- expand_env_strings ---

    #[test]
    fn expand_no_percents_is_copy() {
        let env = mk_env(&[]);
        let mut dst = [0u16; 16];
        let r = expand_env_strings(&env, &w("hello"), &mut dst);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.dest_chars_written, 5);
        assert_eq!(&dst[..5], &w("hello")[..]);
        assert_eq!(dst[5], 0);
    }

    #[test]
    fn expand_substitutes_known_var() {
        let env = mk_env(&[("FOO", "BAR")]);
        let mut dst = [0u16; 16];
        let r = expand_env_strings(&env, &w("a%FOO%c"), &mut dst);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(&dst[..5], &w("aBARc")[..]);
        assert_eq!(dst[5], 0);
    }

    #[test]
    fn expand_unknown_var_copies_literal() {
        let env = mk_env(&[]);
        let mut dst = [0u16; 16];
        let r = expand_env_strings(&env, &w("a%NOPE%c"), &mut dst);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(&dst[..8], &w("a%NOPE%c")[..]);
    }

    #[test]
    fn expand_unterminated_percent_copies_literal() {
        let env = mk_env(&[("X", "y")]);
        let mut dst = [0u16; 16];
        let r = expand_env_strings(&env, &w("a%FOO"), &mut dst);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(&dst[..5], &w("a%FOO")[..]);
    }

    #[test]
    fn expand_buffer_too_small() {
        let env = mk_env(&[("PATH", "C:\\Windows")]);
        let mut dst = [0u16; 5]; // way too small
        let r = expand_env_strings(&env, &w("%PATH%"), &mut dst);
        assert_eq!(r.status, STATUS_BUFFER_TOO_SMALL);
    }
}
