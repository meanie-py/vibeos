//! Path track — libcntpr `_splitpath`/`_wsplitpath` (`_tsplitpath_x.h`)
//! + `_makepath`/`_wmakepath` (`makepath.c`/`wmakpath.c`). Pure
//! `no_std`, NO `unsafe`, slice-based. Generic over `PChar` (u8 narrow
//! / u16 wide) — `_tsplitpath_x.h` is one tchar logic header
//! instantiated both widths, and makepath/wmakpath are byte-identical.
//!
//! libcntpr build context (NOT secure-CRT, NOT msvcrt): `_LIBCNT_` is
//! defined so the `_ismbblead` MBCS block and the `WINVER>=0x600`
//! errno path are compiled OUT (no multibyte handling even narrow);
//! `IS_SECAPI` is never set so the return is `void`, component
//! pointers may be NULL (skip that component), and the size caps are
//! `INT_MAX` (unbounded — the caller's `_MAX_*` fixed buffers).
//! `WINVER < 0x600` so the `\\?\` skip and the early `if(!path)` are
//! also out; NULL `path` UB-derefs in C and is never fuzzed.

/// Path element: u8 (narrow) / u16 (wide). The split/make logic only
/// compares ASCII punctuation, so one predicate set serves both.
pub trait PChar: Copy + PartialEq {
    const NUL: Self;
    const COLON: Self;
    const BACKSLASH: Self;
    const SLASH: Self;
    const DOT: Self;
}
impl PChar for u8 {
    const NUL: u8 = 0;
    const COLON: u8 = b':';
    const BACKSLASH: u8 = b'\\';
    const SLASH: u8 = b'/';
    const DOT: u8 = b'.';
}
impl PChar for u16 {
    const NUL: u16 = 0;
    const COLON: u16 = b':' as u16;
    const BACKSLASH: u16 = b'\\' as u16;
    const SLASH: u16 = b'/' as u16;
    const DOT: u16 = b'.' as u16;
}

/// Split result. `dir`/`fname`/`ext` are `(start,end)` content ranges
/// into the FULL `s` slice the shim passed (NOT a post-drive subslice
/// — the C does a SINGLE `path[1]==':'` detection then `path+=2`;
/// re-detecting on a sub-slice would be a bug). `has_drive` ⇒ the shim
/// synthesizes `[s[0], ':', NUL]` into the caller's `drive` buffer iff
/// `drive != NULL` (the C writes those 3 unconditionally given a
/// non-NULL `drive`, and advances `path+=2` regardless of `drive`).
pub struct Split {
    pub has_drive: bool,
    pub dir: (usize, usize),
    pub fname: (usize, usize),
    pub ext: (usize, usize),
}

/// `_splitpath`/`_wsplitpath` core (`_tsplitpath_x.h`, non-secure
/// `!IS_SECAPI` + `_LIBCNT_`). `s` = the NUL-excluded content of the
/// caller's `path`. Preserved misfeatures (each pinned by the diff):
///  * drive = exactly `s[1]==':'` (single letter; no UNC/`\\?\`).
///  * drive synthesized not sliced — `"x:"` ⇒ drive `"x:"`.
///  * `file_start` = position AFTER last `\`/`/`; trailing sep ⇒ empty
///    fname AND ext, dir gets it all.
///  * `ext_start` = last `.` but DISCARDED if `< file_start` (a dot in
///    a directory component is not an extension).
///  * leading-dot filename IS the ext (`.bashrc` ⇒ fname "", ext
///    ".bashrc").
///  * no separator ⇒ `file_start = dir_start` (whole tail is fname).
///  * dir = `[dir_start, file_start)` INCLUDES the trailing separator.
///  * ext INCLUDES the leading `.`.
///  * empty path ⇒ all components "" (C returns after truncating).
///  * `INT_MAX-1` copy caps unreachable with real buffers (not
///    modeled — matches the genuine !IS_SECAPI contract).
pub fn split<E: PChar>(s: &[E]) -> Split {
    if s.is_empty() {
        return Split { has_drive: false, dir: (0, 0), fname: (0, 0), ext: (0, 0) };
    }
    // C: `if (path[1] == ':')` — for a 1-elem path, path[1] is the
    // NUL of the terminated buffer (≠ ':'); modeled by the len check.
    let has_drive = s.len() > 1 && s[1] == E::COLON;
    let dir_start = if has_drive { 2 } else { 0 };
    let end = s.len();
    let mut file_start: Option<usize> = None;
    let mut ext_start: Option<usize> = None;
    let mut i = dir_start;
    while i < end {
        let c = s[i];
        if c == E::BACKSLASH || c == E::SLASH {
            file_start = Some(i + 1);
        }
        if c == E::DOT {
            ext_start = Some(i);
        }
        i += 1;
    }
    let file_start = file_start.unwrap_or(dir_start);
    let ext_start = match ext_start {
        Some(e) if e >= file_start => e,
        _ => end,
    };
    Split {
        has_drive,
        dir: (dir_start, file_start),
        fname: (file_start, ext_start),
        ext: (ext_start, end),
    }
}

/// `_makepath`/`_wmakepath` core (`makepath.c`/`wmakpath.c`, Wine
/// lineage, identical narrow/wide). Builds into `out`, returns the
/// element count written (excl. the trailing NUL it always writes).
/// `drive`/`dir`/`fname`/`ext` are the already-resolved content slices
/// (the shim maps NULL → empty, and NULL `path` → early no-op).
/// Misfeatures: empty component skipped (`X && X[0]`); drive emits
/// ONLY `drive[0]`+`:`; dir gets a trailing `\` appended unless it
/// already ended `/`/`\`; fname bare-concat (no sep/dot); ext: `.`
/// prepended only if `ext[0] != '.'`; always NUL-terminates.
pub fn make<E: PChar>(
    out: &mut [E],
    drive: &[E],
    dir: &[E],
    fname: &[E],
    ext: &[E],
) -> usize {
    let mut p = 0usize;
    if !drive.is_empty() {
        out[p] = drive[0];
        p += 1;
        out[p] = E::COLON;
        p += 1;
    }
    if !dir.is_empty() {
        for &e in dir {
            out[p] = e;
            p += 1;
        }
        let last = out[p - 1];
        if last != E::SLASH && last != E::BACKSLASH {
            out[p] = E::BACKSLASH;
            p += 1;
        }
    }
    if !fname.is_empty() {
        for &e in fname {
            out[p] = e;
            p += 1;
        }
    }
    if !ext.is_empty() {
        if ext[0] != E::DOT {
            out[p] = E::DOT;
            p += 1;
        }
        for &e in ext {
            out[p] = e;
            p += 1;
        }
    }
    out[p] = E::NUL;
    p
}
