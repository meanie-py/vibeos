//! Search / scan / tokenise: `_mbschr`, `_mbsrchr`, `_mbscspn`, `_mbspbrk`,
//! `_mbsspn`, `_mbsspnp`, `_mbsstr`, `_mbsrev`, `_mbstok`, `_mbstok_s` —
//! plain, `_l`.
//!
//! `mbscspn.cpp`/`mbspbrk.cpp` and `mbsspn.cpp`/`mbsspnp.cpp` are each ONE
//! algorithm compiled twice in the C (`#define _RETURN_PTR` + `#include`);
//! `cspn_or_pbrk`/`spn_or_spnp` below are that shared body written once.
//!
//! ## Details that are silent if wrong
//!
//! * **All of these fall to a plain `str*` primitive on `ismbcodepage==0`**
//!   except `_mbsrev` and `_mbstok`/`_mbstok_s`, which do too (`_strrev`,
//!   `strtok_s`) — so ALL nine algorithms here have the fast path; none of
//!   this family lacks it (unlike `nav.rs`).
//! * **`_mbschr`/`_mbsrchr` build a 16-bit codepoint the same way `compare.rs`
//!   does**, but a dangling lead byte is handled differently per-function:
//!   `_mbschr` returns `nullptr` outright on a dangling lead (treats it as a
//!   hard stop, no match possible); `_mbsrchr` treats a dangling lead as "the
//!   NUL", eligible to match `c == 0` and recorded as a candidate result
//!   pointing AT the NUL if none was found yet and `c==0` was requested —
//!   this is why `_mbsrchr`'s do/while re-tests `cc` after the loop for a
//!   final "does `c` match NUL" case that `_mbschr` also has, spelled
//!   differently (`if (c == cc) return string;` after the for-loop, using
//!   the LAST read `cc`, i.e. the NUL byte itself).
//! * **`_mbscspn`/`_mbspbrk`'s charset scan treats a lead byte in `charset`
//!   specially even when scanning for a SINGLE-byte `*q`**: `p[1]=='\0'`
//!   (a dangling lead in the CHARSET) matches unconditionally, regardless of
//!   what `*q` is — a malformed charset's trailing lead byte becomes a
//!   wildcard.
//! * **`_mbsstr` computes `endp` from `strlen(str1) - strlen(str2)`** — a
//!   BYTE-length subtraction done once up front, not updated as `cp` walks
//!   forward character-by-character, and the inner match loop advances `s1`/
//!   `s2` with plain `++` (byte-wise), relying on MBCS strings being "legal"
//!   per the C's own comment; only the OUTER cursor `cp` is advanceed via
//!   `_ismbblead_l`-aware stepping.
//! * **`_mbsrev` truncates the string in place on a dangling trailing lead
//!   byte** (sets `errno = EINVAL`, backs up, writes a NUL there, and
//!   continues reversing the shortened string) rather than failing —
//!   the ONE function in this whole module that can shrink its argument's
//!   apparent length as a side effect of an error path.
//! * **`_mbstok`/`_mbstok_l` share ONE PER-THREAD token slot** — the plain
//!   `_mbstok` calls `_mbstok_l` (not `_mbstok_s_l` directly) specifically so
//!   that single-threaded callers keep working against one C `static`-style
//!   slot; this port's `_mbstok_l` still routes through `_mbstok_s_l` with
//!   `&(*__acrt_getptd()).mbstok_token`, i.e. the REAL per-thread slot, not a
//!   process-global static — see `ptd.rs`.
//! * **`_mbstok_s_l`'s delimiter scan treats a dangling lead byte in
//!   `_Control` as a syntax error (`_SET_MBCS_ERROR`, i.e. `errno = EILSEQ`)
//!   but keeps going** rather than stopping — the malformed delimiter entry
//!   is simply skipped (`ctl++` past it) in both the token-start and
//!   token-end scans.

use super::{Locale, EILSEQ, EINVAL};
use crate::ismbbyte::_ismbblead_l;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int};

extern "C" {
    fn _errno() -> *mut c_int;
    fn strchr(s: *const u8, c: c_int) -> *mut u8;
    fn strrchr(s: *const u8, c: c_int) -> *mut u8;
    fn strcspn(s: *const u8, set: *const u8) -> usize;
    fn strpbrk(s: *const u8, set: *const u8) -> *mut u8;
    fn strspn(s: *const u8, set: *const u8) -> usize;
    fn strstr(s: *const u8, needle: *const u8) -> *mut u8;
    fn strlen(s: *const c_char) -> usize;
    fn _strrev(s: *mut u8) -> *mut u8;
    fn strtok_s(s: *mut u8, delim: *const u8, ctx: *mut *mut u8) -> *mut u8;
}

