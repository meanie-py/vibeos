//! UCRT's locale data layout — the substrate every `_l`-suffixed body needs.
//!
//! ## Why this file exists at all
//!
//! 809 of ucrtbase's owed exports already have a Rust body in `msvcrt-crt-rust`,
//! and the obvious move is to reuse them. For every `_l`-suffixed one that is
//! **unsafe**, and silently so: `msvcrt-crt-rust` is bound against msvcrt's own
//! headers, where a `_locale_t` points at `threadlocaleinfostruct`
//! (`sdk/include/vcruntime/crtdefs.h:261`). A caller reaching ucrtbase passes a
//! pointer to `__crt_locale_data` instead, and the two are not variations on a
//! theme — they are different shapes:
//!
//! * msvcrt leads with `refcount, lc_codepage, lc_collate_cp, lc_handle[6],
//!   lc_id[6]`; UCRT leads with a `_public` sub-struct and has no `lc_handle`
//!   or `lc_id` at all.
//! * UCRT adds `lc_time_cp` and a trailing `locale_name[6]`.
//! * `pctype`/`mb_cur_max`/`lc_codepage` are inline members in msvcrt but live
//!   inside `_public` in UCRT.
//!
//! So essentially every field lands somewhere else. Dereferencing a UCRT
//! `_locale_t` through msvcrt's layout reads adjacent memory and keeps going —
//! no fault, no diagnostic, wrong answers. This module is the one place that
//! knows UCRT's shape, so that mistake can only be made here.
//!
//! ## The design rule that follows
//!
//! **The shared core never sees a `_locale_t`.** When a family is lifted into a
//! neutral core crate for both DLLs to use (the `cnt-rust` pattern — see this
//! crate's Cargo.toml), the core takes already-extracted values: a ctype
//! pointer, a codepage, an `mb_cur_max`. Each DLL's `#[no_mangle]` shim does the
//! struct access with its own layout and passes plain values in. That keeps
//! layout knowledge in two small places instead of smeared through every body,
//! and it is what makes one implementation serve two ABIs.
//!
//! ## Where these numbers come from, and why they are written down
//!
//! Measured, not derived: a probe TU emitting `__builtin_offsetof` for every
//! field of the real `struct __crt_locale_data`, compiled with the BUILD'S OWN
//! flags — the exact argv from `build-<arch>/compile_commands.json` for a ucrt
//! TU, source and output swapped. Reconstructing the compiler invocation by
//! hand instead had it silently resolving the toolchain's `winnt.h` rather than
//! ours, which is the sort of answer that looks right and is not.
//!
//! They are recorded here because the ground truth is about to be DELETED.
//! `sdk/lib/ucrt/inc/corecrt_internal.h` is vendored Windows SDK source and goes
//! when the last family that needs it goes; after that there is nothing in the
//! tree to re-derive these from. The `const` assertions below turn the table
//! into something the compiler checks on every build of every arch, so a
//! mistake here is a build error rather than a corrupted read.
//!
//! To re-derive while the header still exists, see the commit that added this
//! file. `_public` sits at offset 0 on all three arches — it is absent from the
//! probe's output only because a zero-valued initialiser lands in `.bss`.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_ushort, c_void};

/// The public head of UCRT's locale data — `__crt_locale_data_public`
/// (`ucrt/corecrt.h:621`). A `_locale_t` is a pointer to a
/// `__crt_locale_pointers` whose first member points here, and these three
/// fields are the ones ordinary locale-aware code actually reads.
#[repr(C)]
pub struct CrtLocaleDataPublic {
    pub locale_pctype: *const c_ushort,
    pub locale_mb_cur_max: c_int,
    pub locale_lc_codepage: c_uint,
}

/// One entry of `lc_category[6]` — `__crt_locale_refcount`.
#[repr(C)]
pub struct CrtLocaleRefcount {
    pub locale: *mut c_char,
    pub wlocale: *mut u16,
    pub refcount: *mut c_long,
    pub wrefcount: *mut c_long,
}

