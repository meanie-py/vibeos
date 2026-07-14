//! `str*` / `wcs*` core. One generic impl over [`Elem`] (`u8` narrow,
//! `u16` wide) — the `tcs*.h` tchar headers are element-agnostic, so
//! every `str*`/`wcs*` pair is the *same* logic instantiated twice.
//! The only width-sensitive bit is the `strcmp` element difference:
//! narrow `char` is **signed** on x86 (`tcscmp.h` `_TCHAR`), wide
//! `wchar_t` is **unsigned 16-bit** — captured by [`Elem::diff`].
//!
//! Inputs are the *content* slices (the cabi shim resolves NUL
//! termination and NULL pointers — those are ABI concerns, not
//! algorithm concerns).

/// String element: `u8` (narrow) or `u16` (wide). `diff` is the
/// `*s1 - *s2` of `tcscmp.h` under the element's C integer promotion —
/// signed-char for `u8`, value-preserving unsigned for `u16`.
pub trait Elem: Copy + PartialEq {
    const ZERO: Self;
    fn diff(a: Self, b: Self) -> i32;
    /// The **unsigned** element difference — the textbook-libc / msvcrt
    /// convention (`(unsigned char)*s1 - (unsigned char)*s2`), as opposed
    /// to [`Elem::diff`]'s signed-`char` libcntpr convention.
    ///
    /// Two CRT bodies in this tree want *different* answers here and both
    /// are right for their caller, so the contract is a knob rather than a
    /// bug:
    ///   * the **kernel** (libcntpr) wants [`diff`](Elem::diff) — signed
    ///     `char`, pinned by `cnt-difftest`;
    ///   * **msvcrt.dll** wants this one — pinned by `msvcrt-difftest`.
    /// They agree on ASCII and disagree in *sign* on any byte >= 0x80, so
    /// picking the wrong one silently corrupts ordering/search on non-ASCII
    /// data (UTF-8 paths, accented filenames, binary buffers).
    ///
    /// Defaults to [`diff`](Elem::diff): `u16` is `wchar_t`, unsigned by
    /// definition, so the two already coincide there and only `u8` overrides.
    #[inline]
    fn udiff(a: Self, b: Self) -> i32 {
        Self::diff(a, b)
    }
    /// `_TCHAR cc = c;` — the `strchr`/`strrchr` target cast.
    fn from_cint(c: i32) -> Self;
}
impl Elem for u8 {
    const ZERO: u8 = 0;
    #[inline]
    fn diff(a: u8, b: u8) -> i32 {
        (a as i8 as i32) - (b as i8 as i32)
    }
    #[inline]
    fn udiff(a: u8, b: u8) -> i32 {
        (a as i32) - (b as i32)
    }
    #[inline]
    fn from_cint(c: i32) -> u8 {
        c as u8
    }
}
impl Elem for u16 {
    const ZERO: u16 = 0;
    #[inline]
    fn diff(a: u16, b: u16) -> i32 {
        (a as i32) - (b as i32)
    }
    #[inline]
    fn from_cint(c: i32) -> u16 {
        c as u16
    }
}

/// `strlen` core: length of the content slice (the shim resolves
/// NUL-termination and the `NULL → empty` guard). Trivial, but kept as
/// a core fn so the "core is the only impl" rule holds uniformly and
/// it's Kani-targetable.
#[inline]
pub fn len_of<T>(s: &[T]) -> usize {
    s.len()
}

/// `strcmp` core, faithful to ReactOS `tcscmp.h`: `_TCHAR` is **signed
/// `char`** on x86, so the result is the *signed*-char difference of
/// the first differing position (textbook/POSIX `strcmp` is unsigned —
/// this is a deliberate ReactOS deviation, pinned by the diff harness).
/// Slices are NUL-excluded contents; the implicit terminator (0) at
/// each end participates exactly as in the C `while(*s1==*s2)` loop.
#[inline]
pub fn cmp<T: Elem>(a: &[T], b: &[T]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let ca = if i < a.len() { a[i] } else { T::ZERO };
        let cb = if i < b.len() { b[i] } else { T::ZERO };
        if ca != cb {
            return T::diff(ca, cb);
        }
        if ca == T::ZERO {
            return 0; // both reached the terminator equal
        }
    }
    0
}

