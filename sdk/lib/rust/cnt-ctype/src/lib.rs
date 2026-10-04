//! cnt-ctype: Rust reimplementation of libcntpr `string/ctype.c`.
//!
//! The exported data tables (`_ctype`/`_wctype`) and narrow
//! classification (`_isctype`, `isalpha`…`toupper`) are pure (the
//! `pure` module below — no `unsafe`, Kani-able) and pinned byte-for-
//! byte against the genuine compiled C by cnt-difftest. The wide
//! `isw*` / `towlower` forward to the still-C `iswctype` (sub-step A),
//! exactly as ctype.c does — a faithful forwarder, no behavior change
//! regardless of which `iswctype` (iswctype_nt.c / wctype.c) is linked.
//!
//! Own crate (not a cnt-rust module) to keep cnt-rust's cgu layout
//! unperturbed — see Cargo.toml.
#![cfg_attr(not(test), no_std)]
#![allow(non_upper_case_globals)]

/// Pure, slice-free tables + classification (no `unsafe`). THE
/// implementation; the `#[no_mangle]` shims below only adapt the ABI.
pub mod pure {
    // <ctype.h> flag masks (verified against sdk/include/crt/ctype.h).
    pub const UPPER: i32 = 0x1;
    pub const LOWER: i32 = 0x2;
    pub const DIGIT: i32 = 0x4;
    pub const SPACE: i32 = 0x8;
    pub const PUNCT: i32 = 0x10;
    pub const CONTROL: i32 = 0x20;
    pub const BLANK: i32 = 0x40;
    pub const HEX: i32 = 0x80;
    pub const LEADBYTE: i32 = 0x8000;
    pub const ALPHA: i32 = 0x100 | UPPER | LOWER;

    // upalpha = 'A' - 'a' = -32.
    const UPALPHA: i32 = -32;

    pub static CTYPE: [u16; 257] = [
        0x0000, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0028, 0x0028, 0x0028, 0x0028, 0x0028, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0048, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0084,
        0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0081, 0x0081, 0x0081, 0x0081,
        0x0081, 0x0081, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
        0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
        0x0001, 0x0001, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0082, 0x0082,
        0x0082, 0x0082, 0x0082, 0x0082, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002,
        0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002,
        0x0002, 0x0002, 0x0002, 0x0002, 0x0010, 0x0010, 0x0010, 0x0010, 0x0020, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ];

    pub static WCTYPE: [u16; 257] = [
        0x0000, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0068, 0x0028, 0x0028, 0x0028, 0x0028, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0048, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0084,
        0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0181, 0x0181, 0x0181, 0x0181,
        0x0181, 0x0181, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
        0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
        0x0101, 0x0101, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0182, 0x0182,
        0x0182, 0x0182, 0x0182, 0x0182, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
        0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
        0x0102, 0x0102, 0x0102, 0x0102, 0x0010, 0x0010, 0x0010, 0x0010, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
        0x0020, 0x0048, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0014,
        0x0014, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0014, 0x0010, 0x0010, 0x0010,
        0x0010, 0x0010, 0x0010, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
        0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
        0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0010, 0x0101, 0x0101, 0x0101,
        0x0101, 0x0101, 0x0101, 0x0101, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
        0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
        0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0010, 0x0102,
        0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    ];

    /// `_isctype(c, flags)` = `_pctype[(u8)(c & 0xFF)] & flags`; with
    /// `_pctype == _ctype + 1` the index is `1 + (c as u8)` (1..=256).
    #[inline]
    pub fn isctype(c: i32, flags: i32) -> i32 {
        (CTYPE[1 + (c as u8) as usize] as i32) & flags
    }

    /// `__isascii(c)` = `(unsigned char)c <= 0x7f` (low byte only).
    #[inline]
    pub fn isascii(c: i32) -> i32 {
        (((c as u8) as i32) <= 0x7f) as i32
    }
    /// `__toascii(c)` = `(unsigned int)c & 0x7f`.
    #[inline]
    pub fn toascii(c: i32) -> i32 {
        (c as u32 & 0x7f) as i32
    }
    /// `__iscsym(c)` = `c < 127 && (isalnum(c) || c == '_')`.
    #[inline]
    pub fn iscsym(c: i32) -> i32 {
        (c < 127 && (isctype(c, ALPHA | DIGIT) != 0 || c == b'_' as i32)) as i32
    }
    /// `__iscsymf(c)` = `c < 127 && (isalpha(c) || c == '_')`.
    #[inline]
    pub fn iscsymf(c: i32) -> i32 {
        (c < 127 && (isctype(c, ALPHA) != 0 || c == b'_' as i32)) as i32
    }