/// UCRT's `struct __crt_locale_data` (`sdk/lib/ucrt/inc/corecrt_internal.h:502`).
///
/// NOT interchangeable with msvcrt's `threadlocaleinfostruct`. See the module
/// docs: the shapes differ from the second field onwards.
impl CrtLocaleData {
    /// The public head, where `_locale_pctype` lives. A method rather than a
    /// bare field access so the read site reads like the C's
    /// `locinfo->_public._locale_pctype`.
    pub fn public_ref(&self) -> *const CrtLocaleDataPublic {
        &self.public
    }
}

#[repr(C)]
pub struct CrtLocaleData {
    pub public: CrtLocaleDataPublic,
    pub refcount: c_long,
    pub lc_collate_cp: c_uint,
    pub lc_time_cp: c_uint,
    pub lc_clike: c_int,
    pub lc_category: [CrtLocaleRefcount; 6],
    pub lconv_intl_refcount: *mut c_long,
    pub lconv_num_refcount: *mut c_long,
    pub lconv_mon_refcount: *mut c_long,
    pub lconv: *mut c_void,
    pub ctype1_refcount: *mut c_long,
    pub ctype1: *mut c_ushort,
    pub pclmap: *const u8,
    pub pcumap: *const u8,
    pub lc_time_curr: *const c_void,
    pub locale_name: [*mut u16; 6],
}


/// UCRT's `struct __crt_locale_pointers` (`ucrt/corecrt.h:628`).
/// **This is what a `_locale_t` actually points at** — the locale data hangs
/// off `locinfo`, and the multibyte data off `mbcinfo`. Code that treats a
/// `_locale_t` as a direct pointer to the locale data is off by one level.
#[repr(C)]
pub struct CrtLocalePointers {
    pub locinfo: *mut CrtLocaleData,
    pub mbcinfo: *mut CrtMultibyteData,
}

/// UCRT's `struct __crt_multibyte_data` (`sdk/lib/ucrt/inc/corecrt_internal.h`).
///
/// `mbctype` is the table the whole `_ismbb*` family indexes, and note it is
/// **257 bytes, indexed at `1 + byte`** — the leading entry is not padding, so
/// an implementation that indexes at `byte` is wrong for every input.
#[repr(C)]
pub struct CrtMultibyteData {
    pub refcount: c_long,
    pub mbcodepage: c_int,
    pub ismbcodepage: c_int,
    pub mbulinfo: [c_ushort; 6],
    pub mbctype: [u8; 257],
    pub mbcasemap: [u8; 256],
    pub mblocalename: *const u16,
}

// ── measured ground truth ────────────────────────────────────────────────────
//
// i386 differs from amd64/arm64 only by pointer width; amd64 and arm64 agree
// exactly. Checked at compile time on whichever arch is being built, so a
// reordered or dropped field cannot reach a link.

macro_rules! assert_offset {
    ($field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!(CrtLocaleData, $field) == $want,
            concat!("__crt_locale_data.", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

#[cfg(target_pointer_width = "32")]
mod pinned {
    use super::*;
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, locinfo) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, mbcinfo) == 4);
    const _: () = assert!(core::mem::size_of::<CrtLocalePointers>() == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, refcount) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcodepage) == 4);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, ismbcodepage) == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbulinfo) == 12);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbctype) == 24);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcasemap) == 281);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mblocalename) == 540);
    const _: () = assert!(core::mem::size_of::<CrtMultibyteData>() == 544);
    assert_offset!(public, 0);
    assert_offset!(refcount, 12);
    assert_offset!(lc_collate_cp, 16);
    assert_offset!(lc_time_cp, 20);
    assert_offset!(lc_clike, 24);
    assert_offset!(lc_category, 28);
    assert_offset!(lconv_intl_refcount, 124);
    assert_offset!(lconv_num_refcount, 128);
    assert_offset!(lconv_mon_refcount, 132);
    assert_offset!(lconv, 136);
    assert_offset!(ctype1_refcount, 140);
    assert_offset!(ctype1, 144);
    assert_offset!(pclmap, 148);
    assert_offset!(pcumap, 152);
    assert_offset!(lc_time_curr, 156);
    assert_offset!(locale_name, 160);
    const _: () = assert!(core::mem::size_of::<CrtLocaleData>() == 184);
}