/// [`cmp`]'s **unsigned-element** twin — the msvcrt / textbook-libc
/// contract (`(unsigned char)*s1 - (unsigned char)*s2`). Identical
/// stopping rule; only [`Elem::udiff`] replaces [`Elem::diff`], so the
/// two agree on ASCII and differ in *sign* on bytes >= 0x80.
///
/// Deliberately a separate function rather than a knob threaded through
/// [`cmp`]: the kernel's `strcmp` is on every hot path in the OS and this
/// crate has been bitten before by codegen that shifted under it
/// (`core_rs::mem::set`'s DCE'd fill), so `cmp`'s body stays byte-for-byte
/// what it was. The duplicated loop is the cheap side of that trade.
///
/// Returns the raw difference; callers wanting msvcrt's `strcmp` must clamp
/// to -1/0/1 themselves (msvcrt.dll's narrow `strcmp` does, its i386
/// `strncmp` does not — see `msvcrt-crt-rust`).
#[inline]
pub fn cmp_u<T: Elem>(a: &[T], b: &[T]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let ca = if i < a.len() { a[i] } else { T::ZERO };
        let cb = if i < b.len() { b[i] } else { T::ZERO };
        if ca != cb {
            return T::udiff(ca, cb);
        }
        if ca == T::ZERO {
            return 0; // both reached the terminator equal
        }
    }
    0
}

/// `strstr` core (`strstr.c`): first index of `needle` in `hay`, or
/// `None`. Empty needle → `Some(0)` (C returns `hay`: `*find++` is 0
/// so the search is skipped and `s` returned).
#[inline]
pub fn find_sub<T: Elem>(hay: &[T], needle: &[T]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// `wcsstr` core (`wcsstr.c`): like [`find_sub`] **but an empty
/// needle returns `None`** — the C `while(*x)` loop body is never
/// entered for a 0-length `b`, so it falls through to `return NULL`.
/// (Deliberately distinct from narrow `strstr`'s empty→hay; the diff
/// harness pins both.)
#[inline]
pub fn find_sub_strict<T: Elem>(hay: &[T], needle: &[T]) -> Option<usize> {
    if needle.is_empty() {
        None
    } else {
        find_sub(hay, needle)
    }
}

/// `strspn`/`strcspn`/`wcsspn`/`wcscspn` core: length of the leading
/// run of `s` whose membership in `set` equals `want_in`. Faithful to
/// `strxspn.h` (bitmap) and the wide double-loop — identical result.
/// `set`/`s` are NUL-excluded content slices (the C scans stop at the
/// terminator, never in `set`, so this is exact).
#[inline]
pub fn span<T: Elem>(s: &[T], set: &[T], want_in: bool) -> usize {
    s.iter()
        .take_while(|&&c| set.contains(&c) == want_in)
        .count()
}

/// `strspn`/`wcsspn` (`_x = !(...)`: stop at first char NOT in set).
#[inline]
pub fn span_in<T: Elem>(s: &[T], set: &[T]) -> usize {
    span(s, set, true)
}

/// `strcspn`/`wcscspn` (`_x = (...)`: stop at first char IN set).
#[inline]
pub fn span_out<T: Elem>(s: &[T], set: &[T]) -> usize {
    span(s, set, false)
}

/// `strpbrk` core: first index of `s` whose char is in `set`, else
/// `None`. Empty `set` → `None` (C `if(*s2==0) return 0;`). All the
/// C's strchr/2-char fast paths reduce to exactly this.
#[inline]
pub fn find_any<T: Elem>(s: &[T], set: &[T]) -> Option<usize> {
    s.iter().position(|c| set.contains(c))
}

/// `strncmp` core, faithful to ReactOS `tcsncmp.h`: signed-char
/// (`_TCHAR`=signed `char`), bounded by `n`, stops at the first NUL,
/// `n==0 → 0`. Slices are NUL-excluded contents. Returns the signed
/// difference at the first differing position within `n`, else 0.
#[inline]
pub fn ncmp<T: Elem>(a: &[T], b: &[T], n: usize) -> i32 {
    for i in 0..n {
        let ca = if i < a.len() { a[i] } else { T::ZERO };
        let cb = if i < b.len() { b[i] } else { T::ZERO };
        if ca != cb {
            return T::diff(ca, cb);
        }
        if ca == T::ZERO {
            return 0;
        }
    }
    0
}

/// [`ncmp`]'s **unsigned-element** twin — see [`cmp_u`] for why this is a
/// separate function and not a knob on [`ncmp`].
#[inline]
pub fn ncmp_u<T: Elem>(a: &[T], b: &[T], n: usize) -> i32 {
    for i in 0..n {
        let ca = if i < a.len() { a[i] } else { T::ZERO };
        let cb = if i < b.len() { b[i] } else { T::ZERO };
        if ca != cb {
            return T::udiff(ca, cb);
        }
        if ca == T::ZERO {
            return 0;
        }
    }
    0
}

/// `strchr` core, faithful to ReactOS `tcschr.h`: `cc = (char)c`,
/// scans content; **the terminating NUL is a valid target** — if
/// `(c as u8) == 0` it returns `Some(len)` (pointer to the NUL), not
/// `None`. Otherwise first index whose byte equals `c as u8`, else
/// `None`. `s` is the NUL-excluded content slice.
#[inline]
pub fn find_char<T: Elem>(s: &[T], c: i32) -> Option<usize> {
    let target = T::from_cint(c);
    if let Some(i) = s.iter().position(|&b| b == target) {
        return Some(i);
    }
    if target == T::ZERO {
        Some(s.len()) // the implicit terminator
    } else {
        None
    }
}

/// `strrchr` core, faithful to ReactOS `tcsrchr.h`: last occurrence;
/// `(c as u8)==0 → Some(len)` (the terminator), else last matching
/// index or `None`.
#[inline]
pub fn find_last_char<T: Elem>(s: &[T], c: i32) -> Option<usize> {
    let target = T::from_cint(c);
    if target == T::ZERO {
        return Some(s.len());
    }
    s.iter().rposition(|&b| b == target)
}

/// `_stricmp` core, faithful to ReactOS `_stricmp_nt.c` (libcntpr NT
/// variant): per position load both bytes; if they differ, ASCII-fold
/// **only** `A..=Z`→`a..=z` (locale-independent — high-bit bytes are
/// never folded), and if still differing return the **signed-char**
/// difference (`char` is signed on x86 — `chr1 - chr2` promoted). The
/// terminating NUL participates: equal NULs → 0; a NUL vs non-NUL
/// returns the signed diff (shorter string orders first). Slices are
/// NUL-excluded contents.
#[inline]
pub fn cmp_ci(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i8 as i32;
            let mut c2 = r2 as i8 as i32;
            if r1.is_ascii_uppercase() {
                c1 += 32;
            }
            if r2.is_ascii_uppercase() {
                c2 += 32;
            }
            if c1 != c2 {
                return c1 - c2;
            }
        } else if r1 == 0 {
            return 0;
        }
    }
    0
}