    /// `_tolower(c)` = `if _isctype(c,_UPPER) { c - upalpha } else { c }`
    /// (upalpha = -32 → the uppercase branch is `c + 32`).
    #[inline]
    pub fn tolower(c: i32) -> i32 {
        if isctype(c, UPPER) != 0 {
            c - UPALPHA
        } else {
            c
        }
    }
    /// `_toupper(c)` = `if _isctype(c,_LOWER) { c + upalpha } else { c }`
    /// (upalpha = -32 → the lowercase branch is `c - 32`). libcntpr's
    /// plain `toupper` is identical (outside the `#ifndef _LIBCNT_`).
    #[inline]
    pub fn toupper(c: i32) -> i32 {
        if isctype(c, LOWER) != 0 {
            c + UPALPHA
        } else {
            c
        }
    }

    /// `iswctype(_C, _Type)` (iswctype_nt.c) = `_C <= 0xFF ?
    /// (_wctype[_C + 1] & _Type) : 0`. Uses the narrow `_wctype` 257-
    /// table (idx 0 = EOF), NOT the wine full-Unicode table — faithful
    /// to the libcntpr-linked iswctype.
    #[inline]
    pub fn iswctype(c: u16, ty: u16) -> i32 {
        if c <= 0xFF {
            (WCTYPE[c as usize + 1] as i32) & (ty as i32)
        } else {
            0
        }
    }
}

use pure as ct;

// ---- exported data tables (C: `const unsigned short _ctype[257]`) ----
// i386 PE decorates both data and functions with a leading `_`, matching
// what i686 C emits for these already-underscored names (verified: C
// `_wctype` → symbol `__wctype`, Rust `#[no_mangle] static _wctype` →
// `__wctype`). Plain `#[no_mangle]` is correct on all three arches.
#[no_mangle]
pub static _ctype: [u16; 257] = ct::CTYPE;
#[no_mangle]
pub static _wctype: [u16; 257] = ct::WCTYPE;

/// `const unsigned short *` wrapper — a raw pointer isn't `Sync`, but
/// these point at immutable `'static` tables and are only ever read.
#[repr(transparent)]
pub struct CtypePtr(pub *const u16);
unsafe impl Sync for CtypePtr {}

// `_pctype = _ctype + 1`, `_pwctype = _wctype + 1` — pointers into the
// exported tables above (const relocation + const ptr offset).
#[no_mangle]
pub static _pctype: CtypePtr = CtypePtr(unsafe { core::ptr::addr_of!(_ctype).cast::<u16>().add(1) });
#[no_mangle]
pub static _pwctype: CtypePtr =
    CtypePtr(unsafe { core::ptr::addr_of!(_wctype).cast::<u16>().add(1) });

/// `const unsigned short **__p__pwctype(void)` — address of `_pwctype`.
#[no_mangle]
pub unsafe extern "C" fn __p__pwctype() -> *const *const u16 {
    core::ptr::addr_of!(_pwctype) as *const *const u16
}

/// `const unsigned short *__pwctype_func(void)` — the `_pwctype` value.
#[no_mangle]
pub unsafe extern "C" fn __pwctype_func() -> *const u16 {
    _pwctype.0
}

// ---- narrow classification (pure) ----------------------------------
/// `int _isctype(int c, int ctypeFlags)` (the `_LIBCNT_` branch).
#[no_mangle]
pub unsafe extern "C" fn _isctype(c: i32, ctype_flags: i32) -> i32 {
    ct::isctype(c, ctype_flags)
}

#[no_mangle]
pub unsafe extern "C" fn isalpha(c: i32) -> i32 {
    ct::isctype(c, ct::ALPHA)
}
#[no_mangle]
pub unsafe extern "C" fn isalnum(c: i32) -> i32 {
    ct::isctype(c, ct::ALPHA | ct::DIGIT)
}
#[no_mangle]
pub unsafe extern "C" fn iscntrl(c: i32) -> i32 {
    ct::isctype(c, ct::CONTROL)
}
#[no_mangle]
pub unsafe extern "C" fn isdigit(c: i32) -> i32 {
    ct::isctype(c, ct::DIGIT)
}
#[no_mangle]
pub unsafe extern "C" fn isgraph(c: i32) -> i32 {
    ct::isctype(c, ct::PUNCT | ct::ALPHA | ct::DIGIT)
}
#[no_mangle]
pub unsafe extern "C" fn islower(c: i32) -> i32 {
    ct::isctype(c, ct::LOWER)
}
#[no_mangle]
pub unsafe extern "C" fn isprint(c: i32) -> i32 {
    ct::isctype(c, ct::BLANK | ct::PUNCT | ct::ALPHA | ct::DIGIT)
}
#[no_mangle]
pub unsafe extern "C" fn ispunct(c: i32) -> i32 {
    ct::isctype(c, ct::PUNCT)
}
#[no_mangle]
pub unsafe extern "C" fn isspace(c: i32) -> i32 {
    ct::isctype(c, ct::SPACE)
}
#[no_mangle]
pub unsafe extern "C" fn isupper(c: i32) -> i32 {
    ct::isctype(c, ct::UPPER)
}
#[no_mangle]
pub unsafe extern "C" fn isxdigit(c: i32) -> i32 {
    ct::isctype(c, ct::HEX)
}

