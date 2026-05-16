//! Counted-string (UNICODE_STRING / STRING / ANSI_STRING) pure
//! semantics — Wave RTL-1: the NLS-free ops. Faithful to ReactOS
//! `sdk/lib/rtl/unicode.c`:
//!
//!  - `RtlInitAnsiString`: `Size = strlen(src)`; clamp
//!    `Size > MAXUSHORT - sizeof(CHAR)` → `MAXUSHORT - 1`; then
//!    `Length = Size`, `MaximumLength = Size + sizeof(CHAR)`. NULL ⇒
//!    `{0, 0}` (and the shim still stores `Buffer = src`).
//!  - `RtlInitAnsiStringEx`: same lengths but `Size > MAXUSHORT - 1`
//!    is a hard `STATUS_NAME_TOO_LONG` (NOT a silent clamp — the
//!    pinned asymmetry).
//!  - `RtlInitUnicodeString`: `Size = wcslen(src)*sizeof(WCHAR)`;
//!    `MaxSize = (MAXUSHORT & ~1) - sizeof(WCHAR)` (an *even* number,
//!    0xFFFC); clamp; `MaximumLength = Size + sizeof(WCHAR)`.
//!  - `RtlInitUnicodeStringEx`: same `MaxSize`; over ⇒
//!    `STATUS_NAME_TOO_LONG`.
//!  - `RtlCompareString` (case-sensitive): `len = min(L1,L2)`; signed
//!    `char` diff per byte (x86 `char` is signed — the Buffer is
//!    `PCHAR`); if equal, `ret = (int)L1 - (int)L2` (USHORT promoted).
//!  - `RtlEqualString`: `L1 != L2 ⇒ FALSE`; else
//!    `!RtlCompareString`.
//!  - `RtlPrefixString` (case-sensitive): `L2 < L1 ⇒ FALSE`; else the
//!    first `L1` bytes equal. (NULL Buffer ⇒ FALSE — even for an
//!    otherwise-empty prefix; handled at the shim where null-ness is
//!    known.)

/// `MAXUSHORT` (umtypes.h).
pub const MAXUSHORT: usize = 0xFFFF;

/// `RtlInitAnsiString`/`RtlInitString` lengths for a `strlen` of
/// `src_len`. Silent clamp at `MAXUSHORT - sizeof(CHAR)`.
#[inline]
pub fn ansi_lengths(src_len: usize) -> (u16, u16) {
    let cap = MAXUSHORT - 1; // MAXUSHORT - sizeof(CHAR)
    let size = if src_len > cap { cap } else { src_len };
    // `MaximumLength = (USHORT)Size + sizeof(CHAR)` — the +1 wraps in
    // USHORT exactly as the C cast does (only reachable at the clamp).
    (size as u16, (size as u16).wrapping_add(1))
}

/// `RtlInitAnsiStringEx`: `None` ⇒ `STATUS_NAME_TOO_LONG` (no clamp).
#[inline]
pub fn ansi_lengths_ex(src_len: usize) -> Option<(u16, u16)> {
    if src_len > MAXUSHORT - 1 {
        return None;
    }
    Some((src_len as u16, src_len as u16 + 1))
}

/// `RtlInitUnicodeString` lengths for a `wcslen` of `src_units`.
/// `MaxSize = (MAXUSHORT & ~1) - sizeof(WCHAR)` = 0xFFFC (even).
#[inline]
pub fn uni_lengths(src_units: usize) -> (u16, u16) {
    let maxsize = (MAXUSHORT & !1) - 2; // 0xFFFE - 2 = 0xFFFC
    let size = {
        let s = src_units * 2;
        if s > maxsize {
            maxsize
        } else {
            s
        }
    };
    (size as u16, (size as u16).wrapping_add(2))
}

/// `RtlInitUnicodeStringEx`: `None` ⇒ `STATUS_NAME_TOO_LONG`.
#[inline]
pub fn uni_lengths_ex(src_units: usize) -> Option<(u16, u16)> {
    let maxsize = (MAXUSHORT & !1) - 2;
    let size = src_units * 2;
    if size > maxsize {
        return None;
    }
    Some((size as u16, size as u16 + 2))
}

/// `RtlCompareString` (case-sensitive). `a`/`b` are the
/// `Length`-bounded byte views. Signed-`char` per-byte diff, then the
/// `(int)L1 - (int)L2` length tiebreak.
#[inline]
pub fn compare(a: &[u8], b: &[u8]) -> i32 {
    let len = a.len().min(b.len());
    let mut ret = 0i32;
    let mut i = 0;
    while ret == 0 && i < len {
        ret = (a[i] as i8 as i32) - (b[i] as i8 as i32);
        i += 1;
    }
    if ret == 0 {
        ret = a.len() as i32 - b.len() as i32;
    }
    ret
}

/// `RtlEqualString` (case-sensitive).
#[inline]
pub fn equal(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && compare(a, b) == 0
}

/// `RtlPrefixString` (case-sensitive), buffers known non-null. `s1`
/// is the candidate prefix, `s2` the string. `L2 < L1 ⇒ false`.
#[inline]
pub fn prefix(s1: &[u8], s2: &[u8]) -> bool {
    if s2.len() < s1.len() {
        return false;
    }
    s1.iter().zip(s2.iter()).all(|(x, y)| x == y)
}
