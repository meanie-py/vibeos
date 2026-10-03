//! `_ismbb*` — the MBCS byte-classification family.
//! Replaces `sdk/lib/ucrt/mbstring/ismbbyte.cpp` in full (all 24 exports).
//!
//! The first locale-aware family taken over, and it is 24 exports over one
//! ~10-line generic. What made it possible was not the classification logic —
//! that is a table lookup — but `locale_update`: a NULL `_locale_t` here means
//! "the current thread's", so even the 12 non-`_l` spellings are downstream of
//! per-thread data.
//!
//! ## Three details that are silent if wrong
//!
//! * **The C returns a LOGICAL or.** `(k & kmask) || (cmask ? (c & cmask) : 0)`
//!   yields int 1 or 0, NOT the masked bits. Writing `|` instead of `||` here
//!   would return 0x40 where the C returns 1 — still "truthy", so every
//!   `if (_ismbbalpha(c))` keeps working and only code comparing against 1
//!   breaks. `_ismbbblank` is the deliberate exception: it returns `_BLANK`
//!   (0x40) for a literal tab, which is why it is written out separately.
//!
//! * **`mbctype` is indexed at `1 + byte`, not `byte`.** The table is 257
//!   entries and the leading one is not padding. Off-by-one here misclassifies
//!   every input and nothing faults.
//!
//! * **The input is truncated to a byte first** — `(unsigned char)tst` — so
//!   `_ismbbalpha(0x141)` asks about 0x41, not about nothing.
//!
//! The C dereferences `mbcinfo->mbctype` with no null check (only `_ismbbkana`
//! tests it), and that is transcribed rather than "improved": adding a guard
//! would diverge from the behaviour callers have been built against, and this
//! family's job is to be indistinguishable from what it replaces.
//!
//! Masks come from `ucrt/mbctype.h:42-45` and
//! `corecrt_wctype.h:48-58`; the per-function pairs are each function's
//! documented classification expressed in those bits.

use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

// mbctype.h:42-45 — the _mbctype[] table bits.
const _MS: c_int = 0x01; // single-byte symbol
const _MP: c_int = 0x02; // punctuation
const _M1: c_int = 0x04; // lead byte
const _M2: c_int = 0x08; // trail byte

// corecrt_wctype.h:48-58 — the _ctype[] table bits. _ALPHA carries NLS's
// C1_ALPHA (0x0100) alongside the case bits, so it is not a single bit.
const _DIGIT: c_int = 0x04;
const _PUNCT: c_int = 0x10;
const _BLANK: c_int = 0x40;
const _ALPHA: c_int = 0x0100 | 0x01 | 0x02;

/// mbctype.h:59 — `_ismbbkana` is Shift-JIS only.
const _KANJI_CP: c_int = 932;

/// `x_ismbbtype_l` (`ismbbyte.cpp:203`). Returns 1 or 0, per the C's `||`.
///
/// # Safety
/// `locale` must be null or a valid `_locale_t`; the CRT must be initialised.
unsafe fn x_ismbbtype(locale: *mut CrtLocalePointers, tst: u32, cmask: c_int, kmask: c_int) -> c_int {
    let lu = LocaleUpdate::new(locale);
    let byte = (tst as u8) as usize;

    if (*lu.mbcinfo()).mbctype[1 + byte] as c_int & kmask != 0 {
        return 1;
    }
    if cmask != 0 {
        let pctype = (*(*lu.locinfo()).public_ref()).locale_pctype;
        if *pctype.add(byte) as c_int & cmask != 0 {
            return 1;
        }
    }
    0
}

macro_rules! ismbb {
    ($name:ident, $name_l:ident, $cmask:expr, $kmask:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(tst: u32) -> c_int {
            x_ismbbtype(core::ptr::null_mut(), tst, $cmask, $kmask)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $name_l(tst: u32, plocinfo: *mut CrtLocalePointers) -> c_int {
            x_ismbbtype(plocinfo, tst, $cmask, $kmask)
        }
    };
}

ismbb!(_ismbbkalnum, _ismbbkalnum_l, 0, _MS);
ismbb!(_ismbbkprint, _ismbbkprint_l, 0, _MS | _MP);
ismbb!(_ismbbkpunct, _ismbbkpunct_l, 0, _MP);
ismbb!(_ismbbalnum, _ismbbalnum_l, _ALPHA | _DIGIT, _MS);
ismbb!(_ismbbalpha, _ismbbalpha_l, _ALPHA, _MS);
ismbb!(_ismbbgraph, _ismbbgraph_l, _PUNCT | _ALPHA | _DIGIT, _MS | _MP);
ismbb!(_ismbbprint, _ismbbprint_l, _BLANK | _PUNCT | _ALPHA | _DIGIT, _MS | _MP);
ismbb!(_ismbbpunct, _ismbbpunct_l, _PUNCT, _MP);
ismbb!(_ismbblead, _ismbblead_l, 0, _M1);
ismbb!(_ismbbtrail, _ismbbtrail_l, 0, _M2);

// `_ismbbblank` is not in the macro because a literal tab short-circuits to
// _BLANK (0x40) rather than to 1 — the one place this family returns a mask.
#[no_mangle]
pub unsafe extern "C" fn _ismbbblank(tst: u32) -> c_int {
    if tst == b'\t' as u32 {
        _BLANK
    } else {
        x_ismbbtype(core::ptr::null_mut(), tst, _BLANK, _MP)
    }
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbblank_l(tst: u32, plocinfo: *mut CrtLocalePointers) -> c_int {
    if tst == b'\t' as u32 {
        _BLANK
    } else {
        x_ismbbtype(plocinfo, tst, _BLANK, _MP)
    }
}

// `_ismbbkana` is Shift-JIS only, and unlike the rest it null-checks mbcinfo
// before reading the codepage. Two nested LocaleUpdates, as in the C: the
// outer one answers "which codepage", then the original `plocinfo` is passed
// down so the inner resolution is identical to every other member's.
#[no_mangle]
pub unsafe extern "C" fn _ismbbkana_l(tst: u32, plocinfo: *mut CrtLocalePointers) -> c_int {
    let lu = LocaleUpdate::new(plocinfo);
    let mb = lu.mbcinfo();
    if !mb.is_null() && (*mb).mbcodepage == _KANJI_CP {
        drop(lu);
        return x_ismbbtype(plocinfo, tst, 0, _MS | _MP);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbkana(tst: u32) -> c_int {
    _ismbbkana_l(tst, core::ptr::null_mut())
}
