//! `convert-rest`: the remaining `sdk/lib/ucrt/convert/` files not already
//! owned by another family (ctype/wctype tables, `LocaleUpdate`, the locale
//! layouts, `strtod.rs`, `tocase.rs`, `ismbc.rs`/`ismbbyte.rs`).
//!
//! ## Files taken this pass
//!
//! `mbrtowc.cpp wcrtomb.cpp mbstowcs.cpp wcstombs.cpp wctomb.cpp mbtowc.cpp
//! mblen.cpp _mbslen.cpp ismbstr.cpp c16rtomb.cpp c32rtomb.cpp mbrtoc16.cpp
//! mbrtoc32.cpp swab.cpp wctrans.cpp fcvt.cpp gcvt.cpp atoldbl.cpp`, plus
//! `common_utf8.cpp` (the two tiny `__crt_mbstring` helpers `mbrtoc32.cpp`
//! and friends share -- nothing outside this cluster calls them).
//!
//! `atoldbl.cpp` (`atoldbl.rs`) reuses `strtod.rs`'s scanning front end
//! (`parse_floating_point_string`, split out of `parse_floating_point`
//! centrally for exactly this) and ports the `_LDBL12` 80-bit-extended bignum
//! assembly (`add_ld12`/`shl_ld12`/`multiply_ld12`, the two power-of-ten
//! tables, `convert_ld12_to_ldouble`) by hand as the dispatch target `strtod.rs`
//! uses `Traits`/`write_result` for -- see `atoldbl.rs`'s module doc for the
//! quirks worth knowing (the `MSB_USHORT`-not-`MSB_ULONG` normalize check,
//! the uncarried rounding-digit increment, `multiply_ld12`'s one-past-the-end
//! scratch write).
//!
//! **NOT TAKEN**: `cvt.cpp`/`cfout.cpp`/`_fptostr.cpp`/`fp_flags.cpp` (the
//! `__acrt_fp_format`/`__acrt_fltout`/`__acrt_fp_strflt_to_string` engine
//! `fcvt.rs`/`gcvt.rs` call INTO, but also shared by `output.rs`'s printf --
//! taking it over is that lane's call, not this one's), and `wctype.cpp`
//! (`iswctype`/`wctype`: already effectively covered by `ctype.rs`'s
//! `iswctype`/`_wctype`; nothing here needed its symbols).
//!
//! ## The `__crt_mbstring` namespace and why one export keeps its C++ name
//!
//! UCRT's UTF-8 helpers (`__mbrtoc32_utf8`, `__mbrtoc16_utf8`,
//! `__c16rtomb_utf8`, `__c32rtomb_utf8`, `__mbrtowc_utf8`, `__wcsrtombs_utf8`,
//! `return_illegal_sequence`, `reset_and_return`) live in
//! `namespace __crt_mbstring` in the header, which is C++ linkage, not
//! `extern "C"`. Grepping `sdk/lib/ucrt` (outside `convert/`) for every one of
//! them found exactly ONE external caller across all of them:
//! `sdk/lib/ucrt/lowio/write.cpp` calls `__crt_mbstring::__mbsrtowcs_utf8`,
//! and that file is not ours and stays C. So `__mbsrtowcs_utf8` alone must be
//! exported under its Itanium-mangled name (mingw/clang use the Itanium C++
//! ABI even on Windows targets, not MSVC mangling) so `write.cpp` keeps
//! linking after `mbrtowc.cpp` is deleted. The mangled names were read off the
//! real compiled objects with `llvm-nm`, not guessed:
//!   * i386:        `__ZN14__crt_mbstring16__mbsrtowcs_utf8EPwPPKcjP9_MbstatetR21__crt_cached_ptd_host`
//!   * amd64/arm64: `_ZN14__crt_mbstring16__mbsrtowcs_utf8EPwPPKcyP9_MbstatetR21__crt_cached_ptd_host`
//! (the only difference is `j` (`unsigned int`, the `len` parameter) on
//! 32-bit vs `y` (`unsigned __int64`) on 64-bit, plus i386's extra leading
//! underscore from cdecl name decoration). See `mbrtowc.rs`.
//!
//! Every OTHER `__crt_mbstring` helper here is called only from within this
//! same batch (verified by grep), so they are plain private Rust functions --
//! no mangling contract to honour.
//!
//! ## House rules
//!
//! Every function is a straight transcription of UCRT's C, quirks called out
//! in doc comments rather than "fixed". `mbstate_t` is `common::MbstateT`,
//! measured layout-identical to `_Mbstatet` (`u32 wchar; u16 byte; u16
//! state;` -- 8 bytes, `unsigned long` is always 32 bits on Windows).

mod atoldbl;
mod common;
mod fcvt;
mod gcvt;
mod ismbstr;
mod mblen;
mod mbrtoc16;
mod mbrtoc32;
mod mbrtowc;
mod mbslen;
mod mbstowcs;
mod mbtowc;
mod swab;
mod wcrtomb;
mod wcstombs;
mod wctomb;
mod wctrans;

// c16rtomb / c32rtomb export only `c16rtomb`/`c32rtomb`; their UTF-8 engines
// live alongside mbrtoc16/mbrtoc32 in `common.rs`'s siblings below.
mod c16rtomb;
mod c32rtomb;