#[cfg(target_pointer_width = "64")]
mod pinned {
    use super::*;
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, locinfo) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, mbcinfo) == 8);
    const _: () = assert!(core::mem::size_of::<CrtLocalePointers>() == 16);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, refcount) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcodepage) == 4);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, ismbcodepage) == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbulinfo) == 12);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbctype) == 24);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcasemap) == 281);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mblocalename) == 544);
    const _: () = assert!(core::mem::size_of::<CrtMultibyteData>() == 552);
    assert_offset!(public, 0);
    assert_offset!(refcount, 16);
    assert_offset!(lc_collate_cp, 20);
    assert_offset!(lc_time_cp, 24);
    assert_offset!(lc_clike, 28);
    assert_offset!(lc_category, 32);
    assert_offset!(lconv_intl_refcount, 224);
    assert_offset!(lconv_num_refcount, 232);
    assert_offset!(lconv_mon_refcount, 240);
    assert_offset!(lconv, 248);
    assert_offset!(ctype1_refcount, 256);
    assert_offset!(ctype1, 264);
    assert_offset!(pclmap, 272);
    assert_offset!(pcumap, 280);
    assert_offset!(lc_time_curr, 288);
    assert_offset!(locale_name, 296);
    const _: () = assert!(core::mem::size_of::<CrtLocaleData>() == 344);
}

// ── the rest of the locale substrate (the locale takeover's frozen interface) ──
//
// Measured 2026-09-26 the same way as the structs above: a probe TU of
// `offsetof`s compiled with the exact argv compile_commands.json gives a
// sdk/lib/ucrt locale TU, on all three arches. amd64 and arm64 agree exactly.

/// `MAX_LANG_LEN`, `MAX_CTRY_LEN`, `MAX_LC_LEN`, `MAX_CP_LEN`,
/// `LOCALE_NAME_MAX_LENGTH` (corecrt_internal.h, winnls.h).
pub const MAX_LANG_LEN: usize = 64;
pub const MAX_CTRY_LEN: usize = 64;
pub const MAX_LC_LEN: usize = MAX_LANG_LEN + MAX_CTRY_LEN + 3;
pub const MAX_CP_LEN: usize = 16;
pub const LOCALE_NAME_MAX_LENGTH: usize = 85;
/// `CATNAMES_LEN`: the length of "LC_COLLATE=;LC_CTYPE=;...".
pub const CATNAMES_LEN: usize = 57;

/// The `LC_*` category indices (locale.h). `lc_category[]` and
/// `locale_name[]` are indexed by these.
pub const CRT_LC_ALL: c_int = 0;
pub const CRT_LC_COLLATE: c_int = 1;
pub const CRT_LC_CTYPE: c_int = 2;
pub const CRT_LC_MONETARY: c_int = 3;
pub const CRT_LC_NUMERIC: c_int = 4;
pub const CRT_LC_TIME: c_int = 5;
pub const CRT_LC_MIN: c_int = CRT_LC_ALL;
pub const CRT_LC_MAX: c_int = CRT_LC_TIME;

/// `__acrt_GetLocaleInfoA`'s `_LCType` selector.
pub const LC_INT_TYPE: c_int = 0;
pub const LC_STR_TYPE: c_int = 1;
pub const LC_WSTR_TYPE: c_int = 2;

/// `_configthreadlocale` / `__acrt_ptd::_own_locale` bits.
pub const WSETLOCALE_AVOID_SYNC_LOCALE_BIT: c_int = 0x10;
pub const PER_THREAD_LOCALE_BIT: c_int = 0x2;
pub const GLOBAL_LOCALE_BIT: c_int = 0x1;