#[no_mangle]
pub unsafe extern "C" fn __isascii(c: i32) -> i32 {
    ct::isascii(c)
}
#[no_mangle]
pub unsafe extern "C" fn __toascii(c: i32) -> i32 {
    ct::toascii(c)
}
#[no_mangle]
pub unsafe extern "C" fn __iscsym(c: i32) -> i32 {
    ct::iscsym(c)
}
#[no_mangle]
pub unsafe extern "C" fn __iscsymf(c: i32) -> i32 {
    ct::iscsymf(c)
}

/// `int _tolower(int c)`.
#[no_mangle]
pub unsafe extern "C" fn _tolower(c: i32) -> i32 {
    ct::tolower(c)
}
/// `int _toupper(int c)`.
#[no_mangle]
pub unsafe extern "C" fn _toupper(c: i32) -> i32 {
    ct::toupper(c)
}
/// `int toupper(int c)` — libcntpr provides plain `toupper` (it is
/// outside the `#ifndef _LIBCNT_`); identical to `_toupper`.
#[no_mangle]
pub unsafe extern "C" fn toupper(c: i32) -> i32 {
    ct::toupper(c)
}

// ---- wide classification (now fully Rust: iswctype_nt.c flipped) ----
/// `int iswctype(wint_t _C, wctype_t _Type)` (iswctype_nt.c).
#[cfg(not(feature = "no-iswctype"))]
#[no_mangle]
pub unsafe extern "C" fn iswctype(c: u16, t: u16) -> i32 {
    ct::iswctype(c, t)
}
/// `int is_wctype(wint_t wc, wctype_t wctypeFlags)` (is_wctype.c) —
/// obsolete alias that just forwards to `iswctype`.
#[no_mangle]
pub unsafe extern "C" fn is_wctype(wc: u16, wctype_flags: u16) -> i32 {
    ct::iswctype(wc, wctype_flags)
}

/// `int iswascii(wint_t c)` = `__isascii(c)` (does NOT use iswctype).
#[no_mangle]
pub unsafe extern "C" fn iswascii(c: u16) -> i32 {
    ct::isascii(c as i32)
}

#[no_mangle]
pub unsafe extern "C" fn iswalnum(c: u16) -> i32 {
    ct::iswctype(c, (ct::ALPHA | ct::DIGIT) as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswalpha(c: u16) -> i32 {
    ct::iswctype(c, ct::ALPHA as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswcntrl(c: u16) -> i32 {
    ct::iswctype(c, ct::CONTROL as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswdigit(c: u16) -> i32 {
    ct::iswctype(c, ct::DIGIT as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswgraph(c: u16) -> i32 {
    ct::iswctype(c, (ct::PUNCT | ct::ALPHA | ct::DIGIT) as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswprint(c: u16) -> i32 {
    ct::iswctype(c, (ct::BLANK | ct::PUNCT | ct::ALPHA | ct::DIGIT) as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswpunct(c: u16) -> i32 {
    ct::iswctype(c, ct::PUNCT as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswlower(c: u16) -> i32 {
    ct::iswctype(c, ct::LOWER as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswupper(c: u16) -> i32 {
    ct::iswctype(c, ct::UPPER as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswspace(c: u16) -> i32 {
    ct::iswctype(c, ct::SPACE as u16)
}
#[no_mangle]
pub unsafe extern "C" fn iswxdigit(c: u16) -> i32 {
    ct::iswctype(c, ct::HEX as u16)
}

/// `wint_t towlower(wint_t c)` = `if iswctype(c,_UPPER) { c - upalpha }
/// else { c }` (upalpha = -32 → uppercase branch is `c + 32`).
#[no_mangle]
pub unsafe extern "C" fn towlower(c: u16) -> u16 {
    if ct::iswctype(c, ct::UPPER as u16) != 0 {
        (c as i32 + 32) as u16
    } else {
        c
    }
}
