//! Environment block search/classify core — faithful to ReactOS
//! `stdlib/getenv.c` + `misc/environ.c::SetEnv`, NOT textbook setenv.
//!
//! Pure, `no_std`, no `unsafe`: only the *decisions* (does this entry
//! match this name? is this option a delete? does this entry collide
//! with this option under SetEnv's prefix rule?) live here. The
//! process-global owned environment block (the realloc/duplicate-on-
//! first-write/dual narrow+wide sync) is in the `cabi` layer
//! (`cabi/manual.rs`) — it can't be a safe-slice core (raw mutable
//! statics + returned interior pointers).
//!
//! Genuine quirks pinned here (the diff harness verifies them against
//! the real compiled `getenv.c`/`environ.c`):
//!  - getenv match: split at FIRST `=`; require `(pos)==strlen(name)`
//!    AND `_strnicmp(str,name,length)==0`. CASE-INSENSITIVE under the
//!    libcntpr NT ASCII fold (only `A..=Z`), and the name length must
//!    EXACTLY equal the part before `=`. Returns the slice AFTER `=`.
//!  - SetEnv classify: option must contain `=` (else the caller
//!    returns -1). `remove = (epos[1] == 0)` — i.e. `"NAME="` (empty
//!    value) DELETES.
//!  - SetEnv search predicate is `_wcsnicmp(*wenvptr, option,
//!    epos-option)` — a PREFIX compare of only (name-len) units against
//!    the WHOLE entry, NOT anchored on the entry's own `=`. So an
//!    existing entry `"NAMEX=..."` IS matched by option `"NAME=..."`
//!    when the first (name-len) units tie — a genuine asymmetry vs
//!    getenv's exact-length match. Preserve it verbatim.

#![allow(dead_code)]

/// ASCII-NT lowercase fold of one code unit, matching libcntpr
/// `_strnicmp`/`_wcsnicmp` (`*_nt.c`, pinned Wave D): fold ONLY literal
/// `A..=Z`; everything else (incl. Latin-1) raw.
#[inline]
fn fold<E: Into<u32> + Copy>(c: E) -> u32 {
    let v: u32 = c.into();
    if (0x41..=0x5A).contains(&v) {
        v + 0x20
    } else {
        v
    }
}

/// Case-insensitive equality of two equal-length slices (NT-ASCII
/// fold) — mirrors `_strnicmp`/`_wcsnicmp` over `n` units.
#[inline]
pub fn eq_ci<E: Into<u32> + Copy>(a: &[E], b: &[E]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| fold(x) == fold(y))
}

/// `_wcsnicmp(entry, option, n) == 0`: equal over the first `n` units
/// under NT-ASCII fold. `entry`/`option` are NUL-excluded; a short
/// `entry` (< n units) hits its NUL before `n` → genuine nonzero ⇒
/// NOT a match.
#[inline]
pub fn ncmp_ci_zero<E: Into<u32> + Copy>(entry: &[E], option: &[E], n: usize) -> bool {
    if entry.len() < n || option.len() < n {
        return false;
    }
    (0..n).all(|i| fold(entry[i]) == fold(option[i]))
}

/// `getenv`/`_wgetenv` per-entry test. `entry` = one block entry with
/// the trailing NUL excluded ("NAME=VALUE"). `name` = the queried name
/// (NUL excluded). Returns the index of the FIRST value unit (just
/// past `=`) when the genuine match predicate holds, else `None`.
#[inline]
pub fn getenv_value_start<E: Into<u32> + Copy + PartialEq>(
    entry: &[E],
    name: &[E],
    eq: E,
) -> Option<usize> {
    let pos = entry.iter().position(|&c| c == eq)?;
    if pos == name.len() && eq_ci(&entry[..pos], name) {
        Some(pos + 1)
    } else {
        None
    }
}

/// `SetEnv` option classification. `None` ⇒ no `=` present ⇒ the
/// caller (`_putenv`/`_wputenv`/`SetEnv`) returns -1. Otherwise the
/// `=` index and whether this is a delete (`"NAME="`, empty value:
/// genuine `epos[1] == 0`).
pub struct SetOption {
    pub eq_index: usize,
    pub remove: bool,
}

#[inline]
pub fn classify_set<E: Into<u32> + Copy + PartialEq>(
    option: &[E],
    eq: E,
) -> Option<SetOption> {
    let i = option.iter().position(|&c| c == eq)?;
    // `remove = (epos[1] == 0)`: the unit right after `=` is the C
    // string terminator. In NUL-excluded slice terms: nothing follows.
    let remove = i + 1 >= option.len();
    Some(SetOption {
        eq_index: i,
        remove,
    })
}

/// SetEnv's existing-entry collision test: `_wcsnicmp(*wenvptr,
/// option, name_len) == 0` where `name_len == epos - option`. A PREFIX
/// compare against the whole entry (the documented asymmetry vs
/// `getenv`'s exact-length match).
#[inline]
pub fn set_entry_matches<E: Into<u32> + Copy>(
    entry: &[E],
    option: &[E],
    name_len: usize,
) -> bool {
    ncmp_ci_zero(entry, option, name_len)
}