/// `struct lconv` (locale.h) -- UCRT's, with the eight wide members after the
/// narrow ones.
#[repr(C)]
pub struct Lconv {
    pub decimal_point: *mut c_char,
    pub thousands_sep: *mut c_char,
    pub grouping: *mut c_char,
    pub int_curr_symbol: *mut c_char,
    pub currency_symbol: *mut c_char,
    pub mon_decimal_point: *mut c_char,
    pub mon_thousands_sep: *mut c_char,
    pub mon_grouping: *mut c_char,
    pub positive_sign: *mut c_char,
    pub negative_sign: *mut c_char,
    pub int_frac_digits: c_char,
    pub frac_digits: c_char,
    pub p_cs_precedes: c_char,
    pub p_sep_by_space: c_char,
    pub n_cs_precedes: c_char,
    pub n_sep_by_space: c_char,
    pub p_sign_posn: c_char,
    pub n_sign_posn: c_char,
    pub w_decimal_point: *mut u16,
    pub w_thousands_sep: *mut u16,
    pub w_int_curr_symbol: *mut u16,
    pub w_currency_symbol: *mut u16,
    pub w_mon_decimal_point: *mut u16,
    pub w_mon_thousands_sep: *mut u16,
    pub w_positive_sign: *mut u16,
    pub w_negative_sign: *mut u16,
}

/// `__crt_lc_time_data`.
#[repr(C)]
pub struct LcTimeData {
    pub wday_abbr: [*mut c_char; 7],
    pub wday: [*mut c_char; 7],
    pub month_abbr: [*mut c_char; 12],
    pub month: [*mut c_char; 12],
    pub ampm: [*mut c_char; 2],
    pub ww_sdatefmt: *mut c_char,
    pub ww_ldatefmt: *mut c_char,
    pub ww_timefmt: *mut c_char,
    pub ww_caltype: c_int,
    pub refcount: c_long,
    pub w_wday_abbr: [*mut u16; 7],
    pub w_wday: [*mut u16; 7],
    pub w_month_abbr: [*mut u16; 12],
    pub w_month: [*mut u16; 12],
    pub w_ampm: [*mut u16; 2],
    pub w_ww_sdatefmt: *mut u16,
    pub w_ww_ldatefmt: *mut u16,
    pub w_ww_timefmt: *mut u16,
    pub w_ww_locale_name: *mut u16,
}

/// `__crt_locale_strings`: a parsed locale name.
#[repr(C)]
pub struct LocaleStrings {
    pub language: [u16; MAX_LANG_LEN],
    pub country: [u16; MAX_CTRY_LEN],
    pub code_page: [u16; MAX_CP_LEN],
    pub locale_name: [u16; LOCALE_NAME_MAX_LENGTH],
}

/// `__crt_locale_string_table`: one row of `__acrt_rg_language` /
/// `__acrt_rg_country`.
#[repr(C)]
pub struct LocaleStringTable {
    pub name: *const u16,
    pub abbrev: [u16; 4],
}

/// `__crt_ctype_compatibility_data`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CtypeCompatibilityData {
    pub id: c_ulong,
    pub is_clike: c_int,
}

/// `__crt_qualified_locale_data` -- `__acrt_ptd::_setloc_data`, by value.
#[repr(C)]
pub struct QualifiedLocaleData {
    pub pch_language: *const u16,
    pub pch_country: *const u16,
    pub i_loc_state: c_int,
    pub i_primary_len: c_int,
    pub b_abbrev_language: c_int,
    pub b_abbrev_country: c_int,
    pub cachecp: c_uint,
    pub cachein: [u16; MAX_LC_LEN],
    pub cacheout: [u16; MAX_LC_LEN],
    pub loc_c: [CtypeCompatibilityData; 5],
    pub cache_locale_name: [u16; LOCALE_NAME_MAX_LENGTH],
}

/// `__crt_qualified_locale_data_downlevel` -- `__acrt_ptd::_setloc_downlevel_data`
/// points at one, allocated on first use.
#[repr(C)]
pub struct QualifiedLocaleDataDownlevel {
    pub i_lcid_state: c_int,
    pub lcid_language: u32,
    pub lcid_country: u32,
}

