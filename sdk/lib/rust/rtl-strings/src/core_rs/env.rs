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