/// `_strnicmp` core, faithful to ReactOS `_strnicmp_nt.c`: as
/// [`cmp_ci`] but bounded — `while (remaining-- != 0)` so `n == 0 → 0`,
/// compares at most `n` positions, and the equal-NUL break still ends
/// early. Signed-char fold + diff.
#[inline]
pub fn ncmp_ci(a: &[u8], b: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i8 as i32;
            let mut c2 = r2 as i8 as i32;
            if r1.is_ascii_uppercase() {
                c1 += 32;
            }
            if r2.is_ascii_uppercase() {
                c2 += 32;
            }
            if c1 != c2 {
                return c1 - c2;
            }
        } else if r1 == 0 {
            return 0;
        }
    }
    0
}

/// `strncpy` core. `dst` is the fixed n-length destination; `src` is
/// the source contents (NUL-excluded). Faithful to ReactOS `tcsncpy`:
/// copy `min(src.len(), n)` bytes; if `src` ended first, pad the rest
/// of `dst` with 0; if `dst` filled first, **do not NUL-terminate**.
///
/// Returns `truncated`: `true` when `dst` filled before `src`'s NUL
/// (the load-bearing misfeature — no terminator). The C `strncpy`
/// shim discards this; `_rs` callers get it as the honest signal.
#[inline]
pub fn ncpy<T: Elem>(dst: &mut [T], src: &[T]) -> bool {
    let n = dst.len();
    let copy = src.len().min(n);
    dst[..copy].copy_from_slice(&src[..copy]);
    if copy < n {
        // src's NUL reached first → pad remainder with 0.
        for b in dst[copy..].iter_mut() {
            *b = T::ZERO;
        }
        false
    } else {
        // dst filled first (copy == n) → truncated, NOT terminated.
        true
    }
}

/// `strcpy` core, faithful to ReactOS `tcscpy.h`
/// (`for(;(*to=*from);++from,++to)`): copy `src`'s content **and its
/// terminating NUL**. `dst` is exactly `src.len()+1` (the cabi shim
/// reconstructs that — `strcpy` trusts the caller's buffer, like C).
#[inline]
pub fn cpy<T: Elem>(dst: &mut [T], src: &[T]) {
    let m = src.len();
    dst[..m].copy_from_slice(src);
    dst[m] = T::ZERO;
}

/// `strcat` core, faithful to ReactOS `tcscat.h`: identical to
/// [`cpy`] applied at `dst + strlen(dst)` — the shim passes the tail
/// region (length `src.len()+1`); we write `src` + NUL there.
#[inline]
pub fn cat<T: Elem>(tail: &mut [T], src: &[T]) {
    cpy(tail, src);
}

/// `strncat` core, faithful to ReactOS `tcsncat.h`: append at most
/// `n` chars of `src` (stopping early at `src`'s NUL), then **always**
/// a terminating NUL. `tail` is the region after `dst`'s content,
/// length `min(src.len(), n) + 1`. (`n == 0` ⇒ the C does nothing at
/// all — not even the NUL — handled by the shim, which then passes a
/// zero-length job here only when `n > 0`.)
#[inline]
pub fn ncat<T: Elem>(tail: &mut [T], src: &[T], n: usize) {
    let m = src.len().min(n);
    tail[..m].copy_from_slice(&src[..m]);
    tail[m] = T::ZERO;
}
