//! Shared plumbing for the `convert_rest` cluster: `mbstate_t`'s layout, the
//! two `__crt_mbstring::common_utf8.cpp` helpers, `_isleadbyte_fast_internal`,
//! and the Win32/NLS externs every sibling needs (declared once here so no
//! two submodules can drift on a prototype and trip
//! `#![deny(clashing_extern_declarations)]`).

use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int, c_uint};

/// `_Mbstatet` / `mbstate_t` (`ucrt/corecrt.h:636`): `unsigned
/// long _Wchar; unsigned short _Byte, _State;`. `unsigned long` is always 32
/// bits on every Windows target, so this is 8 bytes / align 4 on all three
/// arches -- verified against `sizeof(mbstate_t)` call sites (`MB_LEN_MAX`
/// buffers etc.) rather than re-measured, since the C source gives the exact
/// member types directly.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MbstateT {
    pub wchar: u32,
    pub byte: u16,
    pub state: u16,
}

const _: () = assert!(core::mem::size_of::<MbstateT>() == 8);

impl MbstateT {
    pub const fn zeroed() -> Self {
        Self { wchar: 0, byte: 0, state: 0 }
    }
}

/// `__crt_mbstring::INVALID`.
pub const INVALID: usize = usize::MAX;
/// `__crt_mbstring::INCOMPLETE`.
pub const INCOMPLETE: usize = usize::MAX - 1;

pub const CP_UTF8: c_uint = 65001;
pub const CP_ACP: c_uint = 0;
pub const MB_PRECOMPOSED: c_uint = 0x1;
pub const MB_ERR_INVALID_CHARS: c_uint = 0x8;

/// `_LEADBYTE` (corecrt_wctype.h:57) as read out of `_locale_pctype`.
const LEADBYTE: c_int = 0x8000;

/// `CRT_LC_CTYPE` -- an index into `locale_name[6]`.
pub const CRT_LC_CTYPE: usize = 2;

/// `EILSEQ`/`EINVAL`/`ERANGE` (errno.h). Named locally per the crate's
/// per-module convention (grepped: no contradictory value exists elsewhere).
pub const EILSEQ: c_int = 42;
pub const EINVAL: c_int = 22;
pub const ERANGE: c_int = 34;
/// `STRUNCATE` (corecrt.h).
pub const STRUNCATE: c_int = 80;
/// `_TRUNCATE` (corecrt.h): `(size_t)-1`.
pub const TRUNCATE: usize = usize::MAX;
/// `MB_LEN_MAX` (limits.h, UCRT's value).
pub const MB_LEN_MAX: usize = 5;

