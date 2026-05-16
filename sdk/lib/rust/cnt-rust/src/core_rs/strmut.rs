//! Wave I — in-place string mutators (libcntpr `_strrev`/`_strset`/
//! `_strnset`). All operate over the NUL-terminated content (never the
//! terminator, never past it) and the C returns the original pointer.
//! No wide twins exist in libcntpr (no `wcsrev`/`wcsset` .c) — narrow
//! only today; kept generic for any future twin. Pure, no `unsafe`.

/// `_strrev` (`strrev.c`): reverse the content in place. The C walks
/// to the NUL then swaps inward (`while (b < e)`), so length 0/1 are
/// natural no-ops — `slice::reverse` matches exactly. NO NULL guard in
/// the C (`while (*e)` derefs immediately) — NULL never fuzzed.
#[inline]
pub fn reverse<E: Copy>(buf: &mut [E]) {
    buf.reverse();
}

/// `_strset`/`_strnset` (`strset.c`): overwrite each unit with `v`
/// (the C `int szFill` truncated to `char` — the caller passes
/// `szFill as u8`). The slice is pre-bounded by the shim (whole
/// content for `_strset`; `min(strlen, (int)sizeMaxFill)` for
/// `_strnset` — the signed-int cast of `size_t` is a preserved
/// misfeature, computed in the shim).
#[inline]
pub fn fill<E: Copy>(buf: &mut [E], v: E) {
    for e in buf.iter_mut() {
        *e = v;
    }
}
