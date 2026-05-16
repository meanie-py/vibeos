//! Wave STRTOK — libcntpr `strtok`/`strtok_s`/`wcstok`/`wcstok_s`
//! (Wine lineage). One pure tokenizer core; the cabi shim layer owns
//! the raw-pointer walk + the non-reentrant saved-pointer state (a
//! process-global `AtomicPtr`, faithful to the genuine per-thread
//! `thread_data_t::{strtok,wcstok}_next` — single-threaded-equivalent,
//! the `_invalid_parameter` relaxed-atomic precedent). `no_std`, no
//! `unsafe`, slice-based, generic over `TokElem` (u8/u16).
//!
//! Classic `strtok`: skip leading delimiters, the next run of
//! non-delimiters is the token, a NUL is written over the terminating
//! delimiter (if any), and scanning resumes past it on the next call.
//! Delimiter membership = `strchr(delim, c)` with `c != 0` guaranteed
//! by the `*str &&` guard, so the NUL-excluded `delim` slice is exact.

/// Token element: u8 (narrow) / u16 (wide).
pub trait TokElem: Copy + PartialEq {
    const NUL: Self;
}
impl TokElem for u8 {
    const NUL: u8 = 0;
}
impl TokElem for u16 {
    const NUL: u16 = 0;
}

/// Outcome of one tokenize step over `s` (the NUL-excluded content
/// from the resume point). Indices are into `s`.
pub enum Tok {
    /// No token (only delimiters / empty). `stop` = index where the
    /// scan hit the string NUL after skipping leading delimiters.
    /// `strtok_s` sets `*ctx = base + stop` here; `strtok`/`wcstok`/
    /// `wcstok_s` leave their saved pointer UNCHANGED (a faithful
    /// asymmetry — `strtok_s` is the only one that updates on empty).
    Empty { stop: usize },
    /// Token is `s[start .. <delim or end>]`. A NUL has been written
    /// at the terminating delimiter (if one was found). `next` = where
    /// the next call resumes (past the written NUL, or the trailing
    /// string-NUL index if the token ran to the end).
    Found { start: usize, next: usize },
}

/// Tokenize `s` in place. Faithful to the Wine `strtok` body:
/// `while (*s && in(delim,*s)) s++; if(!*s) ∅; ret=s++; while(*s &&
/// !in(delim,*s)) s++; if(*s) *s++=0;`. `s` is NUL-excluded (the shim
/// builds it from the resume pointer); a found delimiter is
/// overwritten with NUL inside `s` (mutating the caller's buffer,
/// exactly as the C does).
pub fn next_token<E: TokElem>(s: &mut [E], delim: &[E]) -> Tok {
    let in_delim = |c: E| delim.iter().any(|&d| d == c);
    let mut i = 0usize;
    while i < s.len() && in_delim(s[i]) {
        i += 1;
    }
    if i == s.len() {
        // *str == 0 after skipping leading delimiters.
        return Tok::Empty { stop: i };
    }
    let start = i;
    i += 1; // ret = str++
    while i < s.len() && !in_delim(s[i]) {
        i += 1;
    }
    if i < s.len() {
        s[i] = E::NUL; // *str++ = 0
        Tok::Found { start, next: i + 1 }
    } else {
        // Ran to the terminating NUL: no separator written, resume
        // sits on the string NUL (next call yields Empty → NULL).
        Tok::Found { start, next: i }
    }
}