// ── mbschr.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbschr_l(string: *const u8, c: u32, locale: Locale) -> *mut u8 {
    let mut lu = LocaleUpdate::new(locale);
    if string.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strchr(string, c as c_int);
    }
    let l = lu.locale_t();
    let mut s = string;
    loop {
        let cc = *s;
        if cc == 0 {
            break;
        }
        if _ismbblead_l(cc as u32, l) != 0 {
            s = s.add(1);
            if *s == 0 {
                return core::ptr::null_mut();
            }
            if c == (((cc as u32) << 8) | *s as u32) {
                return s.sub(1) as *mut u8;
            }
        } else if c == cc as u32 {
            break;
        }
        s = s.add(1);
    }
    if c == *s as u32 {
        s as *mut u8
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbschr(string: *const u8, c: u32) -> *mut u8 {
    _mbschr_l(string, c, core::ptr::null_mut())
}

// ── mbsrchr.cpp ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsrchr_l(str_: *const u8, c: u32, locale: Locale) -> *mut u8 {
    if str_.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strrchr(str_, c as c_int);
    }
    let l = lu.locale_t();
    let mut r: *mut u8 = core::ptr::null_mut();
    let mut s = str_;
    loop {
        let cc = *s as u32;
        if _ismbblead_l(cc, l) != 0 {
            s = s.add(1);
            if *s != 0 {
                if c == ((cc << 8) | *s as u32) {
                    r = s.sub(1) as *mut u8;
                }
            } else if r.is_null() {
                r = s as *mut u8;
            }
        } else if c == cc {
            r = s as *mut u8;
        }
        let cont = *s != 0;
        s = s.add(1);
        if !cont {
            break;
        }
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrchr(str_: *const u8, c: u32) -> *mut u8 {
    _mbsrchr_l(str_, c, core::ptr::null_mut())
}

// ── mbscspn.cpp / mbspbrk.cpp (one algorithm, two return shapes) ─────────

#[inline]
unsafe fn cspn_scan(string: *const u8, charset: *const u8, l: Locale) -> *const u8 {
    let (mut p, mut q);
    q = string;
    while *q != 0 {
        p = charset;
        while *p != 0 {
            if _ismbblead_l(*p as u32, l) != 0 {
                if (*p == *q && p.add(1).read() == q.add(1).read()) || p.add(1).read() == 0 {
                    break;
                }
                p = p.add(1);
            } else if *p == *q {
                break;
            }
            p = p.add(1);
        }
        if *p != 0 {
            break;
        }
        if _ismbblead_l(*q as u32, l) != 0 {
            q = q.add(1);
            if *q == 0 {
                break;
            }
        }
        q = q.add(1);
    }
    q
}

#[no_mangle]
pub unsafe extern "C" fn _mbscspn_l(string: *const u8, charset: *const u8, locale: Locale) -> usize {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strcspn(string, charset);
    }
    if string.is_null() || charset.is_null() {
        *_errno() = EINVAL;
        return 0;
    }
    cspn_scan(string, charset, lu.locale_t()) as usize - string as usize
}