macro_rules! pin_at {
    ($t:ty, $field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!($t, $field) == $want,
            concat!(stringify!($t), ".", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

// Width-independent: every field is u16.
const _: () = assert!(core::mem::size_of::<LocaleStrings>() == 458);
pin_at!(LocaleStrings, country, 128);
pin_at!(LocaleStrings, code_page, 256);
pin_at!(LocaleStrings, locale_name, 288);
const _: () = assert!(core::mem::size_of::<CtypeCompatibilityData>() == 8);
const _: () = assert!(core::mem::size_of::<QualifiedLocaleDataDownlevel>() == 12);

#[cfg(target_pointer_width = "32")]
mod pinned_rest {
    use super::*;
    const _: () = assert!(core::mem::size_of::<Lconv>() == 80);
    pin_at!(Lconv, int_frac_digits, 40);
    pin_at!(Lconv, n_sign_posn, 47);
    pin_at!(Lconv, w_decimal_point, 48);
    pin_at!(Lconv, w_negative_sign, 76);
    const _: () = assert!(core::mem::size_of::<LcTimeData>() == 356);
    pin_at!(LcTimeData, wday, 28);
    pin_at!(LcTimeData, month_abbr, 56);
    pin_at!(LcTimeData, month, 104);
    pin_at!(LcTimeData, ampm, 152);
    pin_at!(LcTimeData, ww_sdatefmt, 160);
    pin_at!(LcTimeData, ww_caltype, 172);
    pin_at!(LcTimeData, refcount, 176);
    pin_at!(LcTimeData, w_wday_abbr, 180);
    pin_at!(LcTimeData, w_ampm, 332);
    pin_at!(LcTimeData, w_ww_sdatefmt, 340);
    pin_at!(LcTimeData, w_ww_locale_name, 352);
    const _: () = assert!(core::mem::size_of::<QualifiedLocaleData>() == 764);
    pin_at!(QualifiedLocaleData, pch_country, 4);
    pin_at!(QualifiedLocaleData, i_loc_state, 8);
    pin_at!(QualifiedLocaleData, cachecp, 24);
    pin_at!(QualifiedLocaleData, cachein, 28);
    pin_at!(QualifiedLocaleData, cacheout, 290);
    pin_at!(QualifiedLocaleData, loc_c, 552);
    pin_at!(QualifiedLocaleData, cache_locale_name, 592);
    const _: () = assert!(core::mem::size_of::<LocaleStringTable>() == 12);
    pin_at!(LocaleStringTable, abbrev, 4);
}

#[cfg(target_pointer_width = "64")]
mod pinned_rest {
    use super::*;
    const _: () = assert!(core::mem::size_of::<Lconv>() == 152);
    pin_at!(Lconv, int_frac_digits, 80);
    pin_at!(Lconv, n_sign_posn, 87);
    pin_at!(Lconv, w_decimal_point, 88);
    pin_at!(Lconv, w_negative_sign, 144);
    const _: () = assert!(core::mem::size_of::<LcTimeData>() == 704);
    pin_at!(LcTimeData, wday, 56);
    pin_at!(LcTimeData, month_abbr, 112);
    pin_at!(LcTimeData, month, 208);
    pin_at!(LcTimeData, ampm, 304);
    pin_at!(LcTimeData, ww_sdatefmt, 320);
    pin_at!(LcTimeData, ww_caltype, 344);
    pin_at!(LcTimeData, refcount, 348);
    pin_at!(LcTimeData, w_wday_abbr, 352);
    pin_at!(LcTimeData, w_ampm, 656);
    pin_at!(LcTimeData, w_ww_sdatefmt, 672);
    pin_at!(LcTimeData, w_ww_locale_name, 696);
    const _: () = assert!(core::mem::size_of::<QualifiedLocaleData>() == 776);
    pin_at!(QualifiedLocaleData, pch_country, 8);
    pin_at!(QualifiedLocaleData, i_loc_state, 16);
    pin_at!(QualifiedLocaleData, cachecp, 32);
    pin_at!(QualifiedLocaleData, cachein, 36);
    pin_at!(QualifiedLocaleData, cacheout, 298);
    pin_at!(QualifiedLocaleData, loc_c, 560);
    pin_at!(QualifiedLocaleData, cache_locale_name, 600);
    const _: () = assert!(core::mem::size_of::<LocaleStringTable>() == 16);
    pin_at!(LocaleStringTable, abbrev, 8);
}