extern "C" {
    // corecrt_internal.h:601 / :611 (cdecl) -- identical prototype to the one
    // in findfile.rs / nls_wrappers.rs / setlocale.rs.
    pub fn __acrt_MultiByteToWideChar(
        cp: c_uint,
        flags: c_uint,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    pub fn __acrt_WideCharToMultiByte(
        cp: c_uint,
        flags: c_uint,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;

    /// `corecrt_internal.h:922`: the non-aborting, non-caching getptd used by
    /// `try_get_ptd_buffer` (fcvt.cpp). Distinct from `__acrt_getptd` (which
    /// aborts on failure, and is what `PtdHost::raw_ptd` calls) -- this one
    /// returns null instead. One-shot, so no caching wrapper is needed the
    /// way `PtdHost` caches `__acrt_getptd`.
    pub fn __acrt_getptd_noexit() -> *mut crate::ptd::AcrtPtd;
}

/// `_ctype_fast_check_internal(c, _LEADBYTE, locale)`
/// (corecrt_internal.h:1552, `__forceinline`, so no external symbol to link
/// against -- this is a straight transcription, not a call-through).
///
/// # Safety
/// `locale` must be a valid, non-null `_locale_t` (as returned by
/// `PtdHost::get_locale`/`LocaleUpdate::locale_t`).
#[inline]
pub unsafe fn is_leadbyte_fast(c: u8, locale: *mut CrtLocalePointers) -> bool {
    let pctype = (*(*(*locale).locinfo).public_ref()).locale_pctype;
    (*pctype.add(c as usize) as c_int & LEADBYTE) != 0
}

/// `_M1` (mbctype.h:44): the MBCS lead-byte bit in `mbcinfo->mbctype[1 + c]`.
const M1: c_int = 0x04;

/// `_ismbblead_l(c, locale)` (mbctype.h:164 / `ismbbyte.rs`'s
/// `x_ismbbtype(locale, c, 0, _M1)`), reimplemented locally rather than
/// depending on the `ismbbyte` feature (which `convert-rest` does not always
/// pull in) -- same table, same bit, same indexing (`mbctype + 1`, so byte 0
/// is real, not padding).
///
/// # Safety
/// `locale` must be a valid, non-null `_locale_t`.
#[inline]
pub unsafe fn is_mbb_lead(c: u8, locale: *mut CrtLocalePointers) -> bool {
    let mbcinfo = (*locale).mbcinfo;
    ((*mbcinfo).mbctype[1 + c as usize] as c_int & M1) != 0
}

/// `__crt_mbstring::return_illegal_sequence` (common_utf8.cpp): reset the
/// state, set `EILSEQ`, and answer `INVALID`. Only this cluster's own UTF-8
/// engines call it (grepped), so it stays a plain function.
pub unsafe fn return_illegal_sequence(ps: *mut MbstateT, ptd: &mut PtdHost) -> usize {
    *ps = MbstateT::zeroed();
    ptd.errno().set(EILSEQ);
    INVALID
}

/// `__crt_mbstring::reset_and_return` (common_utf8.cpp).
pub unsafe fn reset_and_return(retval: usize, ps: *mut MbstateT) -> usize {
    *ps = MbstateT::zeroed();
    retval
}

/// `_UCRT_VALIDATE_RETURN_ERRCODE(ptd, expr, err)`: if `expr` is false, record
/// `err`, invoke the invalid-parameter handler (`PtdHost::invalid_parameter`,
/// which is the ptd-propagation family's `_invalid_parameter_internal` path
/// -- distinct from the plain `ptd.get_errno().set(e)` used for a genuine
/// runtime encoding failure like `EILSEQ`, which does NOT invoke the
/// handler), and answer `Some(err)` for the caller to `return`; `None` when
/// `expr` held and the caller should keep going.
///
/// # Safety
/// `ptd` must be a valid, exclusively-borrowed `PtdHost`.
#[inline]
pub unsafe fn validate_errcode(ptd: &mut PtdHost, expr: bool, err: c_int) -> Option<c_int> {
    if !expr {
        ptd.invalid_parameter(err);
        Some(err)
    } else {
        None
    }
}

/// `_UCRT_VALIDATE_RETURN(ptd, expr, err, ret)`: same as
/// [`validate_errcode`], but the failure return value is independent of
/// `err` (e.g. `mbsrtowcs`'s `(size_t)-1`).
///
/// # Safety
/// Same as [`validate_errcode`].
#[inline]
pub unsafe fn validate_return<T>(ptd: &mut PtdHost, expr: bool, err: c_int, ret: T) -> Option<T> {
    if !expr {
        ptd.invalid_parameter(err);
        Some(ret)
    } else {
        None
    }
}

/// Sets `ptd`'s cached errno to `e` and returns `e` -- the Rust stand-in for
/// the C++ `cached<T>::set()`, which (unlike `ptd_host.rs`'s `Cached::set`)
/// returns the value it just stored. `Cached`'s fields are private to
/// `ptd_host`, so this cluster cannot read the value back out any other way;
/// every call site here already knows the value it is setting, so this is
/// exactly `ptd.get_errno().set(e)` used as an expression in the C.
#[inline]
pub unsafe fn set_errno(ptd: &mut PtdHost, e: c_int) -> c_int {
    ptd.errno().set(e);
    e
}