#[no_mangle]
pub unsafe extern "C" fn _mbscspn(string: *const u8, charset: *const u8) -> usize {
    _mbscspn_l(string, charset, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbspbrk_l(string: *const u8, charset: *const u8, locale: Locale) -> *mut u8 {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strpbrk(string, charset);
    }
    if string.is_null() || charset.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let q = cspn_scan(string, charset, lu.locale_t());
    if *q != 0 { q as *mut u8 } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn _mbspbrk(string: *const u8, charset: *const u8) -> *mut u8 {
    _mbspbrk_l(string, charset, core::ptr::null_mut())
}

// ── mbsspn.cpp / mbsspnp.cpp ───────────────────────────────────────────────

#[inline]
unsafe fn spn_scan(string: *const u8, charset: *const u8, l: Locale) -> *const u8 {
    let (mut p, mut q);
    q = string;
    while *q != 0 {
        p = charset;
        while *p != 0 {
            if _ismbblead_l(*p as u32, l) != 0 {
                if (*p == *q && p.add(1).read() == q.add(1).read()) || p.add(1).read() == 0 {
                    break;
                }
                p = p.add(1);
            } else if *p == *q {
                break;
            }
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }
        if _ismbblead_l(*q as u32, l) != 0 {
            q = q.add(1);
            if *q == 0 {
                break;
            }
        }
        q = q.add(1);
    }
    q
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspn_l(string: *const u8, charset: *const u8, locale: Locale) -> usize {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strspn(string, charset);
    }
    if string.is_null() || charset.is_null() {
        *_errno() = EINVAL;
        return 0;
    }
    spn_scan(string, charset, lu.locale_t()) as usize - string as usize
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspn(string: *const u8, charset: *const u8) -> usize {
    _mbsspn_l(string, charset, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspnp_l(string: *const u8, charset: *const u8, locale: Locale) -> *mut u8 {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        let retval = strspn(string, charset);
        let at = string.add(retval);
        return if *at != 0 { at as *mut u8 } else { core::ptr::null_mut() };
    }
    if string.is_null() || charset.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let q = spn_scan(string, charset, lu.locale_t());
    if *q != 0 { q as *mut u8 } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspnp(string: *const u8, charset: *const u8) -> *mut u8 {
    _mbsspnp_l(string, charset, core::ptr::null_mut())
}

// ── mbsstr.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsstr_l(str1: *const u8, str2: *const u8, locale: Locale) -> *mut u8 {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strstr(str1, str2);
    }
    if str2.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    if *str2 == 0 {
        return str1 as *mut u8;
    }
    if str1.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let l = lu.locale_t();
    let len1 = strlen(str1.cast());
    let len2 = strlen(str2.cast());
    let endp = str1.add(len1.wrapping_sub(len2));
    let mut cp = str1;
    while *cp != 0 && cp <= endp {
        let (mut s1, mut s2) = (cp, str2);
        while *s1 != 0 && *s2 != 0 && *s1 == *s2 {
            s1 = s1.add(1);
            s2 = s2.add(1);
        }
        if *s2 == 0 {
            return cp as *mut u8;
        }
        let lead_c = *cp;
        cp = cp.add(1);
        if _ismbblead_l(lead_c as u32, l) != 0 && *cp != 0 {
            cp = cp.add(1);
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _mbsstr(str1: *const u8, str2: *const u8) -> *mut u8 {
    _mbsstr_l(str1, str2, core::ptr::null_mut())
}

// ── mbsrev.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsrev_l(string: *mut u8, locale: Locale) -> *mut u8 {
    if string.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let start = string;
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strrev(string);
    }
    let l = lu.locale_t();
    let mut left = string;
    let mut s = string;
    while *s != 0 {
        let is_lead = _ismbblead_l(*s as u32, l) != 0;
        s = s.add(1);
        if is_lead {
            if *s != 0 {
                let c = *s;
                *s = *s.sub(1);
                *s.sub(1) = c;
                s = s.add(1);
            } else {
                *_errno() = EINVAL;
                s = s.sub(1);
                *s = 0;
                break;
            }
        }
    }
    s = s.sub(1);
    while left < s {
        let c = *left;
        *left = *s;
        left = left.add(1);
        *s = c;
        s = s.sub(1);
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrev(string: *mut u8) -> *mut u8 {
    _mbsrev_l(string, core::ptr::null_mut())
}

// ── mbstok_s.cpp ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbstok_s_l(
    mut string: *mut u8,
    control: *const u8,
    context: *mut *mut u8,
    locale: Locale,
) -> *mut u8 {
    if context.is_null() || control.is_null() {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    if !(!string.is_null() || !(*context).is_null()) {
        *_errno() = EINVAL;
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strtok_s(string, control, context);
    }
    let l = lu.locale_t();
    if string.is_null() {
        string = *context;
    }

    // Skip leading delimiters.
    loop {
        if *string == 0 {
            break;
        }
        let mut ctl = control;
        loop {
            if *ctl == 0 {
                break;
            }
            if _ismbblead_l(*ctl as u32, l) != 0 {
                if *ctl.add(1) == 0 {
                    ctl = ctl.add(1);
                    *_errno() = EILSEQ;
                    break;
                }
                if *ctl == *string && *ctl.add(1) == *string.add(1) {
                    break;
                }
                ctl = ctl.add(1);
            } else if *ctl == *string {
                break;
            }
            ctl = ctl.add(1);
        }
        if *ctl == 0 {
            break;
        }
        if _ismbblead_l(*string as u32, l) != 0 {
            string = string.add(1);
            if *string == 0 {
                *_errno() = EILSEQ;
                break;
            }
        }
        string = string.add(1);
    }

    let token = string;

    // Find the end of the token.
    loop {
        if *string == 0 {
            break;
        }
        let mut ctl = control;
        let mut dbc = false;
        loop {
            if *ctl == 0 {
                break;
            }
            if _ismbblead_l(*ctl as u32, l) != 0 {
                if *ctl.add(1) == 0 {
                    ctl = ctl.add(1);
                    break;
                }
                if *ctl == *string && *ctl.add(1) == *string.add(1) {
                    dbc = true;
                    break;
                }
                ctl = ctl.add(1);
            } else if *ctl == *string {
                break;
            }
            ctl = ctl.add(1);
        }
        if *ctl != 0 {
            *string = 0;
            string = string.add(1);
            if dbc {
                *string = 0;
                string = string.add(1);
            }
            break;
        }
        if _ismbblead_l(*string as u32, l) != 0 {
            if *string.add(1) == 0 {
                *string = 0;
                break;
            }
            string = string.add(1);
        }
        string = string.add(1);
    }

    *context = string;
    if token == string {
        core::ptr::null_mut()
    } else {
        token
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok_s(string: *mut u8, control: *const u8, context: *mut *mut u8) -> *mut u8 {
    _mbstok_s_l(string, control, context, core::ptr::null_mut())
}

// ── mbstok.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbstok_l(string: *mut u8, sepset: *const u8, locale: Locale) -> *mut u8 {
    let ptd = crate::locale_update::__acrt_getptd();
    _mbstok_s_l(string, sepset, &mut (*ptd).mbstok_token, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok(string: *mut u8, sepset: *const u8) -> *mut u8 {
    _mbstok_l(string, sepset, core::ptr::null_mut())
}
