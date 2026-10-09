use crate::prelude::*;
// NOT `use msvcrt_sys::*;` as well — prelude.rs already re-exports the whole island, and a
// second glob makes every overlapping name E0659-ambiguous. See file.rs for the long form.
use core::ffi::c_void;

// CRT siblings that are ALREADY Rust in this crate. IMPORTED rather than re-declared
// `extern`: both spellings link (they are `#[no_mangle]`), but only the import makes rustc
// check the call against the definition.
use crate::msvcrt::mbcs::{create_mbcinfo, _setmbcp};
use crate::{atoi, wcscpy, wcslen, _wcsicmp, _wcsnicmp};

extern "C" {
    // <stdlib.h>. Still C, and reached by symbol rather than by `use` — see the note on the
    // same declaration in errno.rs.
    fn wcstombs(mbstr: *mut core::ffi::c_char, wcstr: *const u16, count: usize) -> usize;
}

// locale.c's wine-debug diagnostics. `macro_rules!`, never a `fn` — an out-of-line no-op is
// an optimization barrier that has uncloaked latent ABI mismatches into `ud2`. The call sites
// are kept (that is what `scan_dropped_prints` grades): a diagnostic deleted from the source
// is one nobody can ever turn back on.

// ---- file-scope preamble (translated) ----
pub const MAX_ELEM_LEN: usize = 64;
pub const MAX_LOCALE_LENGTH: usize = 256;

/// locale.c:43-49 own the CRT's exported locale state, and all seven are GLOBAL in
/// `locale.c.obj` — `MSVCRT_locale`, `MSVCRT__pctype`, `MSVCRT___lc_codepage`,
/// `MSVCRT___lc_collate_cp`, `MSVCRT___lc_handle`, `MSVCRT___mb_cur_max` and
/// `initial_locale` — as is `cloc_time_data` below. The translation emitted all eight as
/// PRIVATE Rust statics, which compiles and links and would have left locale.c.obj as the
/// only definition of each, pulling that object back in and duplicating everything else in
/// this file. Same class as exit.c's `_aexit_rtn`; `llvm-nm --defined-only` on the object is
/// what says so, never the `static` keywords in the source.
#[no_mangle]
pub static mut MSVCRT_locale: _locale_t = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT__pctype: *mut u16 = core::ptr::null_mut();
#[no_mangle]
pub static mut MSVCRT___lc_codepage: u32 = 0;
#[no_mangle]
pub static mut MSVCRT___lc_collate_cp: i32 = 0;
#[no_mangle]
pub static mut MSVCRT___lc_handle: [LCID; 6] = [0; 6];
#[no_mangle]
pub static mut MSVCRT___mb_cur_max: i32 = 1;
#[no_mangle]
pub static mut initial_locale: BOOL = 1;

pub const MSVCRT_LEADBYTE: u16 = 0x8000;
pub const MSVCRT_C1_DEFINED: u16 = 0x200;
pub const LCID_CONVERSION_FLAGS: u32 = 0;

#[no_mangle]
pub static mut cloc_time_data: __lc_time_data = __lc_time_data {
    str_: __lc_time_data__bindgen_ty_1 { str_: [b"Sun\0".as_ptr() as *mut i8, b"Mon\0".as_ptr() as *mut i8, b"Tue\0".as_ptr() as *mut i8, b"Wed\0".as_ptr() as *mut i8, b"Thu\0".as_ptr() as *mut i8, b"Fri\0".as_ptr() as *mut i8, b"Sat\0".as_ptr() as *mut i8,
      b"Sunday\0".as_ptr() as *mut i8, b"Monday\0".as_ptr() as *mut i8, b"Tuesday\0".as_ptr() as *mut i8, b"Wednesday\0".as_ptr() as *mut i8, b"Thursday\0".as_ptr() as *mut i8, b"Friday\0".as_ptr() as *mut i8, b"Saturday\0".as_ptr() as *mut i8,
      b"Jan\0".as_ptr() as *mut i8, b"Feb\0".as_ptr() as *mut i8, b"Mar\0".as_ptr() as *mut i8, b"Apr\0".as_ptr() as *mut i8, b"May\0".as_ptr() as *mut i8, b"Jun\0".as_ptr() as *mut i8, b"Jul\0".as_ptr() as *mut i8, b"Aug\0".as_ptr() as *mut i8, b"Sep\0".as_ptr() as *mut i8, b"Oct\0".as_ptr() as *mut i8, b"Nov\0".as_ptr() as *mut i8, b"Dec\0".as_ptr() as *mut i8,
      b"January\0".as_ptr() as *mut i8, b"February\0".as_ptr() as *mut i8, b"March\0".as_ptr() as *mut i8, b"April\0".as_ptr() as *mut i8, b"May\0".as_ptr() as *mut i8, b"June\0".as_ptr() as *mut i8, b"July\0".as_ptr() as *mut i8,
      b"August\0".as_ptr() as *mut i8, b"September\0".as_ptr() as *mut i8, b"October\0".as_ptr() as *mut i8, b"November\0".as_ptr() as *mut i8, b"December\0".as_ptr() as *mut i8,
      b"AM\0".as_ptr() as *mut i8, b"PM\0".as_ptr() as *mut i8, b"MM/dd/yy\0".as_ptr() as *mut i8, b"dddd, MMMM dd, yyyy\0".as_ptr() as *mut i8, b"HH:mm:ss\0".as_ptr() as *mut i8] },
    // msvcrt.h:105-112 at `_MSVCR_VER=0` is `lcid, unk, refcount` — and the C initializer
    // supplies `MAKELCID(LANG_ENGLISH, SORT_DEFAULT), 1, -1` in that order. This arrived as
    // `lcid: 0x0409, wrefcount: 1, unk: -1`: a field that does not exist at this version
    // (`wrefcount` is the `>= 110` spelling), which shifted `1` and `-1` one slot each, and
    // an LCID that is a real value but the wrong one. `refcount = -1` is load-bearing — it
    // is the "statically allocated, never free me" sentinel `free_locinfo` tests, so landing
    // `1` there instead makes the C locale's time data look refcounted and freeable.
    lcid: MAKELCID!(LANG_ENGLISH, SORT_DEFAULT),
    unk: 1,
    refcount: -1,
    wstr: __lc_time_data__bindgen_ty_2 { wstr: [win32macros_rust::wide_bytes!(b"S\0u\0n\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0o\0n\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"T\0u\0e\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"W\0e\0d\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"T\0h\0u\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"F\0r\0i\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"S\0a\0t\0\0\0") as *mut wchar_t,
      win32macros_rust::wide_bytes!(b"S\0u\0n\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0o\0n\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"T\0u\0e\0s\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"W\0e\0d\0n\0e\0s\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"T\0h\0u\0r\0s\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"F\0r\0i\0d\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"S\0a\0t\0u\0r\0d\0a\0y\0\0\0") as *mut wchar_t,
      win32macros_rust::wide_bytes!(b"J\0a\0n\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"F\0e\0b\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0a\0r\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"A\0p\0r\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"J\0u\0n\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"J\0u\0l\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"A\0u\0g\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"S\0e\0p\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"O\0c\0t\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"N\0o\0v\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"D\0e\0c\0\0\0") as *mut wchar_t,
      win32macros_rust::wide_bytes!(b"J\0a\0n\0u\0a\0r\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"F\0e\0b\0r\0u\0a\0r\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0a\0r\0c\0h\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"A\0p\0r\0i\0l\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0a\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"J\0u\0n\0e\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"J\0u\0l\0y\0\0\0") as *mut wchar_t,
      win32macros_rust::wide_bytes!(b"A\0u\0g\0u\0s\0t\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"S\0e\0p\0t\0e\0m\0b\0e\0r\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"O\0c\0t\0o\0b\0e\0r\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"N\0o\0v\0e\0m\0b\0e\0r\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"D\0e\0c\0e\0m\0b\0e\0r\0\0\0") as *mut wchar_t,
      win32macros_rust::wide_bytes!(b"A\0M\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"P\0M\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"M\0M\0/\0d\0d\0/\0y\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"d\0d\0d\0d\0,\0 \0M\0M\0M\0M\0 \0d\0d\0,\0 \0y\0y\0y\0y\0\0\0") as *mut wchar_t, win32macros_rust::wide_bytes!(b"H\0H\0:\0m\0m\0:\0s\0s\0\0\0") as *mut wchar_t] },
    // The trailing flexible-array member; the C initializer omits it, so it is zero.
    data: [0; 1],
};

pub static cloc_clmap: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
    0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
    0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
    0x78, 0x79, 0x7a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
    0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
    0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
    0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
];

pub static cloc_cumap: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
    0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
    0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
    0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
    0x58, 0x59, 0x5a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
    0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
];

pub static mut empty: [i8; 1] = [0];
pub static mut cloc_dec_point: [i8; 2] = [b'.' as i8, 0];

pub static mut cloc_lconv: lconv = lconv {
    decimal_point: unsafe { (&raw const cloc_dec_point).cast::<i8>() as *mut i8 },
    thousands_sep: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    grouping: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    int_curr_symbol: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    currency_symbol: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    mon_decimal_point: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    mon_thousands_sep: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    mon_grouping: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    positive_sign: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    negative_sign: unsafe { (&raw const empty).cast::<i8>() as *mut i8 },
    int_frac_digits: 127,
    frac_digits: 127,
    p_cs_precedes: 127,
    p_sep_by_space: 127,
    n_cs_precedes: 127,
    n_sep_by_space: 127,
    p_sign_posn: 127,
    n_sign_posn: 127,
};

pub static mut _country_synonyms: [*const i8; 70] = [
    b"american\0".as_ptr() as *const i8, b"en\0".as_ptr() as *const i8,
    b"american english\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"american-english\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"english-american\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"english-us\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"english-usa\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"us\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"usa\0".as_ptr() as *const i8, b"en-US\0".as_ptr() as *const i8,
    b"australian\0".as_ptr() as *const i8, b"en-AU\0".as_ptr() as *const i8,
    b"english-aus\0".as_ptr() as *const i8, b"en-AU\0".as_ptr() as *const i8,
    b"belgian\0".as_ptr() as *const i8, b"nl-BE\0".as_ptr() as *const i8,
    b"french-belgian\0".as_ptr() as *const i8, b"fr-BE\0".as_ptr() as *const i8,
    b"canadian\0".as_ptr() as *const i8, b"en-CA\0".as_ptr() as *const i8,
    b"english-can\0".as_ptr() as *const i8, b"en-CA\0".as_ptr() as *const i8,
    b"french-canadian\0".as_ptr() as *const i8, b"fr-CA\0".as_ptr() as *const i8,
    b"chinese\0".as_ptr() as *const i8, b"zh-CN\0".as_ptr() as *const i8,
    b"chinese-simplified\0".as_ptr() as *const i8, b"zh-CN\0".as_ptr() as *const i8,
    b"chinese-traditional\0".as_ptr() as *const i8, b"zh-TW\0".as_ptr() as *const i8,
    b"chs\0".as_ptr() as *const i8, b"zh-CN\0".as_ptr() as *const i8,
    b"cht\0".as_ptr() as *const i8, b"zh-TW\0".as_ptr() as *const i8,
    b"dutch-belgian\0".as_ptr() as *const i8, b"nl-BE\0".as_ptr() as *const i8,
    b"english-nz\0".as_ptr() as *const i8, b"en-NZ\0".as_ptr() as *const i8,
    b"uk\0".as_ptr() as *const i8, b"en-GB\0".as_ptr() as *const i8,
    b"english-uk\0".as_ptr() as *const i8, b"en-GB\0".as_ptr() as *const i8,
    b"french-swiss\0".as_ptr() as *const i8, b"fr-CH\0".as_ptr() as *const i8,
    b"swiss\0".as_ptr() as *const i8, b"de-CH\0".as_ptr() as *const i8,
    b"german-swiss\0".as_ptr() as *const i8, b"de-CH\0".as_ptr() as *const i8,
    b"italian-swiss\0".as_ptr() as *const i8, b"it-CH\0".as_ptr() as *const i8,
    b"german-austrian\0".as_ptr() as *const i8, b"de-AT\0".as_ptr() as *const i8,
    b"portuguese\0".as_ptr() as *const i8, b"pt-BR\0".as_ptr() as *const i8,
    b"portuguese-brazil\0".as_ptr() as *const i8, b"pt-BR\0".as_ptr() as *const i8,
    b"spanish-mexican\0".as_ptr() as *const i8, b"es-MX\0".as_ptr() as *const i8,
    b"norwegian-bokmal\0".as_ptr() as *const i8, b"nb\0".as_ptr() as *const i8,
    b"norwegian-nynorsk\0".as_ptr() as *const i8, b"nn-NO\0".as_ptr() as *const i8,
    b"spanish-modern\0".as_ptr() as *const i8, b"es-ES\0".as_ptr() as *const i8,
];

pub const FOUND_SNAME: u32 = 0x4;
pub const FOUND_LANGUAGE: u32 = 0x2;
pub const FOUND_COUNTRY: u32 = 0x1;

#[repr(C)]
pub struct locale_search_t {
    pub search_language: [u16; MAX_ELEM_LEN],
    pub search_country: [u16; MAX_ELEM_LEN],
    pub found_lang_sname: [u16; (LOCALE_NAME_MAX_LENGTH) as usize],
    pub match_flags: u32,
    pub allow_sname: u8,
}

// locale.c:251-252 — `#define CONTINUE_LOOKING TRUE` / `STOP_LOOKING FALSE`, and they are
// the callback's BOOL result, not a byte flag.
pub const CONTINUE_LOOKING: BOOL = 1;
pub const STOP_LOOKING: BOOL = 0;

pub static mut cloc_static_locinfo: threadlocinfo = threadlocinfo {
    refcount: 0x7FFFFFFF,
    lc_codepage: 0,
    lc_collate_cp: 0,
    lc_handle: [0; 6],
    lc_id: [tagLC_ID {
        wLanguage: 0,
        wCountry: 0,
        wCodePage: 0,
    }; 6],
    lc_category: [threadlocaleinfostruct__bindgen_ty_1 {
        locale: core::ptr::null_mut(),
        wlocale: core::ptr::null_mut(),
        refcount: core::ptr::null_mut(),
        wrefcount: core::ptr::null_mut(),
    }; 6],
    lc_clike: 1,
    mb_cur_max: 1,
    lconv_intl_refcount: core::ptr::null_mut(),
    lconv_num_refcount: core::ptr::null_mut(),
    lconv_mon_refcount: core::ptr::null_mut(),
    lconv: core::ptr::null_mut(),
    ctype1_refcount: core::ptr::null_mut(),
    ctype1: core::ptr::null_mut(),
    pctype: core::ptr::null_mut(),
    pclmap: core::ptr::null_mut(),
    pcumap: core::ptr::null_mut(),
    lc_time_curr: core::ptr::null_mut(),
};

pub static mut cloc_static_mbcinfo: threadmbcinfo = threadmbcinfo {
    refcount: 0x7FFFFFFF,
    mbcodepage: 0,
    ismbcodepage: 0,
    mblcid: 0,
    mbulinfo: [0; 6],
    mbctype: [0; 257],
    mbcasemap: [0; 256],
};

pub static mut cloc_static_initialized: i32 = 0;
unsafe extern "C" fn remap_synonym(name: PCHAR) -> BOOL {
    let mut i: u32 = 0;
    while (i as usize) < ARRAY_SIZE!(_country_synonyms) {
        if _stricmp(_country_synonyms[i as usize], name) == 0 {
            strcpy(name, _country_synonyms[(i + 1) as usize]);
            return (TRUE) as i32;
        }
        i += 2;
    }
    (FALSE) as i32
}
unsafe extern "C" fn compare_info(name: *mut u16, flags: u32, buff: *mut u16, cmp: *const u16, exact: u8) -> i32 {
    let len: i32;

    if (*cmp.offset(0)) == 0 {
        return 0;
    }

    *buff.offset(0) = 0;
    GetLocaleInfoEx(name, flags | LOCALE_NOUSEROVERRIDE, buff, (MAX_ELEM_LEN) as i32);
    if (*buff.offset(0)) == 0 {
        return 0;
    }

    len = (wcslen(cmp)) as i32;

    if exact != 0 || len <= 3 {
        ((_wcsicmp(cmp, buff) == 0) as i32)
    } else {
        ((_wcsnicmp(cmp, buff, (len) as usize) == 0) as i32)
    }
}
// `CALLBACK` in the C, i.e. `__stdcall` on i386 and cdecl everywhere else — which is
// precisely what Rust spells `extern "system"`. The translation arrived as TWO
// `#[cfg(target_arch)]`-split copies of one identical body, one `stdcall` and one `C`;
// they are collapsed here because `system` already IS that conditional, and keeping two
// copies means the next edit lands in one of them.
//
// The return type is `BOOL`, not the `u8` it arrived as, and that one is not cosmetic:
// `EnumSystemLocalesEx` tests the callback's full 32-bit result, so returning a `u8`
// leaves the upper 24 bits of the return register undefined and `STOP_LOOKING` (0) can
// read as nonzero — the enumeration would then never stop early, silently scanning every
// locale on the system and taking the LAST match instead of the best one.
macro_rules! define_find_best_locale_proc {
    ($abi:literal) => {
unsafe extern $abi fn find_best_locale_proc(
    name: *mut u16,
    locale_flags: u32,
    lParam: LPARAM,
) -> BOOL {
    let res = lParam as *mut locale_search_t;
    let mut buff: [u16; MAX_ELEM_LEN] = core::mem::zeroed();
    let mut flags = 0;

    if (*res).allow_sname != 0 && compare_info(
        name,
        LOCALE_SNAME,
        buff.as_mut_ptr(),
        (*res).search_language.as_ptr(),
        (TRUE) as u8,
    ) != 0
    {
        (*res).match_flags = FOUND_SNAME;
        wcscpy((*res).found_lang_sname.as_mut_ptr(), name);
        return STOP_LOOKING;
    }

    if compare_info(
        name,
        LOCALE_SISO639LANGNAME,
        buff.as_mut_ptr(),
        (*res).search_language.as_ptr(),
        (TRUE) as u8,
    ) != 0
        || compare_info(
            name,
            LOCALE_SABBREVLANGNAME,
            buff.as_mut_ptr(),
            (*res).search_language.as_ptr(),
            (TRUE) as u8,
        ) != 0
        || compare_info(
            name,
            LOCALE_SENGLANGUAGE,
            buff.as_mut_ptr(),
            (*res).search_language.as_ptr(),
            FALSE,
        ) != 0
    {
        flags |= FOUND_LANGUAGE;
    } else if ((*res).match_flags & FOUND_LANGUAGE) != 0 {
        return CONTINUE_LOOKING;
    }

    if compare_info(
        name,
        LOCALE_SISO3166CTRYNAME,
        buff.as_mut_ptr(),
        (*res).search_country.as_ptr(),
        (TRUE) as u8,
    ) != 0
        || compare_info(
            name,
            LOCALE_SABBREVCTRYNAME,
            buff.as_mut_ptr(),
            (*res).search_country.as_ptr(),
            (TRUE) as u8,
        ) != 0
        || compare_info(
            name,
            LOCALE_SENGCOUNTRY,
            buff.as_mut_ptr(),
            (*res).search_country.as_ptr(),
            FALSE,
        ) != 0
    {
        flags |= FOUND_COUNTRY;
    } else if flags == 0 && ((*res).match_flags & FOUND_COUNTRY) != 0 {
        return CONTINUE_LOOKING;
    }

    if flags > (*res).match_flags {
        (*res).match_flags = flags;
        wcscpy((*res).found_lang_sname.as_mut_ptr(), name);
    }
    if (flags & (FOUND_LANGUAGE | FOUND_COUNTRY))
        == (FOUND_LANGUAGE | FOUND_COUNTRY)
    {
        return STOP_LOOKING;
    }
    return CONTINUE_LOOKING;
}
    };
}

// `LOCALE_ENUMPROCEX` is `extern "stdcall"` in the i386 island and `extern "C"` in the other
// two, because bindgen writes the ABI it saw rather than the `system` alias. `extern "system"`
// IS that same ABI on every one of these targets, but Rust does not UNIFY the two spellings
// nominally, so a `system` fn cannot be passed where bindgen's type is expected — and there is
// no legal `as` cast between fn pointers of differing ABI strings either. Hence the macro: the
// body is written exactly once, and only the ABI token varies.
#[cfg(target_arch = "x86")]
define_find_best_locale_proc!("stdcall");
#[cfg(not(target_arch = "x86"))]
define_find_best_locale_proc!("C");

#[no_mangle]
pub unsafe extern "C" fn locale_to_sname(locale: *const i8, codepage: *mut u16, sname_match: *mut BOOL, sname: *mut wchar_t) -> BOOL {
    let data = msvcrt_get_thread_data();

    let cp: *const i8;
    let region: *const i8;
    let mut is_sname = FALSE;
    // Zeroed: GetLocaleInfoEx fills it, but Rust forbids `addr_of_mut!` on an
    // uninitialized binding, and the C reads it on the path where that call fails.
    let mut locale_cp: u32 = 0;

    if strcmp(locale, (*data).cached_locale.as_ptr()) == 0 {
        if !codepage.is_null() {
            *codepage = ((*data).cached_cp) as u16;
        }
        if !sname_match.is_null() {
            *sname_match = (*data).cached_sname_match;
        }
        wcscpy(sname, (*data).cached_sname.as_ptr());
        return (TRUE) as i32;
    }

    cp = strchr(locale, '.' as i32);
    region = strchr(locale, '_' as i32);

    if *locale.offset(0) == 0 || (cp == locale && region.is_null()) {
        GetUserDefaultLocaleName(sname, (LOCALE_NAME_MAX_LENGTH) as i32);
    } else {
        let mut search_language_buf: [i8; MAX_ELEM_LEN] = [0; MAX_ELEM_LEN];
        let mut search_country_buf: [i8; MAX_ELEM_LEN] = [0; MAX_ELEM_LEN];
        let mut search: locale_search_t = core::mem::zeroed();
        let mut remapped;

        memset(core::ptr::addr_of_mut!(search) as *mut core::ffi::c_void, 0, core::mem::size_of::<locale_search_t>());
        lstrcpynA(search_language_buf.as_mut_ptr(), locale, (MAX_ELEM_LEN) as i32);
        if !region.is_null() {
            lstrcpynA(search_country_buf.as_mut_ptr(), region.offset(1), (MAX_ELEM_LEN) as i32);
            if (region as usize).wrapping_sub(locale as usize) < MAX_ELEM_LEN {
                search_language_buf[(region as usize).wrapping_sub(locale as usize)] = b'\0' as i8;
            }
        } else {
            search_country_buf[0] = b'\0' as i8;
        }

        if !cp.is_null() {
            if !region.is_null() && (cp as usize).wrapping_sub(region as usize).wrapping_sub(1) < MAX_ELEM_LEN {
                search_country_buf[(cp as usize).wrapping_sub(region as usize).wrapping_sub(1)] = b'\0' as i8;
            }
            if (cp as usize).wrapping_sub(locale as usize) < MAX_ELEM_LEN {
                search_language_buf[(cp as usize).wrapping_sub(locale as usize)] = b'\0' as i8;
            }
        }

        remapped = (remap_synonym(search_language_buf.as_mut_ptr())) as u8;
        if remapped != 0 {
            search.allow_sname = (TRUE) as u8;
        }

        MultiByteToWideChar(CP_ACP, 0, search_language_buf.as_ptr(), -1, search.search_language.as_mut_ptr(), (MAX_ELEM_LEN) as i32);
        if search.allow_sname != 0 && IsValidLocaleName(search.search_language.as_ptr()) != 0 {
            search.match_flags = FOUND_SNAME;
            wcscpy(sname, search.search_language.as_ptr());
        } else {
            MultiByteToWideChar(CP_ACP, 0, search_country_buf.as_ptr(), -1, search.search_country.as_mut_ptr(), (MAX_ELEM_LEN) as i32);
            EnumSystemLocalesEx(
                Some(find_best_locale_proc),
                0,
                core::ptr::addr_of_mut!(search) as LPARAM,
                core::ptr::null_mut(),
            );

            if search.match_flags == 0 {
                return (FALSE) as i32;
            }

            if search.search_language[0] != 0 && (search.match_flags & (FOUND_SNAME | FOUND_LANGUAGE)) == 0 {
                return (FALSE) as i32;
            }
            if search.search_country[0] != 0 && (search.match_flags & FOUND_COUNTRY) == 0 {
                return (FALSE) as i32;
            }

            wcscpy(sname, search.found_lang_sname.as_ptr());
        }

        is_sname = if remapped == 0 && (search.match_flags & FOUND_SNAME) != 0 { 1 } else { 0 };
    }

    if cp.is_null() || *cp.offset(1) == 0 || _strnicmp(cp, b".ACP\0".as_ptr() as *const i8, 4) == 0 {
        GetLocaleInfoEx(sname, LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER, core::ptr::addr_of_mut!(locale_cp) as *mut wchar_t, (core::mem::size_of::<u32>() / core::mem::size_of::<wchar_t>()) as i32);
        if locale_cp == 0 {
            locale_cp = GetACP();
        }
    } else if _strnicmp(cp, b".OCP\0".as_ptr() as *const i8, 4) == 0 {
        GetLocaleInfoEx(sname, LOCALE_IDEFAULTCODEPAGE | LOCALE_RETURN_NUMBER, core::ptr::addr_of_mut!(locale_cp) as *mut wchar_t, (core::mem::size_of::<u32>() / core::mem::size_of::<wchar_t>()) as i32);
    } else {
        locale_cp = atoi((cp.offset(1)).cast()) as u32;
    }
    if IsValidCodePage(locale_cp) == 0 {
        return (FALSE) as i32;
    }

    if locale_cp == 0 {
        return (FALSE) as i32;
    }

    if !codepage.is_null() {
        *codepage = locale_cp as u16;
    }
    if !sname_match.is_null() {
        *sname_match = (is_sname) as i32;
    }

    if strlen(locale) < core::mem::size_of_val(&(*data).cached_locale) {
        strcpy((*data).cached_locale.as_mut_ptr(), locale);
        (*data).cached_cp = (locale_cp as u16) as u32;
        (*data).cached_sname_match = (is_sname) as i32;
        wcscpy((*data).cached_sname.as_mut_ptr(), sname);
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn copy_threadlocinfo_category(
    locinfo: pthreadlocinfo,
    old_locinfo: *const threadlocinfo,
    category: i32,
) {
    let cat_idx = category as usize;
    (*locinfo).lc_handle[cat_idx] = (*old_locinfo).lc_handle[cat_idx];
    (*locinfo).lc_id[cat_idx] = (*old_locinfo).lc_id[cat_idx];
    if (*locinfo).lc_category[cat_idx].locale.is_null() {
        (*locinfo).lc_category[cat_idx].locale = (*old_locinfo).lc_category[cat_idx].locale;
        (*locinfo).lc_category[cat_idx].refcount = (*old_locinfo).lc_category[cat_idx].refcount;
        InterlockedIncrement((*locinfo).lc_category[cat_idx].refcount as *mut i32);
    }
}
unsafe extern "C" fn init_category_name(
    name: *const i8,
    len: i32,
    locinfo: pthreadlocinfo,
    category: i32,
) -> u8 {
    (*locinfo).lc_category[category as usize].locale = malloc((len + 1) as usize) as *mut i8;
    (*locinfo).lc_category[category as usize].refcount = malloc(core::mem::size_of::<i32>()) as *mut i32;
    if (*locinfo).lc_category[category as usize].locale.is_null()
        || (*locinfo).lc_category[category as usize].refcount.is_null()
    {
        free((*locinfo).lc_category[category as usize].locale as *mut core::ffi::c_void);
        free((*locinfo).lc_category[category as usize].refcount as *mut core::ffi::c_void);
        (*locinfo).lc_category[category as usize].locale = core::ptr::null_mut();
        (*locinfo).lc_category[category as usize].refcount = core::ptr::null_mut();
        return FALSE;
    }

    memcpy(
        (*locinfo).lc_category[category as usize].locale as *mut core::ffi::c_void,
        name as *const core::ffi::c_void,
        len as usize,
    );
    *(*locinfo).lc_category[category as usize].locale.offset(len as isize) = 0;
    *(*locinfo).lc_category[category as usize].refcount = 1;
    (TRUE) as u8
}

unsafe extern "C" fn set_lc_locale_name(
    locinfo: pthreadlocinfo,
    cat: i32,
    sname: *mut WCHAR,
) -> BOOL {
    (TRUE) as i32
}
unsafe extern "C" fn update_threadlocinfo_category(
    sname: *mut WCHAR,
    cp: u16,
    locinfo: pthreadlocinfo,
    category: i32,
) -> u8 {
    let mut wbuf: [WCHAR; 256] = [0; 256];
    let mut p: *mut WCHAR;

    if GetLocaleInfoEx(
        sname,
        LOCALE_ILANGUAGE | LOCALE_NOUSEROVERRIDE,
        wbuf.as_mut_ptr(),
        ARRAY_SIZE!(wbuf) as i32,
    ) != 0
    {
        p = wbuf.as_mut_ptr();

        (*locinfo).lc_id[category as usize].wLanguage = 0;
        while *p != 0 {
            (*locinfo).lc_id[category as usize].wLanguage *= 16;

            if *p <= '9' as u16 {
                (*locinfo).lc_id[category as usize].wLanguage += *p - '0' as u16;
            } else {
                (*locinfo).lc_id[category as usize].wLanguage += *p - 'a' as u16 + 10;
            }

            p = p.offset(1);
        }

        (*locinfo).lc_id[category as usize].wCountry = (*locinfo).lc_id[category as usize].wLanguage;
    }

    (*locinfo).lc_id[category as usize].wCodePage = cp;

    (*locinfo).lc_handle[category as usize] = LocaleNameToLCID(sname, LCID_CONVERSION_FLAGS);

    set_lc_locale_name(locinfo, category, sname);

    if (*locinfo).lc_category[category as usize].locale.is_null() {
        let mut buf: [i8; 256] = [0; 256];
        let mut len: i32 = 0;

        if LANGIDFROMLCID!((*locinfo).lc_handle[category as usize]) == MAKELANGID!(LANG_NORWEGIAN, SUBLANG_NORWEGIAN_NYNORSK) {
            wcscpy(wbuf.as_mut_ptr(), win32macros_rust::wide_bytes!(b"N\0o\0r\0w\0e\0g\0i\0a\0n\0-\0N\0y\0n\0o\0r\0s\0k\0\0\0") as *const WCHAR);
            len = wcslen(wbuf.as_ptr()) as i32 + 1;
        } else {
            len += GetLocaleInfoEx(
                sname,
                LOCALE_SENGLANGUAGE | LOCALE_NOUSEROVERRIDE,
                wbuf.as_mut_ptr(),
                ARRAY_SIZE!(wbuf) as i32,
            );
        }
        wbuf[(len - 1) as usize] = '_' as u16;
        len += GetLocaleInfoEx(
            sname,
            LOCALE_SENGCOUNTRY | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr().offset(len as isize),
            ARRAY_SIZE!(wbuf) as i32 - len,
        );
        wbuf[(len - 1) as usize] = '.' as u16;
        // `_snwprintf`, not `swprintf`: the C's `swprintf` here is corecrt_wstdio.h's
        // `static inline` over `_vsnwprintf` (the object imports `__vsnwprintf` and no
        // `swprintf`), and the DLL's exported `swprintf` is the non-conforming 2-argument
        // `_swprintf`. `_snwprintf` is the variadic form of the same `_vsnwprintf`.
        _snwprintf(
            wbuf.as_mut_ptr().offset(len as isize),
            ARRAY_SIZE!(wbuf) - len as usize,
            win32macros_rust::wide_bytes!(b"%\0d\0\0\0") as *const WCHAR,
            cp as core::ffi::c_int,
        );
        len += wcslen(wbuf.as_ptr().offset(len as isize)) as i32;

        WideCharToMultiByte(
            cp as u32,
            0,
            wbuf.as_ptr(),
            -1,
            buf.as_mut_ptr(),
            ARRAY_SIZE!(buf) as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        return init_category_name(buf.as_ptr(), len, locinfo, category);
    }

    return (TRUE) as u8;
}

#[no_mangle]
pub unsafe extern "C" fn _lock_locales() {
    _lock((_SETLOCALE_LOCK) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn _unlock_locales() {
    _unlock((_SETLOCALE_LOCK) as i32);
}
unsafe extern "C" fn grab_locinfo(locinfo: pthreadlocinfo) {
    let mut i: i32;

    InterlockedIncrement(core::ptr::addr_of_mut!((*locinfo).refcount) as *mut i32);
    i = (LC_MIN + 1) as i32;
    while i <= (LC_MAX) as i32 {
        if !(*locinfo).lc_category[i as usize].refcount.is_null() {
            InterlockedIncrement((*locinfo).lc_category[i as usize].refcount as *mut i32);
        }
        if !(*locinfo).lc_category[i as usize].wrefcount.is_null() {
            InterlockedIncrement((*locinfo).lc_category[i as usize].wrefcount as *mut i32);
        }
        i += 1;
    }
    if !(*locinfo).lconv_intl_refcount.is_null() {
        InterlockedIncrement((*locinfo).lconv_intl_refcount as *mut i32);
    }
    if !(*locinfo).lconv_num_refcount.is_null() {
        InterlockedIncrement((*locinfo).lconv_num_refcount as *mut i32);
    }
    if !(*locinfo).lconv_mon_refcount.is_null() {
        InterlockedIncrement((*locinfo).lconv_mon_refcount as *mut i32);
    }
    if !(*locinfo).ctype1_refcount.is_null() {
        InterlockedIncrement((*locinfo).ctype1_refcount as *mut i32);
    }
    if !(*locinfo).lc_time_curr.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*locinfo).lc_time_curr).refcount) as *mut i32);
    }
}
unsafe extern "C" fn ensure_cloc_static_init() {
    // Fast path FIRST: the CAS below is a `lock cmpxchg` -- a shared-line
    // read-modify-write and a full barrier -- and without this it ran on EVERY
    // call, not just the first. `get_locinfo()` reaches here once per CHARACTER
    // whenever no locale has been set (`MSVCRT_locale` NULL, so
    // `update_thread_locale` returns at its first line and `data->locinfo` stays
    // NULL), which put a per-character atomic on the `towlower`/`towupper` and
    // `isw*` hot paths. `tolower` never shows it because it short-circuits on
    // `initial_locale` and never calls `get_locinfo` at all.
    //
    // The flag is set once and never cleared, so a plain read settles it. This
    // does not change the existing race window: a thread that LOSES the CAS
    // already returned immediately while the winner was still writing the
    // fields below, so reading the flag early observes exactly what losing the
    // CAS observed before.
    if core::ptr::read_volatile(core::ptr::addr_of!(cloc_static_initialized)) != 0 {
        return;
    }
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!(cloc_static_initialized),
        1,
        0,
    ) == 0
    {
        cloc_static_locinfo.lconv = core::ptr::addr_of_mut!(cloc_lconv);
        cloc_static_locinfo.pctype = (&raw mut MSVCRT__ctype).cast::<WORD>().offset(1);
        cloc_static_locinfo.pclmap = cloc_clmap.as_ptr();
        cloc_static_locinfo.pcumap = cloc_cumap.as_ptr();
        cloc_static_locinfo.lc_time_curr = core::ptr::addr_of_mut!(cloc_time_data);
    }
}
unsafe extern "C" fn update_thread_locale(data: *mut thread_data_t) {
    if MSVCRT_locale.is_null() {
        return;
    }

    if ((*data).locale_flags & (LOCALE_FREE) as i32) != 0 &&
       (((*data).locale_flags & (LOCALE_THREAD) as i32) != 0 ||
        ((*data).locinfo == (*MSVCRT_locale).locinfo && (*data).mbcinfo == (*MSVCRT_locale).mbcinfo)) {
        return;
    }

    if ((*data).locale_flags & (LOCALE_FREE) as i32) != 0 {
        free_locinfo((*data).locinfo);
        free_mbcinfo((*data).mbcinfo);
    }

    _lock_locales();
    (*data).locinfo = (*MSVCRT_locale).locinfo;
    grab_locinfo((*data).locinfo);
    _unlock_locales();

    _lock(_MB_CP_LOCK);
    (*data).mbcinfo = (*MSVCRT_locale).mbcinfo;
    InterlockedIncrement(core::ptr::addr_of_mut!((*(*data).mbcinfo).refcount));
    _unlock(_MB_CP_LOCK);

    (*data).locale_flags |= (LOCALE_FREE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn get_locinfo() -> pthreadlocinfo {
    let data = msvcrt_get_thread_data();
    update_thread_locale(data);
    if (*data).locinfo.is_null() {
        ensure_cloc_static_init();
        return core::ptr::addr_of_mut!(cloc_static_locinfo);
    }
    (*data).locinfo
}
#[no_mangle]
pub unsafe extern "C" fn get_mbcinfo() -> pthreadmbcinfo {
    let data = msvcrt_get_thread_data();
    update_thread_locale(data);
    if (*data).mbcinfo.is_null() {
        ensure_cloc_static_init();
        return core::ptr::addr_of_mut!(cloc_static_mbcinfo);
    }
    (*data).mbcinfo
}
unsafe extern "C" fn construct_lc_all(locinfo: pthreadlocinfo) -> *mut i8 {
    static mut current_lc_all: [i8; MAX_LOCALE_LENGTH] = [0; MAX_LOCALE_LENGTH];

    let mut i: i32;

    i = (LC_MIN + 1) as i32;
    while i < (LC_MAX) as i32 {
        if strcmp(
            (*locinfo).lc_category[i as usize].locale,
            (*locinfo).lc_category[(i + 1) as usize].locale,
        ) != 0
        {
            break;
        }
        i += 1;
    }

    if i == (LC_MAX) as i32 {
        return (*locinfo).lc_category[LC_COLLATE as usize].locale;
    }

    sprintf(
        (&raw mut current_lc_all).cast::<i8>(),
        b"LC_COLLATE=%s;LC_CTYPE=%s;LC_MONETARY=%s;LC_NUMERIC=%s;LC_TIME=%s\0".as_ptr() as *const i8,
        (*locinfo).lc_category[LC_COLLATE as usize].locale,
        (*locinfo).lc_category[LC_CTYPE as usize].locale,
        (*locinfo).lc_category[LC_MONETARY as usize].locale,
        (*locinfo).lc_category[LC_NUMERIC as usize].locale,
        (*locinfo).lc_category[LC_TIME as usize].locale,
    );

    (&raw mut current_lc_all).cast::<i8>()
}
#[no_mangle]
pub unsafe extern "C" fn _Getdays() -> *mut i8 {
    let cur: *mut __lc_time_data = (*get_locinfo()).lc_time_curr;
    let mut i: i32;
    let mut len: i32;
    let mut size: i32 = 0;
    let mut out: *mut i8;

    i = 0;
    while i < 7 {
        size += (strlen((*cur).str_.names.short_wday[i as usize]) + 1) as i32;
        size += (strlen((*cur).str_.names.wday[i as usize]) + 1) as i32;
        i += 1;
    }
    out = malloc((size + 1) as usize) as *mut i8;
    if out.is_null() {
        return core::ptr::null_mut();
    }

    size = 0;
    i = 0;
    while i < 7 {
        *out.offset(size as isize) = b':' as i8;
        size += 1;
        len = (strlen((*cur).str_.names.short_wday[i as usize])) as i32;
        memcpy(out.offset(size as isize) as *mut core::ffi::c_void, (*cur).str_.names.short_wday[i as usize] as *const core::ffi::c_void, len as usize);
        size += len;

        *out.offset(size as isize) = b':' as i8;
        size += 1;
        len = (strlen((*cur).str_.names.wday[i as usize])) as i32;
        memcpy(out.offset(size as isize) as *mut core::ffi::c_void, (*cur).str_.names.wday[i as usize] as *const core::ffi::c_void, len as usize);
        size += len;
        i += 1;
    }
    *out.offset(size as isize) = b'\0' as i8;

    out
}
#[no_mangle]
pub unsafe extern "C" fn _Getmonths() -> *mut i8 {
    let cur: *mut __lc_time_data = (*get_locinfo()).lc_time_curr;
    let mut i: i32;
    let mut len: i32;
    let mut size: i32 = 0;
    let out: *mut i8;

    i = 0;
    while i < 12 {
        size += (strlen((*cur).str_.names.short_mon[i as usize]) + 1) as i32;
        size += (strlen((*cur).str_.names.mon[i as usize]) + 1) as i32;
        i += 1;
    }

    out = malloc((size + 1) as usize) as *mut i8;
    if out.is_null() {
        return core::ptr::null_mut();
    }

    size = 0;
    i = 0;
    while i < 12 {
        *out.offset(size as isize) = b':' as i8;
        size += 1;
        len = (strlen((*cur).str_.names.short_mon[i as usize])) as i32;
        memcpy(out.offset(size as isize) as *mut core::ffi::c_void, (*cur).str_.names.short_mon[i as usize] as *const core::ffi::c_void, len as usize);
        size += len;

        *out.offset(size as isize) = b':' as i8;
        size += 1;
        len = (strlen((*cur).str_.names.mon[i as usize])) as i32;
        memcpy(out.offset(size as isize) as *mut core::ffi::c_void, (*cur).str_.names.mon[i as usize] as *const core::ffi::c_void, len as usize);
        size += len;

        i += 1;
    }
    *out.offset(size as isize) = b'\0' as i8;

    out
}
#[no_mangle]
pub unsafe extern "C" fn _Gettnames() -> *mut core::ffi::c_void {
    let cur: *mut __lc_time_data = (*get_locinfo()).lc_time_curr;
    let mut size: core::ffi::c_uint = core::mem::size_of::<__lc_time_data>() as core::ffi::c_uint;

    let mut i: core::ffi::c_uint = 0;
    while (i as usize) < ARRAY_SIZE!((*cur).str_.str_) {
        size = size.wrapping_add((strlen((*cur).str_.str_[i as usize]).wrapping_add(1)) as u32);
        i = i.wrapping_add(1);
    }

    let ret: *mut __lc_time_data = malloc(size as usize) as *mut __lc_time_data;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    memcpy(ret as *mut core::ffi::c_void, cur as *const core::ffi::c_void, core::mem::size_of::<__lc_time_data>());

    size = 0;
    i = 0;
    while (i as usize) < ARRAY_SIZE!((*cur).str_.str_) {
        let len: usize = strlen((*cur).str_.str_[i as usize]).wrapping_add(1);
        memcpy((*ret).data.as_mut_ptr().offset(size as isize) as *mut core::ffi::c_void, (*cur).str_.str_[i as usize] as *const core::ffi::c_void, len);
        (*ret).str_.str_[i as usize] = (*ret).data.as_mut_ptr().offset(size as isize);
        size = size.wrapping_add(len as core::ffi::c_uint);
        i = i.wrapping_add(1);
    }

    ret as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "C" fn __crtLCMapStringA(
    lcid: LCID,
    mapflags: u32,
    src: *const i8,
    srclen: i32,
    dst: *mut i8,
    dstlen: i32,
    codepage: u32,
    xflag: i32,
) -> i32 {
    let mut buf_in: [u16; 32] = [0; 32];
    let mut in_ptr: *mut u16 = buf_in.as_mut_ptr();
    let mut buf_out: [u16; 32] = [0; 32];
    let mut out: *mut u16 = buf_out.as_mut_ptr();
    let in_len: i32;
    let out_len: i32;
    let mut r: i32;

    in_len = MultiByteToWideChar(codepage, MB_ERR_INVALID_CHARS, src, srclen, core::ptr::null_mut(), 0);
    if in_len == 0 {
        return 0;
    }
    if in_len as usize > ARRAY_SIZE!(buf_in) {
        in_ptr = malloc((in_len as usize) * core::mem::size_of::<u16>()) as *mut u16;
        if in_ptr.is_null() {
            return 0;
        }
    }

    r = MultiByteToWideChar(codepage, MB_ERR_INVALID_CHARS, src, srclen, in_ptr, in_len);
    if r == 0 {
        if in_ptr != buf_in.as_mut_ptr() {
            free(in_ptr as *mut core::ffi::c_void);
        }
        if out != buf_out.as_mut_ptr() {
            free(out as *mut core::ffi::c_void);
        }
        return r;
    }

    if (mapflags & LCMAP_SORTKEY) != 0 {
        r = LCMapStringW(lcid, mapflags, in_ptr, in_len, dst as *mut u16, dstlen);
        if in_ptr != buf_in.as_mut_ptr() {
            free(in_ptr as *mut core::ffi::c_void);
        }
        if out != buf_out.as_mut_ptr() {
            free(out as *mut core::ffi::c_void);
        }
        return r;
    }

    r = LCMapStringW(lcid, mapflags, in_ptr, in_len, core::ptr::null_mut(), 0);
    if r == 0 {
        if in_ptr != buf_in.as_mut_ptr() {
            free(in_ptr as *mut core::ffi::c_void);
        }
        if out != buf_out.as_mut_ptr() {
            free(out as *mut core::ffi::c_void);
        }
        return r;
    }
    out_len = r;
    if r as usize > ARRAY_SIZE!(buf_out) {
        out = malloc((r as usize) * core::mem::size_of::<u16>()) as *mut u16;
        if out.is_null() {
            r = 0;
            if in_ptr != buf_in.as_mut_ptr() {
                free(in_ptr as *mut core::ffi::c_void);
            }
            if out != buf_out.as_mut_ptr() {
                free(out as *mut core::ffi::c_void);
            }
            return r;
        }
    }

    r = LCMapStringW(lcid, mapflags, in_ptr, in_len, out, out_len);
    if r == 0 {
        if in_ptr != buf_in.as_mut_ptr() {
            free(in_ptr as *mut core::ffi::c_void);
        }
        if out != buf_out.as_mut_ptr() {
            free(out as *mut core::ffi::c_void);
        }
        return r;
    }

    r = WideCharToMultiByte(codepage, 0, out, out_len, dst, dstlen, core::ptr::null_mut(), core::ptr::null_mut());

    if in_ptr != buf_in.as_mut_ptr() {
        free(in_ptr as *mut core::ffi::c_void);
    }
    if out != buf_out.as_mut_ptr() {
        free(out as *mut core::ffi::c_void);
    }
    r
}
#[no_mangle]
pub unsafe extern "C" fn __crtLCMapStringW(
    lcid: LCID,
    mapflags: u32,
    src: *const wchar_t,
    srclen: i32,
    dst: *mut wchar_t,
    dstlen: i32,
    codepage: u32,
    xflag: i32,
) -> i32 {
    FIXME!("(lcid %lx, flags %lx, %S(%d), %p(%d), %x, %d), partial stub!\n", lcid, mapflags, src, srclen, dst, dstlen, codepage, xflag);

    return LCMapStringW(lcid, mapflags, src, srclen, dst, dstlen);
}

#[no_mangle]
pub unsafe extern "C" fn __crtCompareStringA(
    lcid: LCID,
    flags: DWORD,
    src1: *const i8,
    len1: i32,
    src2: *const i8,
    len2: i32,
) -> i32 {
    FIXME!("(lcid %lx, flags %lx, %s(%d), %s(%d), partial stub\n", lcid, flags, src1, len1, src2, len2);
    /* FIXME: probably not entirely right */
    CompareStringA(lcid, flags, src1, len1, src2, len2)
}

#[no_mangle]
pub unsafe extern "C" fn __crtCompareStringW(
    lcid: LCID,
    flags: DWORD,
    src1: *const wchar_t,
    len1: i32,
    src2: *const wchar_t,
    len2: i32,
) -> i32 {
    FIXME!("(lcid %lx, flags %lx, %S(%d), %S(%d), partial stub\n", lcid, flags, src1, len1, src2, len2);
    /* FIXME: probably not entirely right */
    CompareStringW(lcid, flags, src1, len1, src2, len2)
}

#[no_mangle]
pub unsafe extern "C" fn __crtGetLocaleInfoW(
    lcid: LCID,
    r#type: LCTYPE,
    buffer: *mut wchar_t,
    len: i32,
) -> i32 {
    FIXME!("(lcid %lx, type %lx, %p(%d), partial stub\n", lcid, r#type, buffer, len);
    /* FIXME: probably not entirely right */
    GetLocaleInfoW(lcid, r#type, buffer, len)
}

#[no_mangle]
pub unsafe extern "C" fn __crtGetStringTypeW(
    unk: DWORD,
    type_: DWORD,
    buffer: *mut wchar_t,
    len: i32,
    out: *mut WORD,
) -> BOOL {
    FIXME!("(unk %lx, type %lx, wstr %p(%d), %p) partial stub\n", unk, type_, buffer, len, out);

    GetStringTypeW(type_, buffer, len, out)
}

#[no_mangle]
pub unsafe extern "C" fn localeconv() -> *mut lconv {
    (*get_locinfo()).lconv
}
#[no_mangle]
pub unsafe extern "C" fn __lconv_init() -> i32 {
    cloc_lconv.int_frac_digits = UCHAR_MAX as i8;
    cloc_lconv.frac_digits = UCHAR_MAX as i8;
    cloc_lconv.p_cs_precedes = UCHAR_MAX as i8;
    cloc_lconv.p_sep_by_space = UCHAR_MAX as i8;
    cloc_lconv.n_cs_precedes = UCHAR_MAX as i8;
    cloc_lconv.n_sep_by_space = UCHAR_MAX as i8;
    cloc_lconv.p_sign_posn = UCHAR_MAX as i8;
    cloc_lconv.n_sign_posn = UCHAR_MAX as i8;
    0
}

#[no_mangle]
pub unsafe extern "C" fn ___lc_handle_func() -> *mut LCID {
    (*get_locinfo()).lc_handle.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn ___lc_codepage_func() -> u32 {
    (*get_locinfo()).lc_codepage
}
#[no_mangle]
pub unsafe extern "C" fn ___lc_collate_cp_func() -> i32 {
    ((*get_locinfo()).lc_collate_cp) as i32
}
#[no_mangle]
pub unsafe extern "C" fn free_locinfo(locinfo: pthreadlocinfo) {
    let mut i: core::ffi::c_int;

    if locinfo.is_null() {
        return;
    }

    i = (LC_MIN + 1) as i32;
    while i <= (LC_MAX) as i32 {
        if (*locinfo).lc_category[i as usize].refcount.is_null()
            || InterlockedDecrement((*locinfo).lc_category[i as usize].refcount as *mut i32) == 0
        {
            free((*locinfo).lc_category[i as usize].locale as *mut core::ffi::c_void);
            free((*locinfo).lc_category[i as usize].refcount as *mut core::ffi::c_void);
        }
        if (*locinfo).lc_category[i as usize].wrefcount.is_null()
            || InterlockedDecrement((*locinfo).lc_category[i as usize].wrefcount as *mut i32) == 0
        {
            free((*locinfo).lc_category[i as usize].wrefcount as *mut core::ffi::c_void);
        }
        i += 1;
    }

    if !(*locinfo).lconv_num_refcount.is_null()
        && InterlockedDecrement((*locinfo).lconv_num_refcount as *mut i32) == 0
    {
        free((*(*locinfo).lconv).decimal_point as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).thousands_sep as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).grouping as *mut core::ffi::c_void);
        free((*locinfo).lconv_num_refcount as *mut core::ffi::c_void);
    }
    if !(*locinfo).lconv_mon_refcount.is_null()
        && InterlockedDecrement((*locinfo).lconv_mon_refcount as *mut i32) == 0
    {
        free((*(*locinfo).lconv).int_curr_symbol as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).currency_symbol as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).mon_decimal_point as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).mon_thousands_sep as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).mon_grouping as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).positive_sign as *mut core::ffi::c_void);
        free((*(*locinfo).lconv).negative_sign as *mut core::ffi::c_void);
        free((*locinfo).lconv_mon_refcount as *mut core::ffi::c_void);
    }
    if !(*locinfo).lconv_intl_refcount.is_null()
        && InterlockedDecrement((*locinfo).lconv_intl_refcount as *mut i32) == 0
    {
        free((*locinfo).lconv_intl_refcount as *mut core::ffi::c_void);
        free((*locinfo).lconv as *mut core::ffi::c_void);
    }

    if !(*locinfo).ctype1_refcount.is_null()
        && InterlockedDecrement((*locinfo).ctype1_refcount as *mut i32) == 0
    {
        free((*locinfo).ctype1_refcount as *mut core::ffi::c_void);
        free((*locinfo).ctype1 as *mut core::ffi::c_void);
        free((*locinfo).pclmap as *mut core::ffi::c_void);
        free((*locinfo).pcumap as *mut core::ffi::c_void);
    }

    if !(*locinfo).lc_time_curr.is_null()
        && InterlockedDecrement(core::ptr::addr_of_mut!((*(*locinfo).lc_time_curr).refcount) as *mut i32) == 0
        && (*locinfo).lc_time_curr != core::ptr::addr_of_mut!(cloc_time_data)
    {
        free((*locinfo).lc_time_curr as *mut core::ffi::c_void);
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*locinfo).refcount) as *mut i32) != 0 {
        return;
    }

    free(locinfo as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn free_mbcinfo(mbcinfo: pthreadmbcinfo) {
    if mbcinfo.is_null() {
        return;
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*mbcinfo).refcount)) != 0 {
        return;
    }

    free(mbcinfo as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn get_current_locale_noalloc(locale: _locale_t) -> _locale_t {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    update_thread_locale(data);
    (*locale).locinfo = (*data).locinfo;
    (*locale).mbcinfo = (*data).mbcinfo;

    if (*locale).locinfo.is_null() {
        ensure_cloc_static_init();
        (*locale).locinfo = core::ptr::addr_of_mut!(cloc_static_locinfo);
    }
    if (*locale).mbcinfo.is_null() {
        ensure_cloc_static_init();
        (*locale).mbcinfo = core::ptr::addr_of_mut!(cloc_static_mbcinfo);
    }

    grab_locinfo((*locale).locinfo);
    InterlockedIncrement(core::ptr::addr_of_mut!((*(*locale).mbcinfo).refcount));
    locale
}
#[no_mangle]
pub unsafe extern "C" fn free_locale_noalloc(locale: _locale_t) {
    free_locinfo((*locale).locinfo);
    free_mbcinfo((*locale).mbcinfo);
}
#[no_mangle]
pub unsafe extern "C" fn _get_current_locale() -> _locale_t {
    let loc = malloc(core::mem::size_of::<_locale_tstruct>()) as _locale_t;
    if loc.is_null() {
        return core::ptr::null_mut();
    }

    get_current_locale_noalloc(loc)
}
#[no_mangle]
pub unsafe extern "C" fn _free_locale(locale: _locale_t) {
    if locale.is_null() {
        return;
    }

    free_locale_noalloc(locale);
    free((locale).cast());
}
unsafe extern "C" fn category_needs_update(
    cat: i32,
    locinfo: *const threadlocinfo,
    sname: *mut wchar_t,
    cp: u16,
) -> u8 {
    let lcid: LCID;
    if locinfo.is_null() {
        return (TRUE) as u8;
    }
    lcid = if !sname.is_null() {
        LocaleNameToLCID(sname, 0)
    } else {
        0
    };
    if lcid != (*locinfo).lc_handle[cat as usize]
        || cp != (*locinfo).lc_id[cat as usize].wCodePage
    {
        1u8
    } else {
        0u8
    }
}
unsafe extern "C" fn create_time_data(sname: *mut u16) -> *mut __lc_time_data {
    static time_data: [u32; 43] = [
        LOCALE_SABBREVDAYNAME7, LOCALE_SABBREVDAYNAME1, LOCALE_SABBREVDAYNAME2,
        LOCALE_SABBREVDAYNAME3, LOCALE_SABBREVDAYNAME4, LOCALE_SABBREVDAYNAME5,
        LOCALE_SABBREVDAYNAME6,
        LOCALE_SDAYNAME7, LOCALE_SDAYNAME1, LOCALE_SDAYNAME2, LOCALE_SDAYNAME3,
        LOCALE_SDAYNAME4, LOCALE_SDAYNAME5, LOCALE_SDAYNAME6,
        LOCALE_SABBREVMONTHNAME1, LOCALE_SABBREVMONTHNAME2, LOCALE_SABBREVMONTHNAME3,
        LOCALE_SABBREVMONTHNAME4, LOCALE_SABBREVMONTHNAME5, LOCALE_SABBREVMONTHNAME6,
        LOCALE_SABBREVMONTHNAME7, LOCALE_SABBREVMONTHNAME8, LOCALE_SABBREVMONTHNAME9,
        LOCALE_SABBREVMONTHNAME10, LOCALE_SABBREVMONTHNAME11, LOCALE_SABBREVMONTHNAME12,
        LOCALE_SMONTHNAME1, LOCALE_SMONTHNAME2, LOCALE_SMONTHNAME3, LOCALE_SMONTHNAME4,
        LOCALE_SMONTHNAME5, LOCALE_SMONTHNAME6, LOCALE_SMONTHNAME7, LOCALE_SMONTHNAME8,
        LOCALE_SMONTHNAME9, LOCALE_SMONTHNAME10, LOCALE_SMONTHNAME11, LOCALE_SMONTHNAME12,
        LOCALE_S1159, LOCALE_S2359,
        LOCALE_SSHORTDATE, LOCALE_SLONGDATE,
        LOCALE_STIMEFORMAT
    ];

    let mut cur: *mut __lc_time_data;
    let mut i: i32;
    let mut ret: i32;
    let mut size: i32;
    let lcid: u32 = LocaleNameToLCID(sname, LCID_CONVERSION_FLAGS);

    size = 0;
    i = 0;
    while (i as usize) < ARRAY_SIZE!(time_data) {
        ret = GetLocaleInfoA(lcid, time_data[i as usize], core::ptr::null_mut(), 0);
        if ret == 0 {
            return core::ptr::null_mut();
        }
        size += ret;

        ret = GetLocaleInfoEx(sname, time_data[i as usize], core::ptr::null_mut(), 0);
        if ret == 0 {
            return core::ptr::null_mut();
        }
        size += ret * core::mem::size_of::<wchar_t>() as i32;
        i += 1;
    }

    // C: `malloc(FIELD_OFFSET(__lc_time_data, data[size]))` — the flexible-array allocation
    // idiom, sizing the struct to hold `size` trailing bytes. `offset_of!` takes a FIELD, not
    // an indexed place, so the index has to become the arithmetic the C macro was doing:
    // offset of `data` plus `size` elements. `data` is `char[1]` (bindgen's flex-array
    // rendering), so the element size is 1 — spelled out rather than assumed, because if the
    // header ever widens it this line must move with it.
    cur = malloc(
        core::mem::offset_of!(__lc_time_data, data)
            + size as usize * core::mem::size_of::<core::ffi::c_char>(),
    ) as *mut __lc_time_data;
    if cur.is_null() {
        return core::ptr::null_mut();
    }

    ret = 0;
    i = 0;
    while (i as usize) < ARRAY_SIZE!(time_data) {
        (*cur).str_.str_[i as usize] = (*cur).data.as_mut_ptr().add(ret as usize);
        ret += GetLocaleInfoA(lcid, time_data[i as usize], (*cur).data.as_mut_ptr().add(ret as usize), size - ret);
        i += 1;
    }
    i = 0;
    while (i as usize) < ARRAY_SIZE!(time_data) {
        (*cur).wstr.wstr[i as usize] = (*cur).data.as_mut_ptr().add(ret as usize) as *mut wchar_t;
        ret += GetLocaleInfoEx(sname, time_data[i as usize], (*cur).data.as_mut_ptr().add(ret as usize) as *mut wchar_t,
                (size - ret) / core::mem::size_of::<wchar_t>() as i32) * core::mem::size_of::<wchar_t>() as i32;
        i += 1;
    }
    (*cur).lcid = lcid;
    (*cur).unk = 1;
    (*cur).refcount = 1;

    cur
}
unsafe extern "C" fn create_locinfo(
    category: i32,
    locale: *const i8,
    old_locinfo: *const threadlocinfo,
) -> pthreadlocinfo {
    static collate: &[u8] = b"COLLATE=";
    static ctype: &[u8] = b"CTYPE=";
    static monetary: &[u8] = b"MONETARY=";
    static numeric: &[u8] = b"NUMERIC=";
    static time: &[u8] = b"TIME=";

    let mut locinfo: pthreadlocinfo = core::ptr::null_mut();
    let mut cp: [u16; 6] = [0; 6];
    let mut locale_name: [*const i8; 6] = [core::ptr::null(); 6];
    let mut locale_sname: [*mut WCHAR; 6] = [core::ptr::null_mut(); 6];
    // Zeroed for the same reason as `sname_match` above: GetLocaleInfoEx writes it,
    // except on the failure path the C still reads.
    let mut val: i32 = 0;
    let mut locale_len: [i32; 6] = [0; 6];
    let mut buf: [i8; 256] = [0; 256];
    // Zeroed: `locale_to_sname` fills it, but the C leaves it uninitialized on the
    // failure path where it is still read. Rust forbids `addr_of_mut!` on an
    // uninitialized binding at all.
    let mut sname_match: u8 = 0;
    let mut wbuf: [wchar_t; 256] = [0; 256];
    let mut map_buf: [wchar_t; 256] = [0; 256];
    let mut i: i32;

    // C `goto fail` -> a labeled block, which is Rust's only forward jump. The
    // translation emitted the `break 'fail` sites but put the epilogue in a `'fail: loop`
    // AFTER the success `return`, where it is both unreachable and out of scope: a Rust
    // label is only visible inside the block it annotates, so all 12 jumps failed to
    // resolve. Wrapping the body is the shape that matches the C — the success path
    // `return`s straight out and skips the epilogue, exactly as falling past `fail:` does.
    // `locinfo`, `i` and `locale_sname` are declared above it because the epilogue reads them.
    'fail: {
    if category < (LC_MIN) as i32 || category > (LC_MAX) as i32 || locale.is_null() {
        return core::ptr::null_mut();
    }

    if *locale.add(0) == b'C' as i8 && *locale.add(1) == 0 {
        locale_sname[0] = core::ptr::null_mut();
        cp[0] = (CP_ACP) as u16;
    } else if *locale.add(0) == b'L' as i8
        && *locale.add(1) == b'C' as i8
        && *locale.add(2) == b'_' as i8
    {
        let mut p: *const i8;
        let mut locale_mut = locale;

        loop {
            locale_mut = locale_mut.add(3);
            if memcmp(
                locale_mut as *const core::ffi::c_void,
                collate.as_ptr() as *const core::ffi::c_void,
                collate.len() - 1,
            ) == 0
            {
                i = (LC_COLLATE) as i32;
                locale_mut = locale_mut.add(collate.len() - 1);
            } else if memcmp(
                locale_mut as *const core::ffi::c_void,
                ctype.as_ptr() as *const core::ffi::c_void,
                ctype.len() - 1,
            ) == 0
            {
                i = (LC_CTYPE) as i32;
                locale_mut = locale_mut.add(ctype.len() - 1);
            } else if memcmp(
                locale_mut as *const core::ffi::c_void,
                monetary.as_ptr() as *const core::ffi::c_void,
                monetary.len() - 1,
            ) == 0
            {
                i = (LC_MONETARY) as i32;
                locale_mut = locale_mut.add(monetary.len() - 1);
            } else if memcmp(
                locale_mut as *const core::ffi::c_void,
                numeric.as_ptr() as *const core::ffi::c_void,
                numeric.len() - 1,
            ) == 0
            {
                i = (LC_NUMERIC) as i32;
                locale_mut = locale_mut.add(numeric.len() - 1);
            } else if memcmp(
                locale_mut as *const core::ffi::c_void,
                time.as_ptr() as *const core::ffi::c_void,
                time.len() - 1,
            ) == 0
            {
                i = (LC_TIME) as i32;
                locale_mut = locale_mut.add(time.len() - 1);
            } else {
                break 'fail;
            }

            p = strchr(locale_mut, b';' as i32);
            if *locale_mut.add(0) == b'C' as i8
                && (*locale_mut.add(1) == b';' as i8
                    || *locale_mut.add(1) == 0)
            {
                locale_sname[i as usize] = core::ptr::null_mut();
                cp[i as usize] = (CP_ACP) as u16;
            } else {
                let mut locale_found;

                if !p.is_null() {
                    memcpy(
                        buf.as_mut_ptr() as *mut core::ffi::c_void,
                        locale_mut as *const core::ffi::c_void,
                        p.offset_from(locale_mut) as usize,
                    );
                    buf[p.offset_from(locale_mut) as usize] = 0;
                    locale_found = (locale_to_sname(
                        buf.as_ptr(),
                        core::ptr::addr_of_mut!(cp[i as usize]),
                        (core::ptr::addr_of_mut!(sname_match)).cast(),
                        wbuf.as_mut_ptr(),
                    )) as u8;
                } else {
                    locale_found = (locale_to_sname(
                        locale_mut,
                        core::ptr::addr_of_mut!(cp[i as usize]),
                        (core::ptr::addr_of_mut!(sname_match)).cast(),
                        wbuf.as_mut_ptr(),
                    )) as u8;
                }

                if locale_found == 0 {
                    break 'fail;
                }
                locale_sname[i as usize] = wcsdup(wbuf.as_ptr());
                if locale_sname[i as usize].is_null() {
                    break 'fail;
                }
                if sname_match != 0 {
                    locale_name[i as usize] = locale_mut;
                    locale_len[i as usize] = if !p.is_null() {
                        p.offset_from(locale_mut) as i32
                    } else {
                        strlen(locale_mut) as i32
                    };
                }
            }

            if p.is_null()
                || *p.add(1) != b'L' as i8
                || *p.add(2) != b'C' as i8
                || *p.add(3) != b'_' as i8
            {
                break;
            }

            locale_mut = p.add(1);
        }
    } else {
        let locale_found = locale_to_sname(
            locale,
            core::ptr::addr_of_mut!(cp[0]),
            (core::ptr::addr_of_mut!(sname_match)).cast(),
            wbuf.as_mut_ptr(),
        );

        if locale_found == 0 {
            return core::ptr::null_mut();
        }

        locale_sname[0] = wcsdup(wbuf.as_ptr());
        if locale_sname[0].is_null() {
            return core::ptr::null_mut();
        }

        if sname_match != 0 {
            locale_name[0] = locale;
            locale_len[0] = strlen(locale) as i32;
        }

        i = 1;
        while i < 6 {
            locale_sname[i as usize] = wcsdup(locale_sname[0]);
            if locale_sname[i as usize].is_null() {
                break 'fail;
            }

            cp[i as usize] = cp[0];
            locale_name[i as usize] = locale_name[0];
            locale_len[i as usize] = locale_len[0];
            i += 1;
        }
    }

    i = 1;
    'outer: loop {
        if i >= 6 {
            break;
        }

        if i == (LC_CTYPE) as i32 && cp[i as usize] == (CP_UTF8) as u16 {
            let sname_size: i32;
            if !old_locinfo.is_null() && (*old_locinfo).lc_handle[i as usize] != 0 {
                sname_size = LCIDToLocaleName(
                    (*old_locinfo).lc_handle[i as usize],
                    core::ptr::null_mut(),
                    0,
                    0,
                );
                locale_sname[i as usize] = malloc(
                    (sname_size as usize) * core::mem::size_of::<WCHAR>(),
                ) as *mut WCHAR;
                if locale_sname[i as usize].is_null() {
                    break 'fail;
                }
                LCIDToLocaleName(
                    (*old_locinfo).lc_handle[i as usize],
                    locale_sname[i as usize],
                    sname_size,
                    0,
                );
            } else {
                locale_sname[i as usize] = core::ptr::null_mut();
            }

            locale_name[i as usize] = core::ptr::null();
            locale_len[i as usize] = 0;
            cp[i as usize] = if !old_locinfo.is_null() {
                (*old_locinfo).lc_id[i as usize].wCodePage
            } else {
                0
            };
        }
        if category != (LC_ALL) as i32 && category != i {
            if !old_locinfo.is_null() {
                let sname_size: i32;
                if (*old_locinfo).lc_handle[i as usize] != 0 {
                    sname_size = LCIDToLocaleName(
                        (*old_locinfo).lc_handle[i as usize],
                        core::ptr::null_mut(),
                        0,
                        0,
                    );
                    locale_sname[i as usize] = malloc(
                        (sname_size as usize) * core::mem::size_of::<WCHAR>(),
                    ) as *mut WCHAR;
                    if locale_sname[i as usize].is_null() {
                        break 'fail;
                    }
                    LCIDToLocaleName(
                        (*old_locinfo).lc_handle[i as usize],
                        locale_sname[i as usize],
                        sname_size,
                        0,
                    );
                } else {
                    locale_sname[i as usize] = core::ptr::null_mut();
                }
                cp[i as usize] = (*old_locinfo).lc_id[i as usize].wCodePage;
            } else {
                locale_sname[i as usize] = core::ptr::null_mut();
                cp[i as usize] = 0;
            }
        }

        i += 1;
    }

    locinfo = malloc(core::mem::size_of::<threadlocinfo>()) as pthreadlocinfo;
    if locinfo.is_null() {
        break 'fail;
    }

    memset(
        locinfo as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<threadlocinfo>(),
    );
    (*locinfo).refcount = 1;

    if !locale_name[LC_COLLATE].is_null()
        && init_category_name(
            locale_name[LC_COLLATE],
            locale_len[LC_COLLATE],
            locinfo,
            (LC_COLLATE) as i32,
        ) == 0
    {
        break 'fail;
    }

    if category_needs_update(
        (LC_COLLATE) as i32,
        old_locinfo,
        locale_sname[LC_COLLATE],
        cp[LC_COLLATE],
    ) == 0
    {
        copy_threadlocinfo_category(locinfo, old_locinfo, (LC_COLLATE) as i32);
        (*locinfo).lc_collate_cp = (*old_locinfo).lc_collate_cp;
    } else if !locale_sname[LC_COLLATE].is_null() {
        if update_threadlocinfo_category(
            locale_sname[LC_COLLATE],
            cp[LC_COLLATE],
            locinfo,
            (LC_COLLATE) as i32,
        ) == 0
        {
            break 'fail;
        }

        (*locinfo).lc_collate_cp = ((*locinfo).lc_id[LC_COLLATE].wCodePage) as u32;
    } else {
        if init_category_name(
            b"C\0".as_ptr() as *const i8,
            1,
            locinfo,
            (LC_COLLATE) as i32,
        ) == 0
        {
            break 'fail;
        }
    }

    if !locale_name[LC_CTYPE].is_null()
        && init_category_name(
            locale_name[LC_CTYPE],
            locale_len[LC_CTYPE],
            locinfo,
            (LC_CTYPE) as i32,
        ) == 0
    {
        break 'fail;
    }

    if category_needs_update(
        (LC_CTYPE) as i32,
        old_locinfo,
        locale_sname[LC_CTYPE],
        cp[LC_CTYPE],
    ) == 0
    {
        copy_threadlocinfo_category(locinfo, old_locinfo, (LC_CTYPE) as i32);
        (*locinfo).lc_codepage = (*old_locinfo).lc_codepage;
        (*locinfo).lc_clike = (*old_locinfo).lc_clike;
        (*locinfo).mb_cur_max = (*old_locinfo).mb_cur_max;
        (*locinfo).ctype1 = (*old_locinfo).ctype1;
        (*locinfo).ctype1_refcount = (*old_locinfo).ctype1_refcount;
        (*locinfo).pctype = (*old_locinfo).pctype;
        (*locinfo).pclmap = (*old_locinfo).pclmap;
        (*locinfo).pcumap = (*old_locinfo).pcumap;
        if !(*locinfo).ctype1_refcount.is_null() {
            InterlockedIncrement((*locinfo).ctype1_refcount as *mut i32);
        }
    } else if !locale_sname[LC_CTYPE].is_null() {
        let mut cp_info: CPINFO = core::mem::zeroed();
        let mut j: i32;

        if update_threadlocinfo_category(
            locale_sname[LC_CTYPE],
            cp[LC_CTYPE],
            locinfo,
            (LC_CTYPE) as i32,
        ) == 0
        {
            break 'fail;
        }

        (*locinfo).lc_codepage = ((*locinfo).lc_id[LC_CTYPE].wCodePage) as u32;
        (*locinfo).lc_clike = 1;
        if GetCPInfo((*locinfo).lc_codepage as u32, core::ptr::addr_of_mut!(cp_info)) == 0 {
            break 'fail;
        }
        (*locinfo).mb_cur_max = cp_info.MaxCharSize as i32;

        (*locinfo).ctype1_refcount = malloc(core::mem::size_of::<i32>()) as *mut i32;
        if (*locinfo).ctype1_refcount.is_null() {
            break 'fail;
        }
        *(*locinfo).ctype1_refcount = 1;

        (*locinfo).ctype1 =
            malloc(257 * core::mem::size_of::<u16>()) as *mut u16;
        (*locinfo).pclmap =
            malloc(256 * core::mem::size_of::<u8>()) as *mut u8;
        (*locinfo).pcumap =
            malloc(256 * core::mem::size_of::<u8>()) as *mut u8;
        if (*locinfo).ctype1.is_null() || (*locinfo).pclmap.is_null() || (*locinfo).pcumap.is_null() {
            break 'fail;
        }

        *(*locinfo).ctype1.add(0) = 0;
        (*locinfo).pctype = (*locinfo).ctype1.add(1);

        buf[1] = 0;
        buf[2] = 0;
        i = 1;
        while i < 257 {
            buf[0] = (i - 1) as i8;

            MultiByteToWideChar(
                (*locinfo).lc_codepage as u32,
                0,
                buf.as_ptr(),
                1,
                wbuf.as_mut_ptr(),
                1,
            );
            *(*locinfo).ctype1.add(i as usize) = 0;
            GetStringTypeW(
                CT_CTYPE1,
                wbuf.as_ptr(),
                1,
                (*locinfo).ctype1.add(i as usize),
            );
            i += 1;
        }

        i = 0;
        while cp_info.LeadByte[(i + 1) as usize] != 0 {
            j = cp_info.LeadByte[i as usize] as i32;
            while j <= cp_info.LeadByte[(i + 1) as usize] as i32 {
                *(*locinfo).ctype1.add((j + 1) as usize) |= 0x8000u16;
                j += 1;
            }
            i += 2;
        }

        i = 0;
        while i < 256 {
            if (*(*locinfo).pctype.add(i as usize)) & 0x8000u16 != 0 {
                buf[i as usize] = b' ' as i8;
            } else {
                buf[i as usize] = i as i8;
            }
            i += 1;
        }

        MultiByteToWideChar(
            (*locinfo).lc_codepage as u32,
            0,
            buf.as_ptr(),
            256,
            wbuf.as_mut_ptr(),
            256,
        );
        LCMapStringW(
            LOCALE_INVARIANT,
            LCMAP_LOWERCASE,
            wbuf.as_ptr(),
            256,
            map_buf.as_mut_ptr(),
            256,
        );
        WideCharToMultiByte(
            (*locinfo).lc_codepage as u32,
            0,
            map_buf.as_ptr(),
            256,
            (*locinfo).pclmap as *mut i8,
            256,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        LCMapStringW(
            LOCALE_INVARIANT,
            LCMAP_UPPERCASE,
            wbuf.as_ptr(),
            256,
            map_buf.as_mut_ptr(),
            256,
        );
        WideCharToMultiByte(
            (*locinfo).lc_codepage as u32,
            0,
            map_buf.as_ptr(),
            256,
            (*locinfo).pcumap as *mut i8,
            256,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    } else {
        (*locinfo).lc_clike = 1;
        (*locinfo).mb_cur_max = 1;
        (*locinfo).pctype = (&raw mut MSVCRT__ctype).cast::<WORD>().add(1);
        (*locinfo).pclmap = cloc_clmap.as_ptr();
        (*locinfo).pcumap = cloc_cumap.as_ptr();
        if init_category_name(
            b"C\0".as_ptr() as *const i8,
            1,
            locinfo,
            (LC_CTYPE) as i32,
        ) == 0
        {
            break 'fail;
        }
    }

    if category_needs_update(
        (LC_MONETARY) as i32,
        old_locinfo,
        locale_sname[LC_MONETARY],
        cp[LC_MONETARY],
    ) == 0
        && category_needs_update(
            (LC_NUMERIC) as i32,
            old_locinfo,
            locale_sname[LC_NUMERIC],
            cp[LC_NUMERIC],
        ) == 0
    {
        (*locinfo).lconv = (*old_locinfo).lconv;
        (*locinfo).lconv_intl_refcount = (*old_locinfo).lconv_intl_refcount;
        if !(*locinfo).lconv_intl_refcount.is_null() {
            InterlockedIncrement((*locinfo).lconv_intl_refcount as *mut i32);
        }
    } else if !locale_sname[LC_MONETARY].is_null() || !locale_sname[LC_NUMERIC].is_null() {
        (*locinfo).lconv = malloc(core::mem::size_of::<lconv>()) as *mut lconv;
        (*locinfo).lconv_intl_refcount = malloc(core::mem::size_of::<i32>()) as *mut i32;
        if (*locinfo).lconv.is_null() || (*locinfo).lconv_intl_refcount.is_null() {
            free((*locinfo).lconv as *mut core::ffi::c_void);
            free((*locinfo).lconv_intl_refcount as *mut core::ffi::c_void);
            (*locinfo).lconv = core::ptr::null_mut();
            (*locinfo).lconv_intl_refcount = core::ptr::null_mut();
            break 'fail;
        }
        memset(
            (*locinfo).lconv as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<lconv>(),
        );
        *(*locinfo).lconv_intl_refcount = 1;
    } else {
        (*locinfo).lconv = core::ptr::addr_of_mut!(cloc_lconv);
    }

    if !locale_name[LC_MONETARY].is_null()
        && init_category_name(
            locale_name[LC_MONETARY],
            locale_len[LC_MONETARY],
            locinfo,
            (LC_MONETARY) as i32,
        ) == 0
    {
        break 'fail;
    }

    if category_needs_update(
        (LC_MONETARY) as i32,
        old_locinfo,
        locale_sname[LC_MONETARY],
        cp[LC_MONETARY],
    ) == 0
    {
        copy_threadlocinfo_category(locinfo, old_locinfo, (LC_MONETARY) as i32);
        (*locinfo).lconv_mon_refcount = (*old_locinfo).lconv_mon_refcount;
        if !(*locinfo).lconv_mon_refcount.is_null() {
            InterlockedIncrement((*locinfo).lconv_mon_refcount as *mut i32);
        }
        if (*locinfo).lconv != core::ptr::addr_of_mut!(cloc_lconv) && (*locinfo).lconv != (*old_locinfo).lconv {
            (*(*locinfo).lconv).int_curr_symbol = (*(*old_locinfo).lconv).int_curr_symbol;
            (*(*locinfo).lconv).currency_symbol = (*(*old_locinfo).lconv).currency_symbol;
            (*(*locinfo).lconv).mon_decimal_point = (*(*old_locinfo).lconv).mon_decimal_point;
            (*(*locinfo).lconv).mon_thousands_sep = (*(*old_locinfo).lconv).mon_thousands_sep;
            (*(*locinfo).lconv).mon_grouping = (*(*old_locinfo).lconv).mon_grouping;
            (*(*locinfo).lconv).positive_sign = (*(*old_locinfo).lconv).positive_sign;
            (*(*locinfo).lconv).negative_sign = (*(*old_locinfo).lconv).negative_sign;
            (*(*locinfo).lconv).int_frac_digits = (*(*old_locinfo).lconv).int_frac_digits;
            (*(*locinfo).lconv).frac_digits = (*(*old_locinfo).lconv).frac_digits;
            (*(*locinfo).lconv).p_cs_precedes = (*(*old_locinfo).lconv).p_cs_precedes;
            (*(*locinfo).lconv).p_sep_by_space = (*(*old_locinfo).lconv).p_sep_by_space;
            (*(*locinfo).lconv).n_cs_precedes = (*(*old_locinfo).lconv).n_cs_precedes;
            (*(*locinfo).lconv).n_sep_by_space = (*(*old_locinfo).lconv).n_sep_by_space;
            (*(*locinfo).lconv).p_sign_posn = (*(*old_locinfo).lconv).p_sign_posn;
            (*(*locinfo).lconv).n_sign_posn = (*(*old_locinfo).lconv).n_sign_posn;
        }
    } else if !locale_sname[LC_MONETARY].is_null() {
        if update_threadlocinfo_category(
            locale_sname[LC_MONETARY],
            cp[LC_MONETARY],
            locinfo,
            (LC_MONETARY) as i32,
        ) == 0
        {
            break 'fail;
        }

        (*locinfo).lconv_mon_refcount = malloc(core::mem::size_of::<i32>()) as *mut i32;
        if (*locinfo).lconv_mon_refcount.is_null() {
            break 'fail;
        }

        *(*locinfo).lconv_mon_refcount = 1;

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SINTLSYMBOL | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).int_curr_symbol = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).int_curr_symbol.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).int_curr_symbol,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SCURRENCY | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).currency_symbol = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).currency_symbol.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).currency_symbol,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SMONDECIMALSEP | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).mon_decimal_point = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).mon_decimal_point.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).mon_decimal_point,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SMONTHOUSANDSEP | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).mon_thousands_sep = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).mon_thousands_sep.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).mon_thousands_sep,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        i = GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SMONGROUPING | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            wbuf.as_ptr(),
            -1,
            buf.as_mut_ptr(),
            256,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i > 1 {
            i = i / 2
                + (if buf[(i - 2) as usize] == b'0' as i8 {
                    0
                } else {
                    1
                });
        }
        if i != 0 {
            (*(*locinfo).lconv).mon_grouping = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).mon_grouping.is_null() {
                let mut idx = 0;
                while buf[(idx + 1) as usize] == b';' as i8 {
                    *(*(*locinfo).lconv).mon_grouping.add((idx / 2) as usize) =
                        buf[idx as usize] - b'0' as i8;
                    idx += 2;
                }
                *(*(*locinfo).lconv).mon_grouping.add((idx / 2) as usize) =
                    buf[idx as usize] - b'0' as i8;
                if buf[idx as usize] != b'0' as i8 {
                    *(*(*locinfo).lconv).mon_grouping.add((idx / 2 + 1) as usize) =
                        127;
                }
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SPOSITIVESIGN | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).positive_sign = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).positive_sign.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).positive_sign,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_SNEGATIVESIGN | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_MONETARY] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).negative_sign = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).negative_sign.is_null() {
                WideCharToMultiByte(
                    cp[LC_MONETARY] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).negative_sign,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_IINTLCURRDIGITS | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).int_frac_digits = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_ICURRDIGITS | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).frac_digits = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_IPOSSYMPRECEDES | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).p_cs_precedes = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_IPOSSEPBYSPACE | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).p_sep_by_space = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_INEGSYMPRECEDES | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).n_cs_precedes = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_INEGSEPBYSPACE | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).n_sep_by_space = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_IPOSSIGNPOSN | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).p_sign_posn = val as i8;
        } else {
            break 'fail;
        }

        if GetLocaleInfoEx(
            locale_sname[LC_MONETARY],
            LOCALE_INEGSIGNPOSN | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!(val) as *mut WCHAR,
            2,
        ) != 0
        {
            (*(*locinfo).lconv).n_sign_posn = val as i8;
        } else {
            break 'fail;
        }
    } else {
        if (*locinfo).lconv != core::ptr::addr_of_mut!(cloc_lconv) {
            (*(*locinfo).lconv).int_curr_symbol = cloc_lconv.int_curr_symbol;
            (*(*locinfo).lconv).currency_symbol = cloc_lconv.currency_symbol;
            (*(*locinfo).lconv).mon_decimal_point = cloc_lconv.mon_decimal_point;
            (*(*locinfo).lconv).mon_thousands_sep = cloc_lconv.mon_thousands_sep;
            (*(*locinfo).lconv).mon_grouping = cloc_lconv.mon_grouping;
            (*(*locinfo).lconv).positive_sign = cloc_lconv.positive_sign;
            (*(*locinfo).lconv).negative_sign = cloc_lconv.negative_sign;
            (*(*locinfo).lconv).int_frac_digits = cloc_lconv.int_frac_digits;
            (*(*locinfo).lconv).frac_digits = cloc_lconv.frac_digits;
            (*(*locinfo).lconv).p_cs_precedes = cloc_lconv.p_cs_precedes;
            (*(*locinfo).lconv).p_sep_by_space = cloc_lconv.p_sep_by_space;
            (*(*locinfo).lconv).n_cs_precedes = cloc_lconv.n_cs_precedes;
            (*(*locinfo).lconv).n_sep_by_space = cloc_lconv.n_sep_by_space;
            (*(*locinfo).lconv).p_sign_posn = cloc_lconv.p_sign_posn;
            (*(*locinfo).lconv).n_sign_posn = cloc_lconv.n_sign_posn;
        }

        if init_category_name(
            b"C\0".as_ptr() as *const i8,
            1,
            locinfo,
            (LC_MONETARY) as i32,
        ) == 0
        {
            break 'fail;
        }
    }

    if !locale_name[LC_NUMERIC].is_null()
        && init_category_name(
            locale_name[LC_NUMERIC],
            locale_len[LC_NUMERIC],
            locinfo,
            (LC_NUMERIC) as i32,
        ) == 0
    {
        break 'fail;
    }

    if category_needs_update(
        (LC_NUMERIC) as i32,
        old_locinfo,
        locale_sname[LC_NUMERIC],
        cp[LC_NUMERIC],
    ) == 0
    {
        copy_threadlocinfo_category(locinfo, old_locinfo, (LC_NUMERIC) as i32);
        (*locinfo).lconv_num_refcount = (*old_locinfo).lconv_num_refcount;
        if !(*locinfo).lconv_num_refcount.is_null() {
            InterlockedIncrement((*locinfo).lconv_num_refcount as *mut i32);
        }
        if (*locinfo).lconv != core::ptr::addr_of_mut!(cloc_lconv) && (*locinfo).lconv != (*old_locinfo).lconv {
            (*(*locinfo).lconv).decimal_point = (*(*old_locinfo).lconv).decimal_point;
            (*(*locinfo).lconv).thousands_sep = (*(*old_locinfo).lconv).thousands_sep;
            (*(*locinfo).lconv).grouping = (*(*old_locinfo).lconv).grouping;
        }
    } else if !locale_sname[LC_NUMERIC].is_null() {
        if update_threadlocinfo_category(
            locale_sname[LC_NUMERIC],
            cp[LC_NUMERIC],
            locinfo,
            (LC_NUMERIC) as i32,
        ) == 0
        {
            break 'fail;
        }

        (*locinfo).lconv_num_refcount = malloc(core::mem::size_of::<i32>()) as *mut i32;
        if (*locinfo).lconv_num_refcount.is_null() {
            break 'fail;
        }

        *(*locinfo).lconv_num_refcount = 1;

        GetLocaleInfoEx(
            locale_sname[LC_NUMERIC],
            LOCALE_SDECIMAL | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_NUMERIC] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).decimal_point = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).decimal_point.is_null() {
                WideCharToMultiByte(
                    cp[LC_NUMERIC] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).decimal_point,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        GetLocaleInfoEx(
            locale_sname[LC_NUMERIC],
            LOCALE_STHOUSAND | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        i = WideCharToMultiByte(
            cp[LC_NUMERIC] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i != 0 {
            (*(*locinfo).lconv).thousands_sep = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).thousands_sep.is_null() {
                WideCharToMultiByte(
                    cp[LC_NUMERIC] as u32,
                    0,
                    wbuf.as_ptr(),
                    -1,
                    (*(*locinfo).lconv).thousands_sep,
                    i,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }

        i = GetLocaleInfoEx(
            locale_sname[LC_NUMERIC],
            LOCALE_SGROUPING | LOCALE_NOUSEROVERRIDE,
            wbuf.as_mut_ptr(),
            256,
        );
        WideCharToMultiByte(
            cp[LC_NUMERIC] as u32,
            0,
            wbuf.as_ptr(),
            -1,
            buf.as_mut_ptr(),
            256,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if i > 1 {
            i = i / 2
                + (if buf[(i - 2) as usize] == b'0' as i8 {
                    0
                } else {
                    1
                });
        }
        if i != 0 {
            (*(*locinfo).lconv).grouping = malloc(i as usize) as *mut i8;
            if !(*(*locinfo).lconv).grouping.is_null() {
                let mut idx = 0;
                while buf[(idx + 1) as usize] == b';' as i8 {
                    *(*(*locinfo).lconv).grouping.add((idx / 2) as usize) =
                        buf[idx as usize] - b'0' as i8;
                    idx += 2;
                }
                *(*(*locinfo).lconv).grouping.add((idx / 2) as usize) =
                    buf[idx as usize] - b'0' as i8;
                if buf[idx as usize] != b'0' as i8 {
                    *(*(*locinfo).lconv).grouping.add((idx / 2 + 1) as usize) =
                        127;
                }
            } else {
                break 'fail;
            }
        } else {
            break 'fail;
        }
    } else {
        if (*locinfo).lconv != core::ptr::addr_of_mut!(cloc_lconv) {
            (*(*locinfo).lconv).decimal_point = cloc_lconv.decimal_point;
            (*(*locinfo).lconv).thousands_sep = cloc_lconv.thousands_sep;
            (*(*locinfo).lconv).grouping = cloc_lconv.grouping;
        }

        if init_category_name(
            b"C\0".as_ptr() as *const i8,
            1,
            locinfo,
            (LC_NUMERIC) as i32,
        ) == 0
        {
            break 'fail;
        }
    }

    if !locale_name[LC_TIME].is_null()
        && init_category_name(
            locale_name[LC_TIME],
            locale_len[LC_TIME],
            locinfo,
            (LC_TIME) as i32,
        ) == 0
    {
        break 'fail;
    }

    if category_needs_update(
        (LC_TIME) as i32,
        old_locinfo,
        locale_sname[LC_TIME],
        cp[LC_TIME],
    ) == 0
    {
        copy_threadlocinfo_category(locinfo, old_locinfo, (LC_TIME) as i32);
        (*locinfo).lc_time_curr = (*old_locinfo).lc_time_curr;
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*locinfo).lc_time_curr).refcount));
    } else if !locale_sname[LC_TIME].is_null() {
        if update_threadlocinfo_category(
            locale_sname[LC_TIME],
            cp[LC_TIME],
            locinfo,
            (LC_TIME) as i32,
        ) == 0
        {
            break 'fail;
        }

        (*locinfo).lc_time_curr = create_time_data(locale_sname[LC_TIME]);
        if (*locinfo).lc_time_curr.is_null() {
            break 'fail;
        }
    } else {
        if init_category_name(
            b"C\0".as_ptr() as *const i8,
            1,
            locinfo,
            (LC_TIME) as i32,
        ) == 0
        {
            break 'fail;
        }
        (*locinfo).lc_time_curr = core::ptr::addr_of_mut!(cloc_time_data);
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*locinfo).lc_time_curr).refcount));
    }

    i = 0;
    while i < (LC_MAX) as i32 {
        free((locale_sname[i as usize]).cast());
        i += 1;
    }

    return locinfo;
    } // 'fail

    free_locinfo(locinfo);

    i = 0;
    while i < (LC_MAX) as i32 {
        free((locale_sname[i as usize]).cast());
        i += 1;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn _create_locale(category: i32, locale: *const i8) -> _locale_t {
    let loc: _locale_t;

    loc = (malloc(core::mem::size_of::<_locale_tstruct>())).cast();
    if loc.is_null() {
        return core::ptr::null_mut();
    }

    (*loc).locinfo = create_locinfo(category, locale, core::ptr::null());
    if (*loc).locinfo.is_null() {
        free((loc).cast());
        return core::ptr::null_mut();
    }

    (*loc).mbcinfo = create_mbcinfo(
        ((*(*(*loc).locinfo).lc_id.as_ptr().offset(LC_CTYPE as isize)).wCodePage) as i32,
        (*(*loc).locinfo).lc_handle[LC_CTYPE],
        core::ptr::null_mut(),
    );
    if (*loc).mbcinfo.is_null() {
        free_locinfo((*loc).locinfo);
        free((loc).cast());
        return core::ptr::null_mut();
    }
    loc
}

#[no_mangle]
pub unsafe extern "C" fn setlocale(category: i32, locale: *const i8) -> *mut i8 {
    let data = msvcrt_get_thread_data();
    let locinfo = get_locinfo();
    let newlocinfo: pthreadlocinfo;
    let locale_flags: i32;

    if category < (LC_MIN) as i32 || category > (LC_MAX) as i32 {
        return core::ptr::null_mut();
    }

    if locale.is_null() {
        if category == (LC_ALL) as i32 {
            return construct_lc_all(locinfo);
        }

        return (*locinfo).lc_category[category as usize].locale;
    }

    locale_flags = (*data).locale_flags;
    (*data).locale_flags |= (LOCALE_THREAD) as i32;
    newlocinfo = create_locinfo(category, locale, locinfo);
    (*data).locale_flags = locale_flags;
    if newlocinfo.is_null() {
        return core::ptr::null_mut();
    }

    if *locale.offset(0) != b'C' as i8 || *locale.offset(1) != b'\0' as i8 {
        initial_locale = (FALSE) as i32;
    }

    if (*data).locale_flags & (LOCALE_THREAD) as i32 != 0 {
        if (*data).locale_flags & (LOCALE_FREE) as i32 != 0 {
            free_locinfo((*data).locinfo);
        }
        (*data).locinfo = newlocinfo;
    } else {
        let mut i: i32;

        _lock_locales();
        free_locinfo((*MSVCRT_locale).locinfo);
        (*MSVCRT_locale).locinfo = newlocinfo;

        MSVCRT___lc_codepage = (*newlocinfo).lc_codepage;
        MSVCRT___lc_collate_cp = ((*newlocinfo).lc_collate_cp) as i32;
        MSVCRT___mb_cur_max = (*newlocinfo).mb_cur_max;
        MSVCRT__pctype = (*newlocinfo).pctype;
        i = (LC_MIN) as i32;
        while i <= (LC_MAX) as i32 {
            MSVCRT___lc_handle[i as usize] = (*(*MSVCRT_locale).locinfo).lc_handle[i as usize];
            i += 1;
        }
        _unlock_locales();
        update_thread_locale(data);
    }

    if category == (LC_ALL) as i32 {
        return construct_lc_all((*data).locinfo);
    }

    return (*(*data).locinfo).lc_category[category as usize].locale;
}

#[no_mangle]
pub unsafe extern "C" fn _wsetlocale(category: i32, wlocale: *const wchar_t) -> *mut wchar_t {
    static mut current_lc_all: [wchar_t; MAX_LOCALE_LENGTH] = [0; MAX_LOCALE_LENGTH];

    let mut locale: *mut i8 = core::ptr::null_mut();
    let ret: *const i8;
    let len: usize;

    if !wlocale.is_null() {
        len = wcstombs(core::ptr::null_mut(), wlocale, 0);
        if len == (1 as usize).wrapping_neg() {
            return core::ptr::null_mut();
        }

        locale = (malloc(len.wrapping_add(1))).cast();
        if locale.is_null() {
            return core::ptr::null_mut();
        }

        wcstombs(locale, wlocale, len.wrapping_add(1));
    }

    _lock_locales();
    ret = setlocale(category, locale);
    free((locale).cast());

    if !ret.is_null() && mbstowcs((&raw mut current_lc_all).cast::<wchar_t>(), ret, MAX_LOCALE_LENGTH) == (1 as usize).wrapping_neg() {
        return core::ptr::null_mut();
    }

    _unlock_locales();
    if !ret.is_null() {
        (&raw mut current_lc_all).cast::<wchar_t>()
    } else {
        core::ptr::null_mut()
    }
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_locale() -> BOOL {
    let mut i: i32;

    _lock_locales();
    MSVCRT_locale = _create_locale(0, b"C\0".as_ptr() as *const i8);
    _unlock_locales();
    if MSVCRT_locale.is_null() {
        return FALSE as BOOL;
    }

    MSVCRT___lc_codepage = (*(*MSVCRT_locale).locinfo).lc_codepage;
    MSVCRT___lc_collate_cp = ((*(*MSVCRT_locale).locinfo).lc_collate_cp) as i32;
    MSVCRT___mb_cur_max = (*(*MSVCRT_locale).locinfo).mb_cur_max;
    MSVCRT__pctype = (*(*MSVCRT_locale).locinfo).pctype;
    i = (LC_MIN) as i32;
    while i <= (LC_MAX) as i32 {
        MSVCRT___lc_handle[i as usize] = (*(*MSVCRT_locale).locinfo).lc_handle[i as usize];
        i += 1;
    }
    _setmbcp(_MB_CP_ANSI);
    TRUE as BOOL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll locale functions
//  *
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <limits.h>
// #include <locale.h>
// #include <stdarg.h>
// #include <stdio.h>
// #include <stdlib.h>
// #include <mbctype.h>
// #include <wctype.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winuser.h"
// #include "winnls.h"
//
// #include "msvcrt.h"
// #include "mtdll.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// #define MAX_ELEM_LEN 64 /* Max length of country/language/CP string */
// #define MAX_LOCALE_LENGTH 256
// _locale_t MSVCRT_locale = NULL;
// unsigned short *MSVCRT__pctype = NULL;
// unsigned int MSVCRT___lc_codepage = 0;
// int MSVCRT___lc_collate_cp = 0;
// LCID MSVCRT___lc_handle[LC_MAX - LC_MIN + 1] = { 0 };
// int MSVCRT___mb_cur_max = 1;
// BOOL initial_locale = TRUE;
//
// #define MSVCRT_LEADBYTE  0x8000
// #define MSVCRT_C1_DEFINED 0x200
//
// #define LCID_CONVERSION_FLAGS 0
//
// __lc_time_data cloc_time_data =
// {
//     {{"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat",
//       "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
//       "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
//       "January", "February", "March", "April", "May", "June", "July",
//       "August", "September", "October", "November", "December",
//       "AM", "PM", "MM/dd/yy", "dddd, MMMM dd, yyyy", "HH:mm:ss"}},
//     MAKELCID(LANG_ENGLISH, SORT_DEFAULT),
//     1, -1,
//     {{L"Sun", L"Mon", L"Tue", L"Wed", L"Thu", L"Fri", L"Sat",
//       L"Sunday", L"Monday", L"Tuesday", L"Wednesday", L"Thursday", L"Friday", L"Saturday",
//       L"Jan", L"Feb", L"Mar", L"Apr", L"May", L"Jun", L"Jul", L"Aug", L"Sep", L"Oct", L"Nov", L"Dec",
//       L"January", L"February", L"March", L"April", L"May", L"June", L"July",
//       L"August", L"September", L"October", L"November", L"December",
//       L"AM", L"PM", L"MM/dd/yy", L"dddd, MMMM dd, yyyy", L"HH:mm:ss"}},
// };
//
// static const unsigned char cloc_clmap[256] =
// {
//     0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
//     0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
//     0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
//     0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
//     0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
//     0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
//     0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
//     0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
//     0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
//     0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
//     0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
//     0x78, 0x79, 0x7a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
//     0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
//     0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
//     0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
//     0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
//     0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
//     0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
//     0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
//     0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
//     0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
//     0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
//     0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
//     0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
//     0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
//     0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
//     0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
//     0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
//     0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
//     0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
//     0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
//     0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
// };
//
// static const unsigned char cloc_cumap[256] =
// {
//     0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
//     0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
//     0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
//     0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
//     0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
//     0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
//     0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
//     0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
//     0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
//     0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
//     0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
//     0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
//     0x60, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
//     0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
//     0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
//     0x58, 0x59, 0x5a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
//     0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
//     0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
//     0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
//     0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
//     0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
//     0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
//     0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
//     0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
//     0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
//     0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
//     0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
//     0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
//     0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
//     0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
//     0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
//     0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
// };
//
// static char empty[] = "";
// static char cloc_dec_point[] = ".";
// static struct lconv cloc_lconv =
// {
//     cloc_dec_point, empty, empty, empty, empty, empty, empty, empty, empty, empty,
//     CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
// };
//
// /* Friendly country strings & language names abbreviations. */
// static const char * const _country_synonyms[] =
// {
//     "american", "en",
//     "american english", "en-US",
//     "american-english", "en-US",
//     "english-american", "en-US",
//     "english-us", "en-US",
//     "english-usa", "en-US",
//     "us", "en-US",
//     "usa", "en-US",
//     "australian", "en-AU",
//     "english-aus", "en-AU",
//     "belgian", "nl-BE",
//     "french-belgian", "fr-BE",
//     "canadian", "en-CA",
//     "english-can", "en-CA",
//     "french-canadian", "fr-CA",
//     "chinese", "zh-CN",
//     "chinese-simplified", "zh-CN",
//     "chinese-traditional", "zh-TW",
//     "chs", "zh-CN",
//     "cht", "zh-TW",
//     "dutch-belgian", "nl-BE",
//     "english-nz", "en-NZ",
//     "uk", "en-GB",
//     "english-uk", "en-GB",
//     "french-swiss", "fr-CH",
//     "swiss", "de-CH",
//     "german-swiss", "de-CH",
//     "italian-swiss", "it-CH",
//     "german-austrian", "de-AT",
//     "portuguese", "pt-BR",
//     "portuguese-brazil", "pt-BR",
//     "spanish-mexican", "es-MX",
//     "norwegian-bokmal", "nb",
//     "norwegian-nynorsk", "nn-NO",
//     "spanish-modern", "es-ES"
// };
//
//
// /* INTERNAL: Map a synonym to an ISO code */
// static BOOL remap_synonym(char *name)
// {
//   unsigned int i;
//   for (i = 0; i < ARRAY_SIZE(_country_synonyms); i += 2)
//   {
//     if (!_stricmp(_country_synonyms[i],name))
//     {
//       TRACE(":Mapping synonym %s to %s\n",name,_country_synonyms[i+1]);
//       strcpy(name, _country_synonyms[i+1]);
//       return TRUE;
//     }
//   }
//
//   return FALSE;
// }
//
// /* Note: Flags are weighted in order of matching importance */
// #define FOUND_SNAME            0x4
// #define FOUND_LANGUAGE         0x2
// #define FOUND_COUNTRY          0x1
//
// typedef struct {
//   WCHAR search_language[MAX_ELEM_LEN];
//   WCHAR search_country[MAX_ELEM_LEN];
//   WCHAR found_lang_sname[LOCALE_NAME_MAX_LENGTH];
//   unsigned int match_flags;
//   BOOL allow_sname;
// } locale_search_t;
//
// #define CONTINUE_LOOKING TRUE
// #define STOP_LOOKING     FALSE
//
// /* INTERNAL: Get and compare locale info with a given string */
// static int compare_info(WCHAR *name, DWORD flags, WCHAR *buff, const WCHAR *cmp, BOOL exact)
// {
//   int len;
//
//   if(!cmp[0])
//     return 0;
//
//   buff[0] = 0;
//   GetLocaleInfoEx(name, flags|LOCALE_NOUSEROVERRIDE, buff, MAX_ELEM_LEN);
//   if (!buff[0])
//     return 0;
//
//   /* Partial matches are only allowed on language/country names */
//   len = wcslen(cmp);
//
//   if(exact || len<=3)
//     return !_wcsicmp(cmp, buff);
//   else
//     return !_wcsnicmp(cmp, buff, len);
// }
//
// static BOOL __stdcall
// find_best_locale_proc( WCHAR *name, DWORD locale_flags, LPARAM lParam )
// {
//   locale_search_t *res = (locale_search_t *)lParam;
//   WCHAR buff[MAX_ELEM_LEN];
//   unsigned int flags = 0;
//
//   if (res->allow_sname && compare_info(name,LOCALE_SNAME,buff,res->search_language, TRUE))
//   {
//     TRACE(":Found locale: %s->%s\n", wine_dbgstr_w(res->search_language), wine_dbgstr_w(buff));
//     res->match_flags = FOUND_SNAME;
//     wcscpy(res->found_lang_sname, name);
//     return STOP_LOOKING;
//   }
//
//   /* Check Language */
//   if (compare_info(name,LOCALE_SISO639LANGNAME,buff,res->search_language, TRUE) ||
//       compare_info(name,LOCALE_SABBREVLANGNAME,buff,res->search_language, TRUE) ||
//       compare_info(name,LOCALE_SENGLANGUAGE,buff,res->search_language, FALSE))
//   {
//     TRACE(":Found language: %s->%s\n", wine_dbgstr_w(res->search_language), wine_dbgstr_w(buff));
//     flags |= FOUND_LANGUAGE;
//   }
//   else if (res->match_flags & FOUND_LANGUAGE)
//   {
//     return CONTINUE_LOOKING;
//   }
//
//   /* Check Country */
//   if (compare_info(name,LOCALE_SISO3166CTRYNAME,buff,res->search_country, TRUE) ||
//       compare_info(name,LOCALE_SABBREVCTRYNAME,buff,res->search_country, TRUE) ||
//       compare_info(name,LOCALE_SENGCOUNTRY,buff,res->search_country, FALSE))
//   {
//     TRACE("Found country:%s->%s\n", wine_dbgstr_w(res->search_country), wine_dbgstr_w(buff));
//     flags |= FOUND_COUNTRY;
//   }
//   else if (!flags && (res->match_flags & FOUND_COUNTRY))
//   {
//     return CONTINUE_LOOKING;
//   }
//
//   if (flags > res->match_flags)
//   {
//     /* Found a better match than previously */
//     res->match_flags = flags;
//     wcscpy(res->found_lang_sname, name);
//   }
//   if ((flags & (FOUND_LANGUAGE | FOUND_COUNTRY)) ==
//         (FOUND_LANGUAGE | FOUND_COUNTRY))
//   {
//     TRACE(":found exact locale match\n");
//     return STOP_LOOKING;
//   }
//   return CONTINUE_LOOKING;
// }
//
// /* Internal: Find the sname for a locale specification.
//  * sname must be at least LOCALE_NAME_MAX_LENGTH characters long
//  */
// BOOL locale_to_sname(const char *locale, unsigned short *codepage, BOOL *sname_match, WCHAR *sname)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     const char *cp, *region;
//     BOOL is_sname = FALSE;
//     DWORD locale_cp;
//
//     if (!strcmp(locale, data->cached_locale)) {
//         if (codepage)
//             *codepage = data->cached_cp;
//         if (sname_match)
//             *sname_match = data->cached_sname_match;
//         wcscpy(sname, data->cached_sname);
//         return TRUE;
//     }
//
//     cp = strchr(locale, '.');
//     region = strchr(locale, '_');
//
//     if(!locale[0] || (cp == locale && !region)) {
//         GetUserDefaultLocaleName(sname, LOCALE_NAME_MAX_LENGTH);
//     } else {
//         char search_language_buf[MAX_ELEM_LEN] = { 0 }, search_country_buf[MAX_ELEM_LEN] = { 0 };
//         locale_search_t search;
//         BOOL remapped = FALSE;
//
//         memset(&search, 0, sizeof(locale_search_t));
//         lstrcpynA(search_language_buf, locale, MAX_ELEM_LEN);
//         if(region) {
//             lstrcpynA(search_country_buf, region+1, MAX_ELEM_LEN);
//             if(region-locale < MAX_ELEM_LEN)
//                 search_language_buf[region-locale] = '\0';
//         } else
//             search_country_buf[0] = '\0';
//
//         if(cp) {
//             if(region && cp-region-1<MAX_ELEM_LEN)
//                 search_country_buf[cp-region-1] = '\0';
//             if(cp-locale < MAX_ELEM_LEN)
//                 search_language_buf[cp-locale] = '\0';
//         }
//
//         if ((remapped = remap_synonym(search_language_buf)))
//         {
//             search.allow_sname = TRUE;
//         }
//
//
//         MultiByteToWideChar(CP_ACP, 0, search_language_buf, -1, search.search_language, MAX_ELEM_LEN);
//         if (search.allow_sname && IsValidLocaleName(search.search_language))
//         {
//             search.match_flags = FOUND_SNAME;
//             wcscpy(sname, search.search_language);
//         }
//         else
//         {
//             MultiByteToWideChar(CP_ACP, 0, search_country_buf, -1, search.search_country, MAX_ELEM_LEN);
//             EnumSystemLocalesEx( find_best_locale_proc, 0, (LPARAM)&search, NULL);
//
//             if (!search.match_flags)
//                 return FALSE;
//
//             /* If we were given something that didn't match, fail */
//             if (search.search_language[0] && !(search.match_flags & (FOUND_SNAME | FOUND_LANGUAGE)))
//                 return FALSE;
//             if (search.search_country[0] && !(search.match_flags & FOUND_COUNTRY))
//                 return FALSE;
//
//             wcscpy(sname, search.found_lang_sname);
//         }
//
//         is_sname = !remapped && (search.match_flags & FOUND_SNAME) != 0;
//     }
//
//     /* Obtain code page */
//     if (!cp || !cp[1] || !_strnicmp(cp, ".ACP", 4)) {
//         GetLocaleInfoEx(sname, LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
//                 (WCHAR *)&locale_cp, sizeof(DWORD)/sizeof(WCHAR));
//         if (!locale_cp)
//             locale_cp = GetACP();
//     } else if (!_strnicmp(cp, ".OCP", 4)) {
//         GetLocaleInfoEx(sname, LOCALE_IDEFAULTCODEPAGE | LOCALE_RETURN_NUMBER,
//                 (WCHAR *)&locale_cp, sizeof(DWORD)/sizeof(WCHAR));
//     } else {
//         locale_cp = atoi(cp + 1);
//     }
//     if (!IsValidCodePage(locale_cp))
//         return FALSE;
//
//     if (!locale_cp)
//         return FALSE;
//
//     if (codepage)
//         *codepage = locale_cp;
//     if (sname_match)
//         *sname_match = is_sname;
//
//     if (strlen(locale) < sizeof(data->cached_locale)) {
//         strcpy(data->cached_locale, locale);
//         data->cached_cp = locale_cp;
//         data->cached_sname_match = is_sname;
//         wcscpy(data->cached_sname, sname);
//     }
//
//     return TRUE;
// }
//
// static void copy_threadlocinfo_category(pthreadlocinfo locinfo,
//         const threadlocinfo *old_locinfo, int category)
// {
//     locinfo->lc_handle[category] = old_locinfo->lc_handle[category];
//     locinfo->lc_id[category] = old_locinfo->lc_id[category];
//     if(!locinfo->lc_category[category].locale) {
//         locinfo->lc_category[category].locale = old_locinfo->lc_category[category].locale;
//         locinfo->lc_category[category].refcount = old_locinfo->lc_category[category].refcount;
//         InterlockedIncrement((LONG *)locinfo->lc_category[category].refcount);
//     }
// }
//
// static BOOL init_category_name(const char *name, int len,
//         pthreadlocinfo locinfo, int category)
// {
//     locinfo->lc_category[category].locale = malloc(len+1);
//     locinfo->lc_category[category].refcount = malloc(sizeof(int));
//     if(!locinfo->lc_category[category].locale
//             || !locinfo->lc_category[category].refcount) {
//         free(locinfo->lc_category[category].locale);
//         free(locinfo->lc_category[category].refcount);
//         locinfo->lc_category[category].locale = NULL;
//         locinfo->lc_category[category].refcount = NULL;
//         return FALSE;
//     }
//
//     memcpy(locinfo->lc_category[category].locale, name, len);
//     locinfo->lc_category[category].locale[len] = 0;
//     *locinfo->lc_category[category].refcount = 1;
//     return TRUE;
// }
//
// static inline BOOL set_lc_locale_name(pthreadlocinfo locinfo, int cat, WCHAR *sname)
// {
//     return TRUE;
// }
//
// /* INTERNAL: Set lc_handle, lc_id and lc_category in threadlocinfo struct */
// static BOOL update_threadlocinfo_category(WCHAR *sname, unsigned short cp,
//         pthreadlocinfo locinfo, int category)
// {
//     WCHAR wbuf[256], *p;
//
//     if(GetLocaleInfoEx(sname, LOCALE_ILANGUAGE|LOCALE_NOUSEROVERRIDE, wbuf, ARRAY_SIZE(wbuf))) {
//         p = wbuf;
//
//         locinfo->lc_id[category].wLanguage = 0;
//         while(*p) {
//             locinfo->lc_id[category].wLanguage *= 16;
//
//             if(*p <= '9')
//                 locinfo->lc_id[category].wLanguage += *p-'0';
//             else
//                 locinfo->lc_id[category].wLanguage += *p-'a'+10;
//
//             p++;
//         }
//
//         locinfo->lc_id[category].wCountry =
//             locinfo->lc_id[category].wLanguage;
//     }
//
//     locinfo->lc_id[category].wCodePage = cp;
//
//     locinfo->lc_handle[category] = LocaleNameToLCID(sname, LCID_CONVERSION_FLAGS);
//
//     set_lc_locale_name(locinfo, category, sname);
//
//     if(!locinfo->lc_category[category].locale) {
//         char buf[256];
//         int len = 0;
//
//         if (LANGIDFROMLCID!(locinfo->lc_handle[category]) == MAKELANGID!(LANG_NORWEGIAN, SUBLANG_NORWEGIAN_NYNORSK))
//         {
//             /* locale.nls contains "Norwegian Nynorsk" instead for LOCALE_SENGLANGUAGE */
//             wcscpy( wbuf, L"Norwegian-Nynorsk" );
//             len = wcslen( wbuf ) + 1;
//         }
//         else
//             len += GetLocaleInfoEx(sname, LOCALE_SENGLANGUAGE|LOCALE_NOUSEROVERRIDE, wbuf, ARRAY_SIZE(wbuf));
//         wbuf[len-1] = '_';
//         len += GetLocaleInfoEx(sname, LOCALE_SENGCOUNTRY
//                 |LOCALE_NOUSEROVERRIDE, &wbuf[len], ARRAY_SIZE(wbuf) - len);
//         wbuf[len-1] = '.';
//         swprintf(wbuf+len, ARRAY_SIZE(wbuf) - len,L"%d", cp);
//         len += wcslen(wbuf+len);
//
//         WideCharToMultiByte(cp, 0, wbuf, -1, buf, ARRAY_SIZE(buf), NULL, NULL);
//
//         return init_category_name(buf, len, locinfo, category);
//     }
//
//     return TRUE;
// }
//
// /*********************************************************************
//  *      _lock_locales (UCRTBASE.@)
//  */
// void CDECL _lock_locales(void)
// {
//     _lock(_SETLOCALE_LOCK);
// }
//
// /*********************************************************************
//  *      _unlock_locales (UCRTBASE.@)
//  */
// void CDECL _unlock_locales(void)
// {
//     _unlock(_SETLOCALE_LOCK);
// }
//
// static void grab_locinfo(pthreadlocinfo locinfo)
// {
//     int i;
//
//     InterlockedIncrement((LONG *)&locinfo->refcount);
//     for(i=LC_MIN+1; i<=LC_MAX; i++)
//     {
//         if(locinfo->lc_category[i].refcount)
//             InterlockedIncrement((LONG *)locinfo->lc_category[i].refcount);
//         if(locinfo->lc_category[i].wrefcount)
//             InterlockedIncrement((LONG *)locinfo->lc_category[i].wrefcount);
//     }
//     if(locinfo->lconv_intl_refcount)
//         InterlockedIncrement((LONG *)locinfo->lconv_intl_refcount);
//     if(locinfo->lconv_num_refcount)
//         InterlockedIncrement((LONG *)locinfo->lconv_num_refcount);
//     if(locinfo->lconv_mon_refcount)
//         InterlockedIncrement((LONG *)locinfo->lconv_mon_refcount);
//     if(locinfo->ctype1_refcount)
//         InterlockedIncrement((LONG *)locinfo->ctype1_refcount);
//     if(locinfo->lc_time_curr)
//         InterlockedIncrement(&locinfo->lc_time_curr->refcount);
// }
//
// /*
//  * Static C-locale fallbacks for processes where DllMain was never called
//  * (e.g. DLL mapped via SEC_IMAGE but not initialized by the PE loader).
//  * These provide safe defaults so get_locinfo()/get_mbcinfo() never return NULL.
//  */
// static threadlocinfo cloc_static_locinfo = {
//     0x7FFFFFFF, /* refcount — pinned, never freed */
//     0,          /* lc_codepage */
//     0,          /* lc_collate_cp */
//     {0},        /* lc_handle */
//     {{0}},      /* lc_id */
//     {{0}},      /* lc_category — locale strings are NULL but that's OK for basic ops */
//     1,          /* lc_clike */
//     1,          /* mb_cur_max */
//     NULL,       /* lconv_intl_refcount */
//     NULL,       /* lconv_num_refcount */
//     NULL,       /* lconv_mon_refcount */
//     NULL,       /* lconv — set at runtime to &cloc_lconv */
//     NULL,       /* ctype1_refcount */
//     NULL,       /* ctype1 */
//     NULL,       /* pctype — set at runtime to MSVCRT__ctype+1 */
//     NULL,       /* pclmap — set at runtime to cloc_clmap */
//     NULL,       /* pcumap — set at runtime to cloc_cumap */
//     NULL,       /* lc_time_curr — set at runtime to &cloc_time_data */
// };
//
// static threadmbcinfo cloc_static_mbcinfo = {
//     0x7FFFFFFF, /* refcount — pinned, never freed */
//     0,          /* mbcodepage — C locale */
//     0,          /* ismbcodepage */
//     0,          /* mblcid */
//     {0},        /* mbulinfo */
//     {0},        /* mbctype */
//     {0},        /* mbcasemap */
// };
//
// static LONG cloc_static_initialized = 0;
//
// static void ensure_cloc_static_init(void)
// {
//     if(InterlockedCompareExchange(&cloc_static_initialized, 1, 0) == 0)
//     {
//         cloc_static_locinfo.lconv = &cloc_lconv;
//         cloc_static_locinfo.pctype = MSVCRT__ctype + 1;
//         cloc_static_locinfo.pclmap = cloc_clmap;
//         cloc_static_locinfo.pcumap = cloc_cumap;
//         cloc_static_locinfo.lc_time_curr = &cloc_time_data;
//     }
// }
//
// static void update_thread_locale(thread_data_t *data)
// {
//     if(!MSVCRT_locale)
//         return;
//
//     if((data->locale_flags & LOCALE_FREE) && ((data->locale_flags & LOCALE_THREAD) ||
//                 (data->locinfo == MSVCRT_locale->locinfo && data->mbcinfo == MSVCRT_locale->mbcinfo)))
//         return;
//
//     if(data->locale_flags & LOCALE_FREE)
//     {
//         free_locinfo(data->locinfo);
//         free_mbcinfo(data->mbcinfo);
//     }
//
//     _lock_locales();
//     data->locinfo = MSVCRT_locale->locinfo;
//     grab_locinfo(data->locinfo);
//     _unlock_locales();
//
//     _lock(_MB_CP_LOCK);
//     data->mbcinfo = MSVCRT_locale->mbcinfo;
//     InterlockedIncrement(&data->mbcinfo->refcount);
//     _unlock(_MB_CP_LOCK);
//
//     data->locale_flags |= LOCALE_FREE;
// }
//
// /* INTERNAL: returns threadlocinfo struct */
// pthreadlocinfo CDECL get_locinfo(void) {
//     thread_data_t *data = msvcrt_get_thread_data();
//     update_thread_locale(data);
//     if(!data->locinfo) {
//         ensure_cloc_static_init();
//         return &cloc_static_locinfo;
//     }
//     return data->locinfo;
// }
//
// /* INTERNAL: returns pthreadmbcinfo struct */
// pthreadmbcinfo CDECL get_mbcinfo(void) {
//     thread_data_t *data = msvcrt_get_thread_data();
//     update_thread_locale(data);
//     if(!data->mbcinfo) {
//         ensure_cloc_static_init();
//         return &cloc_static_mbcinfo;
//     }
//     return data->mbcinfo;
// }
//
// /* INTERNAL: constructs string returned by setlocale */
// static inline char* construct_lc_all(pthreadlocinfo locinfo) {
//     static char current_lc_all[MAX_LOCALE_LENGTH];
//
//     int i;
//
//     for(i=LC_MIN+1; i<LC_MAX; i++) {
//         if(strcmp(locinfo->lc_category[i].locale,
//                     locinfo->lc_category[i+1].locale))
//             break;
//     }
//
//     if(i==LC_MAX)
//         return locinfo->lc_category[LC_COLLATE].locale;
//
//     sprintf(current_lc_all,
//             "LC_COLLATE=%s;LC_CTYPE=%s;LC_MONETARY=%s;LC_NUMERIC=%s;LC_TIME=%s",
//             locinfo->lc_category[LC_COLLATE].locale,
//             locinfo->lc_category[LC_CTYPE].locale,
//             locinfo->lc_category[LC_MONETARY].locale,
//             locinfo->lc_category[LC_NUMERIC].locale,
//             locinfo->lc_category[LC_TIME].locale);
//
//     return current_lc_all;
// }
//
//
// /*********************************************************************
//  *		_Getdays (MSVCRT.@)
//  */
// char*  _Getdays(void)
// {
//     __lc_time_data *cur = get_locinfo()->lc_time_curr;
//     int i, len, size = 0;
//     char *out;
//
//     TRACE("\n");
//
//     for(i=0; i<7; i++) {
//         size += strlen(cur->str.names.short_wday[i]) + 1;
//         size += strlen(cur->str.names.wday[i]) + 1;
//     }
//     out = malloc(size+1);
//     if(!out)
//         return NULL;
//
//     size = 0;
//     for(i=0; i<7; i++) {
//         out[size++] = ':';
//         len = strlen(cur->str.names.short_wday[i]);
//         memcpy(&out[size], cur->str.names.short_wday[i], len);
//         size += len;
//
//         out[size++] = ':';
//         len = strlen(cur->str.names.wday[i]);
//         memcpy(&out[size], cur->str.names.wday[i], len);
//         size += len;
//     }
//     out[size] = '\0';
//
//     return out;
// }
//
//
// /*********************************************************************
//  *		_Getmonths (MSVCRT.@)
//  */
// char*  _Getmonths(void)
// {
//     __lc_time_data *cur = get_locinfo()->lc_time_curr;
//     int i, len, size = 0;
//     char *out;
//
//     TRACE("\n");
//
//     for(i=0; i<12; i++) {
//         size += strlen(cur->str.names.short_mon[i]) + 1;
//         size += strlen(cur->str.names.mon[i]) + 1;
//     }
//     out = malloc(size+1);
//     if(!out)
//         return NULL;
//
//     size = 0;
//     for(i=0; i<12; i++) {
//         out[size++] = ':';
//         len = strlen(cur->str.names.short_mon[i]);
//         memcpy(&out[size], cur->str.names.short_mon[i], len);
//         size += len;
//
//         out[size++] = ':';
//         len = strlen(cur->str.names.mon[i]);
//         memcpy(&out[size], cur->str.names.mon[i], len);
//         size += len;
//     }
//     out[size] = '\0';
//
//     return out;
// }
//
//
// /*********************************************************************
//  *		_Gettnames (MSVCRT.@)
//  */
// void*  _Gettnames(void)
// {
//     __lc_time_data *ret, *cur = get_locinfo()->lc_time_curr;
//     unsigned int i, len, size = sizeof(__lc_time_data);
//
//     TRACE("\n");
//
//     for(i=0; i<ARRAY_SIZE(cur->str.str); i++)
//         size += strlen(cur->str.str[i])+1;
//
//     ret = malloc(size);
//     if(!ret)
//         return NULL;
//     memcpy(ret, cur, sizeof(*ret));
//
//     size = 0;
//     for(i=0; i<ARRAY_SIZE(cur->str.str); i++) {
//         len = strlen(cur->str.str[i])+1;
//         memcpy(&ret->data[size], cur->str.str[i], len);
//         ret->str.str[i] = &ret->data[size];
//         size += len;
//     }
//
//     return ret;
// }
//
//
// /*********************************************************************
//  *		__crtLCMapStringA (MSVCRT.@)
//  */
// int CDECL __crtLCMapStringA(
//   LCID lcid, DWORD mapflags, const char* src, int srclen, char* dst,
//   int dstlen, unsigned int codepage, int xflag
// ) {
//     WCHAR buf_in[32], *in = buf_in;
//     WCHAR buf_out[32], *out = buf_out;
//     int in_len, out_len, r;
//
//     TRACE("(lcid %lx, flags %lx, %s(%d), %p(%d), %x, %d), partial stub!\n",
//             lcid, mapflags, src, srclen, dst, dstlen, codepage, xflag);
//
//     in_len = MultiByteToWideChar(codepage, MB_ERR_INVALID_CHARS, src, srclen, NULL, 0);
//     if (!in_len) return 0;
//     if (in_len > ARRAY_SIZE(buf_in))
//     {
//         in = malloc(in_len * sizeof(WCHAR));
//         if (!in) return 0;
//     }
//
//     r = MultiByteToWideChar(codepage, MB_ERR_INVALID_CHARS, src, srclen, in, in_len);
//     if (!r) goto done;
//
//     if (mapflags & LCMAP_SORTKEY)
//     {
//         r = LCMapStringW(lcid, mapflags, in, in_len, (WCHAR*)dst, dstlen);
//         goto done;
//     }
//
//     r = LCMapStringW(lcid, mapflags, in, in_len, NULL, 0);
//     if (!r) goto done;
//     out_len = r;
//     if (r > ARRAY_SIZE(buf_out))
//     {
//         out = malloc(r * sizeof(WCHAR));
//         if (!out)
//         {
//             r = 0;
//             goto done;
//         }
//     }
//
//     r = LCMapStringW(lcid, mapflags, in, in_len, out, out_len);
//     if (!r) goto done;
//
//     r = WideCharToMultiByte(codepage, 0, out, out_len, dst, dstlen, NULL, NULL);
//
// done:
//     if (in != buf_in) free(in);
//     if (out != buf_out) free(out);
//     return r;
// }
//
// /*********************************************************************
//  *              __crtLCMapStringW (MSVCRT.@)
//  */
// int CDECL __crtLCMapStringW(LCID lcid, DWORD mapflags, const wchar_t *src,
//         int srclen, wchar_t *dst, int dstlen, unsigned int codepage, int xflag)
// {
//     FIXME("(lcid %lx, flags %lx, %s(%d), %p(%d), %x, %d), partial stub!\n",
//             lcid, mapflags, debugstr_w(src), srclen, dst, dstlen, codepage, xflag);
//
//     return LCMapStringW(lcid, mapflags, src, srclen, dst, dstlen);
// }
//
// /*********************************************************************
//  *		__crtCompareStringA (MSVCRT.@)
//  */
// int CDECL __crtCompareStringA( LCID lcid, DWORD flags, const char *src1, int len1,
//                                const char *src2, int len2 )
// {
//     FIXME("(lcid %lx, flags %lx, %s(%d), %s(%d), partial stub\n",
//           lcid, flags, debugstr_a(src1), len1, debugstr_a(src2), len2 );
//     /* FIXME: probably not entirely right */
//     return CompareStringA( lcid, flags, src1, len1, src2, len2 );
// }
//
// /*********************************************************************
//  *		__crtCompareStringW (MSVCRT.@)
//  */
// int CDECL __crtCompareStringW( LCID lcid, DWORD flags, const wchar_t *src1, int len1,
//                                const wchar_t *src2, int len2 )
// {
//     FIXME("(lcid %lx, flags %lx, %s(%d), %s(%d), partial stub\n",
//           lcid, flags, debugstr_w(src1), len1, debugstr_w(src2), len2 );
//     /* FIXME: probably not entirely right */
//     return CompareStringW( lcid, flags, src1, len1, src2, len2 );
// }
//
// /*********************************************************************
//  *		__crtGetLocaleInfoW (MSVCRT.@)
//  */
// int CDECL __crtGetLocaleInfoW( LCID lcid, LCTYPE type, wchar_t *buffer, int len )
// {
//     FIXME("(lcid %lx, type %lx, %p(%d), partial stub\n", lcid, type, buffer, len );
//     /* FIXME: probably not entirely right */
//     return GetLocaleInfoW( lcid, type, buffer, len );
// }
//
//
// /*********************************************************************
//  *              __crtGetStringTypeW(MSVCRT.@)
//  *
//  * This function was accepting different number of arguments in older
//  * versions of msvcrt.
//  */
// BOOL CDECL __crtGetStringTypeW(DWORD unk, DWORD type,
//         wchar_t *buffer, int len, WORD *out)
// {
//     FIXME("(unk %lx, type %lx, wstr %p(%d), %p) partial stub\n",
//             unk, type, buffer, len, out);
//
//     return GetStringTypeW(type, buffer, len, out);
// }
//
// /*********************************************************************
//  *		localeconv (MSVCRT.@)
//  */
// struct lconv*  localeconv(void)
// {
//     return get_locinfo()->lconv;
// }
//
// /*********************************************************************
//  *		__lconv_init (MSVCRT.@)
//  */
// int CDECL __lconv_init(void)
// {
//     /* this is used to make chars unsigned */
//     cloc_lconv.int_frac_digits = (char)UCHAR_MAX;
//     cloc_lconv.frac_digits = (char)UCHAR_MAX;
//     cloc_lconv.p_cs_precedes = (char)UCHAR_MAX;
//     cloc_lconv.p_sep_by_space = (char)UCHAR_MAX;
//     cloc_lconv.n_cs_precedes = (char)UCHAR_MAX;
//     cloc_lconv.n_sep_by_space = (char)UCHAR_MAX;
//     cloc_lconv.p_sign_posn = (char)UCHAR_MAX;
//     cloc_lconv.n_sign_posn = (char)UCHAR_MAX;
//     return 0;
// }
//
// /*********************************************************************
//  *      ___lc_handle_func (MSVCRT.@)
//  */
// LCID*  ___lc_handle_func(void)
// {
//     return (LCID *)get_locinfo()->lc_handle;
// }
//
//
// /*********************************************************************
//  *      ___lc_codepage_func (MSVCRT.@)
//  */
// unsigned int CDECL ___lc_codepage_func(void)
// {
//     return get_locinfo()->lc_codepage;
// }
//
// /*********************************************************************
//  *      ___lc_collate_cp_func (MSVCRT.@)
//  */
// int CDECL ___lc_collate_cp_func(void)
// {
//     return get_locinfo()->lc_collate_cp;
// }
//
// /* INTERNAL: frees pthreadlocinfo struct */
// void free_locinfo(pthreadlocinfo locinfo)
// {
//     int i;
//
//     if(!locinfo)
//         return;
//
//     for(i=LC_MIN+1; i<=LC_MAX; i++) {
//         if(!locinfo->lc_category[i].refcount
//                 || !InterlockedDecrement((LONG *)locinfo->lc_category[i].refcount)) {
//             free(locinfo->lc_category[i].locale);
//             free(locinfo->lc_category[i].refcount);
//         }
//         if(!locinfo->lc_category[i].wrefcount
//                 || !InterlockedDecrement((LONG *)locinfo->lc_category[i].wrefcount)) {
//             free(locinfo->lc_category[i].wrefcount);
//         }
//     }
//
//     if(locinfo->lconv_num_refcount
//             && !InterlockedDecrement((LONG *)locinfo->lconv_num_refcount)) {
//         free(locinfo->lconv->decimal_point);
//         free(locinfo->lconv->thousands_sep);
//         free(locinfo->lconv->grouping);
//         free(locinfo->lconv_num_refcount);
//     }
//     if(locinfo->lconv_mon_refcount
//             && !InterlockedDecrement((LONG *)locinfo->lconv_mon_refcount)) {
//         free(locinfo->lconv->int_curr_symbol);
//         free(locinfo->lconv->currency_symbol);
//         free(locinfo->lconv->mon_decimal_point);
//         free(locinfo->lconv->mon_thousands_sep);
//         free(locinfo->lconv->mon_grouping);
//         free(locinfo->lconv->positive_sign);
//         free(locinfo->lconv->negative_sign);
//         free(locinfo->lconv_mon_refcount);
//     }
//     if(locinfo->lconv_intl_refcount
//             && !InterlockedDecrement((LONG *)locinfo->lconv_intl_refcount)) {
//         free(locinfo->lconv_intl_refcount);
//         free(locinfo->lconv);
//     }
//
//     if(locinfo->ctype1_refcount
//             && !InterlockedDecrement((LONG *)locinfo->ctype1_refcount)) {
//         free(locinfo->ctype1_refcount);
//         free(locinfo->ctype1);
//         free((void*)locinfo->pclmap);
//         free((void*)locinfo->pcumap);
//     }
//
//     if(locinfo->lc_time_curr && !InterlockedDecrement(&locinfo->lc_time_curr->refcount)
//             && locinfo->lc_time_curr != &cloc_time_data)
//         free(locinfo->lc_time_curr);
//
//     if(InterlockedDecrement((LONG *)&locinfo->refcount))
//         return;
//
//     free(locinfo);
// }
//
// /* INTERNAL: frees pthreadmbcinfo struct */
// void free_mbcinfo(pthreadmbcinfo mbcinfo)
// {
//     if(!mbcinfo)
//         return;
//
//     if(InterlockedDecrement(&mbcinfo->refcount))
//         return;
//
//     free(mbcinfo);
// }
//
// _locale_t CDECL get_current_locale_noalloc(_locale_t locale)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     update_thread_locale(data);
//     locale->locinfo = data->locinfo;
//     locale->mbcinfo = data->mbcinfo;
//
//     if(!locale->locinfo) {
//         ensure_cloc_static_init();
//         locale->locinfo = &cloc_static_locinfo;
//     }
//     if(!locale->mbcinfo) {
//         ensure_cloc_static_init();
//         locale->mbcinfo = &cloc_static_mbcinfo;
//     }
//
//     grab_locinfo(locale->locinfo);
//     InterlockedIncrement(&locale->mbcinfo->refcount);
//     return locale;
// }
//
// void CDECL free_locale_noalloc(_locale_t locale)
// {
//     free_locinfo(locale->locinfo);
//     free_mbcinfo(locale->mbcinfo);
// }
//
// /*********************************************************************
//  *      _get_current_locale (MSVCRT.@)
//  */
// _locale_t CDECL _get_current_locale(void)
// {
//     _locale_t loc = malloc(sizeof(_locale_tstruct));
//     if(!loc)
//         return NULL;
//
//     return get_current_locale_noalloc(loc);
// }
//
// /*********************************************************************
//  *      _free_locale (MSVCRT.@)
//  */
// void CDECL _free_locale(_locale_t locale)
// {
//     if (!locale)
//         return;
//
//     free_locale_noalloc(locale);
//     free(locale);
// }
//
// static inline BOOL category_needs_update(int cat,
//         const threadlocinfo *locinfo, WCHAR *sname, unsigned short cp)
// {
//     LCID lcid;
//     if(!locinfo) return TRUE;
//     lcid = sname ? LocaleNameToLCID(sname, 0) : 0;
//     return lcid!=locinfo->lc_handle[cat] || cp!=locinfo->lc_id[cat].wCodePage;
// }
//
// static __lc_time_data* create_time_data(WCHAR *sname)
// {
//     static const DWORD time_data[] = {
//         LOCALE_SABBREVDAYNAME7, LOCALE_SABBREVDAYNAME1, LOCALE_SABBREVDAYNAME2,
//         LOCALE_SABBREVDAYNAME3, LOCALE_SABBREVDAYNAME4, LOCALE_SABBREVDAYNAME5,
//         LOCALE_SABBREVDAYNAME6,
//         LOCALE_SDAYNAME7, LOCALE_SDAYNAME1, LOCALE_SDAYNAME2, LOCALE_SDAYNAME3,
//         LOCALE_SDAYNAME4, LOCALE_SDAYNAME5, LOCALE_SDAYNAME6,
//         LOCALE_SABBREVMONTHNAME1, LOCALE_SABBREVMONTHNAME2, LOCALE_SABBREVMONTHNAME3,
//         LOCALE_SABBREVMONTHNAME4, LOCALE_SABBREVMONTHNAME5, LOCALE_SABBREVMONTHNAME6,
//         LOCALE_SABBREVMONTHNAME7, LOCALE_SABBREVMONTHNAME8, LOCALE_SABBREVMONTHNAME9,
//         LOCALE_SABBREVMONTHNAME10, LOCALE_SABBREVMONTHNAME11, LOCALE_SABBREVMONTHNAME12,
//         LOCALE_SMONTHNAME1, LOCALE_SMONTHNAME2, LOCALE_SMONTHNAME3, LOCALE_SMONTHNAME4,
//         LOCALE_SMONTHNAME5, LOCALE_SMONTHNAME6, LOCALE_SMONTHNAME7, LOCALE_SMONTHNAME8,
//         LOCALE_SMONTHNAME9, LOCALE_SMONTHNAME10, LOCALE_SMONTHNAME11, LOCALE_SMONTHNAME12,
//         LOCALE_S1159, LOCALE_S2359,
//         LOCALE_SSHORTDATE, LOCALE_SLONGDATE,
//         LOCALE_STIMEFORMAT
//     };
//
//     __lc_time_data *cur;
//     int i, ret, size;
//     LCID lcid = LocaleNameToLCID(sname, LCID_CONVERSION_FLAGS);
//
//     size = 0;
//     for(i=0; i<ARRAY_SIZE(time_data); i++) {
//         ret = GetLocaleInfoA(lcid, time_data[i], NULL, 0);
//         if(!ret)
//             return NULL;
//         size += ret;
//
//         ret = GetLocaleInfoEx(sname, time_data[i], NULL, 0);
//         if(!ret)
//             return NULL;
//         size += ret*sizeof(wchar_t);
//     }
//
//     cur = malloc(FIELD_OFFSET!(__lc_time_data, data[size]));
//     if(!cur)
//         return NULL;
//
//     ret = 0;
//     for(i=0; i<ARRAY_SIZE(time_data); i++) {
//         cur->str.str[i] = &cur->data[ret];
//         ret += GetLocaleInfoA(lcid, time_data[i], &cur->data[ret], size-ret);
//     }
//     for(i=0; i<ARRAY_SIZE(time_data); i++) {
//         cur->wstr.wstr[i] = (wchar_t*)&cur->data[ret];
//         ret += GetLocaleInfoEx(sname, time_data[i], (wchar_t*)&cur->data[ret],
//                 (size - ret) / sizeof(wchar_t)) * sizeof(wchar_t);
//     }
//     cur->lcid = lcid;
//     cur->unk = 1;
//     cur->refcount = 1;
//
//     return cur;
// }
//
// static pthreadlocinfo create_locinfo(int category,
//         const char *locale, const threadlocinfo *old_locinfo)
// {
//     static const char collate[] = "COLLATE=";
//     static const char ctype[] = "CTYPE=";
//     static const char monetary[] = "MONETARY=";
//     static const char numeric[] = "NUMERIC=";
//     static const char time[] = "TIME=";
//
//     pthreadlocinfo locinfo = NULL;
//     unsigned short cp[6] = { 0 };
//     const char *locale_name[6] = { 0 };
//     WCHAR *locale_sname[6] = { 0 };
//     int val, locale_len[6] = { 0 };
//     char buf[256];
//     BOOL sname_match;
//     wchar_t wbuf[256], map_buf[256];
//     int i;
//
//     TRACE("(%d %s)\n", category, locale);
//
//     if(category<LC_MIN || category>LC_MAX || !locale)
//         return NULL;
//
//     if(locale[0]=='C' && !locale[1]) {
//         locale_sname[0] = NULL;
//         cp[0] = CP_ACP;
//     } else if (locale[0] == 'L' && locale[1] == 'C' && locale[2] == '_') {
//         const char *p;
//
//         while(1) {
//             locale += 3; /* LC_ */
//             if(!memcmp(locale, collate, sizeof(collate)-1)) {
//                 i = LC_COLLATE;
//                 locale += sizeof(collate)-1;
//             } else if(!memcmp(locale, ctype, sizeof(ctype)-1)) {
//                 i = LC_CTYPE;
//                 locale += sizeof(ctype)-1;
//             } else if(!memcmp(locale, monetary, sizeof(monetary)-1)) {
//                 i = LC_MONETARY;
//                 locale += sizeof(monetary)-1;
//             } else if(!memcmp(locale, numeric, sizeof(numeric)-1)) {
//                 i = LC_NUMERIC;
//                 locale += sizeof(numeric)-1;
//             } else if(!memcmp(locale, time, sizeof(time)-1)) {
//                 i = LC_TIME;
//                 locale += sizeof(time)-1;
//             } else
//                 goto fail;
//
//             p = strchr(locale, ';');
//             if(locale[0]=='C' && (locale[1]==';' || locale[1]=='\0')) {
//                 locale_sname[i] = NULL;
//                 cp[i] = CP_ACP;
//             } else {
//                 BOOL locale_found = FALSE;
//
//                 if(p) {
//                     memcpy(buf, locale, p-locale);
//                     buf[p-locale] = '\0';
//                     locale_found = locale_to_sname(buf, &cp[i], &sname_match, wbuf);
//                 } else {
//                     locale_found = locale_to_sname(locale, &cp[i], &sname_match, wbuf);
//                 }
//
//                 if(!locale_found || !(locale_sname[i] = wcsdup(wbuf)))
//                     goto fail;
//                 if(sname_match) {
//                     locale_name[i] = locale;
//                     locale_len[i] = p ? p-locale : strlen(locale);
//                 }
//             }
//
//             if(!p || *(p+1)!='L' || *(p+2)!='C' || *(p+3)!='_')
//                 break;
//
//             locale = p+1;
//         }
//     } else {
//         BOOL locale_found = locale_to_sname(locale, &cp[0], &sname_match, wbuf);
//
//         if(!locale_found)
//             return NULL;
//
//         locale_sname[0] = wcsdup(wbuf);
//         if(!locale_sname[0])
//             return NULL;
//
//         if(sname_match) {
//             locale_name[0] = locale;
//             locale_len[0] = strlen(locale);
//         }
//
//         for(i=1; i<6; i++) {
//             locale_sname[i] = wcsdup(locale_sname[0]);
//             if(!locale_sname[i])
//                 goto fail;
//
//             cp[i] = cp[0];
//             locale_name[i] = locale_name[0];
//             locale_len[i] = locale_len[0];
//         }
//     }
//
//     for(i=1; i<6; i++) {
//         if(i==LC_CTYPE && cp[i]==CP_UTF8) {
// #if _MSVCR_VER >= 110
//             if(old_locinfo) {
//                 locale_sname[i] = wcsdup(old_locinfo->lc_name[i]);
//                 if (old_locinfo->lc_name[i] && !locale_sname[i])
//                     goto fail;
//             }
// #else
//             int sname_size;
//             if(old_locinfo && old_locinfo->lc_handle[i]) {
//                 sname_size = LCIDToLocaleName(old_locinfo->lc_handle[i], NULL, 0, 0);
//                 locale_sname[i] = malloc(sname_size * sizeof(WCHAR));
//                 if(!locale_sname[i])
//                     goto fail;
//                 LCIDToLocaleName(old_locinfo->lc_handle[i], locale_sname[i], sname_size, 0);
//             } else {
//                 locale_sname[i] = NULL;
//             }
// #endif
//
//             locale_name[i] = NULL;
//             locale_len[i] = 0;
//             cp[i] = old_locinfo ? old_locinfo->lc_id[i].wCodePage : 0;
//         }
//         if(category!=LC_ALL && category!=i) {
//             if(old_locinfo) {
//                 int sname_size;
//                 if(old_locinfo->lc_handle[i]) {
//                     sname_size = LCIDToLocaleName(old_locinfo->lc_handle[i], NULL, 0, 0);
//                     locale_sname[i] = malloc(sname_size * sizeof(WCHAR));
//                     if(!locale_sname[i])
//                         goto fail;
//                     LCIDToLocaleName(old_locinfo->lc_handle[i], locale_sname[i], sname_size, 0);
//                 } else {
//                     locale_sname[i] = NULL;
//                 }
//                 cp[i] = old_locinfo->lc_id[i].wCodePage;
//             } else {
//                 locale_sname[i] = NULL;
//                 cp[i] = 0;
//             }
//         }
//     }
//
//     locinfo = malloc(sizeof(threadlocinfo));
//     if(!locinfo)
//         goto fail;
//
//     memset(locinfo, 0, sizeof(threadlocinfo));
//     locinfo->refcount = 1;
//
//     if(locale_name[LC_COLLATE] &&
//             !init_category_name(locale_name[LC_COLLATE],
//                 locale_len[LC_COLLATE], locinfo, LC_COLLATE)) {
//         goto fail;
//     }
//
//     if(!category_needs_update(LC_COLLATE, old_locinfo,
//                 locale_sname[LC_COLLATE], cp[LC_COLLATE])) {
//         copy_threadlocinfo_category(locinfo, old_locinfo, LC_COLLATE);
//         locinfo->lc_collate_cp = old_locinfo->lc_collate_cp;
//     } else if(locale_sname[LC_COLLATE]) {
//         if(!update_threadlocinfo_category(locale_sname[LC_COLLATE],
//                     cp[LC_COLLATE], locinfo, LC_COLLATE)) {
//             goto fail;
//         }
//
//         locinfo->lc_collate_cp = locinfo->lc_id[LC_COLLATE].wCodePage;
//     } else {
//         if(!init_category_name("C", 1, locinfo, LC_COLLATE)) {
//             goto fail;
//         }
//     }
//
//     if(locale_name[LC_CTYPE] &&
//             !init_category_name(locale_name[LC_CTYPE],
//                 locale_len[LC_CTYPE], locinfo, LC_CTYPE)) {
//         goto fail;
//     }
//
//     if(!category_needs_update(LC_CTYPE, old_locinfo,
//                 locale_sname[LC_CTYPE], cp[LC_CTYPE])) {
//         copy_threadlocinfo_category(locinfo, old_locinfo, LC_CTYPE);
//         locinfo->lc_codepage = old_locinfo->lc_codepage;
//         locinfo->lc_clike = old_locinfo->lc_clike;
//         locinfo->mb_cur_max = old_locinfo->mb_cur_max;
//         locinfo->ctype1 = old_locinfo->ctype1;
//         locinfo->ctype1_refcount = old_locinfo->ctype1_refcount;
//         locinfo->pctype = old_locinfo->pctype;
//         locinfo->pclmap = old_locinfo->pclmap;
//         locinfo->pcumap = old_locinfo->pcumap;
//         if(locinfo->ctype1_refcount)
//             InterlockedIncrement((LONG *)locinfo->ctype1_refcount);
//     } else if(locale_sname[LC_CTYPE]) {
//         CPINFO cp_info;
//         int j;
//
//         if(!update_threadlocinfo_category(locale_sname[LC_CTYPE],
//                     cp[LC_CTYPE], locinfo, LC_CTYPE)) {
//             goto fail;
//         }
//
//         locinfo->lc_codepage = locinfo->lc_id[LC_CTYPE].wCodePage;
//         locinfo->lc_clike = 1;
//         if(!GetCPInfo(locinfo->lc_codepage, &cp_info)) {
//             goto fail;
//         }
//         locinfo->mb_cur_max = cp_info.MaxCharSize;
//
//         locinfo->ctype1_refcount = malloc(sizeof(int));
//         if(!locinfo->ctype1_refcount) {
//             goto fail;
//         }
//         *locinfo->ctype1_refcount = 1;
//
//         locinfo->ctype1 = malloc(257 * sizeof(*locinfo->ctype1));
//         locinfo->pclmap = malloc(256 * sizeof(*locinfo->pclmap));
//         locinfo->pcumap = malloc(256 * sizeof(*locinfo->pcumap));
//         if(!locinfo->ctype1 || !locinfo->pclmap || !locinfo->pcumap) {
//             goto fail;
//         }
//
//         locinfo->ctype1[0] = 0;
//         locinfo->pctype = locinfo->ctype1+1;
//
//         buf[1] = buf[2] = '\0';
//         for(i=1; i<257; i++) {
//             buf[0] = i-1;
//
//             MultiByteToWideChar(locinfo->lc_codepage, 0, buf, 1, wbuf, 1);
//             /* builtin GetStringType doesn't set output to 0 on invalid input */
//             locinfo->ctype1[i] = 0;
//             GetStringTypeW(CT_CTYPE1, wbuf, 1, &locinfo->ctype1[i]);
//         }
//
//         for(i=0; cp_info.LeadByte[i+1]!=0; i+=2)
//             for(j=cp_info.LeadByte[i]; j<=cp_info.LeadByte[i+1]; j++)
//                 locinfo->ctype1[j+1] |= _LEADBYTE;
//
//         for(i=0; i<256; i++) {
//             if(locinfo->pctype[i] & _LEADBYTE)
//                 buf[i] = ' ';
//             else
//                 buf[i] = i;
//         }
//
//         MultiByteToWideChar(locinfo->lc_codepage, 0, buf, 256, wbuf, 256);
//         LCMapStringW(LOCALE_INVARIANT, LCMAP_LOWERCASE, wbuf, 256, map_buf, 256);
//         WideCharToMultiByte(locinfo->lc_codepage, 0, map_buf, 256, (char *)locinfo->pclmap, 256, NULL, NULL);
//         LCMapStringW(LOCALE_INVARIANT, LCMAP_UPPERCASE, wbuf, 256, map_buf, 256);
//         WideCharToMultiByte(locinfo->lc_codepage, 0, map_buf, 256, (char *)locinfo->pcumap, 256, NULL, NULL);
//     } else {
//         locinfo->lc_clike = 1;
//         locinfo->mb_cur_max = 1;
//         locinfo->pctype = MSVCRT__ctype+1;
//         locinfo->pclmap = cloc_clmap;
//         locinfo->pcumap = cloc_cumap;
//         if(!init_category_name("C", 1, locinfo, LC_CTYPE)) {
//             goto fail;
//         }
//     }
//
//     if(!category_needs_update(LC_MONETARY, old_locinfo,
//                 locale_sname[LC_MONETARY], cp[LC_MONETARY]) &&
//             !category_needs_update(LC_NUMERIC, old_locinfo,
//                 locale_sname[LC_NUMERIC], cp[LC_NUMERIC])) {
//         locinfo->lconv = old_locinfo->lconv;
//         locinfo->lconv_intl_refcount = old_locinfo->lconv_intl_refcount;
//         if(locinfo->lconv_intl_refcount)
//             InterlockedIncrement((LONG *)locinfo->lconv_intl_refcount);
//     } else if(locale_sname[LC_MONETARY] || locale_sname[LC_NUMERIC]) {
//         locinfo->lconv = malloc(sizeof(struct lconv));
//         locinfo->lconv_intl_refcount = malloc(sizeof(int));
//         if(!locinfo->lconv || !locinfo->lconv_intl_refcount) {
//             free(locinfo->lconv);
//             free(locinfo->lconv_intl_refcount);
//             locinfo->lconv = NULL;
//             locinfo->lconv_intl_refcount = NULL;
//             goto fail;
//         }
//         memset(locinfo->lconv, 0, sizeof(struct lconv));
//         *locinfo->lconv_intl_refcount = 1;
//     } else {
//         locinfo->lconv = &cloc_lconv;
//     }
//
//     if(locale_name[LC_MONETARY] &&
//             !init_category_name(locale_name[LC_MONETARY],
//                 locale_len[LC_MONETARY], locinfo, LC_MONETARY)) {
//         goto fail;
//     }
//
//     if(!category_needs_update(LC_MONETARY, old_locinfo,
//                 locale_sname[LC_MONETARY], cp[LC_MONETARY])) {
//         copy_threadlocinfo_category(locinfo, old_locinfo, LC_MONETARY);
//         locinfo->lconv_mon_refcount = old_locinfo->lconv_mon_refcount;
//         if(locinfo->lconv_mon_refcount)
//             InterlockedIncrement((LONG *)locinfo->lconv_mon_refcount);
//         if(locinfo->lconv != &cloc_lconv && locinfo->lconv != old_locinfo->lconv) {
//             locinfo->lconv->int_curr_symbol = old_locinfo->lconv->int_curr_symbol;
//             locinfo->lconv->currency_symbol = old_locinfo->lconv->currency_symbol;
//             locinfo->lconv->mon_decimal_point = old_locinfo->lconv->mon_decimal_point;
//             locinfo->lconv->mon_thousands_sep = old_locinfo->lconv->mon_thousands_sep;
//             locinfo->lconv->mon_grouping = old_locinfo->lconv->mon_grouping;
//             locinfo->lconv->positive_sign = old_locinfo->lconv->positive_sign;
//             locinfo->lconv->negative_sign = old_locinfo->lconv->negative_sign;
//             locinfo->lconv->int_frac_digits = old_locinfo->lconv->int_frac_digits;
//             locinfo->lconv->frac_digits = old_locinfo->lconv->frac_digits;
//             locinfo->lconv->p_cs_precedes = old_locinfo->lconv->p_cs_precedes;
//             locinfo->lconv->p_sep_by_space = old_locinfo->lconv->p_sep_by_space;
//             locinfo->lconv->n_cs_precedes = old_locinfo->lconv->n_cs_precedes;
//             locinfo->lconv->n_sep_by_space = old_locinfo->lconv->n_sep_by_space;
//             locinfo->lconv->p_sign_posn = old_locinfo->lconv->p_sign_posn;
//             locinfo->lconv->n_sign_posn = old_locinfo->lconv->n_sign_posn;
//         }
//     } else if(locale_sname[LC_MONETARY]) {
//         if(!update_threadlocinfo_category(locale_sname[LC_MONETARY],
//                     cp[LC_MONETARY], locinfo, LC_MONETARY)) {
//             goto fail;
//         }
//
//         locinfo->lconv_mon_refcount = malloc(sizeof(int));
//         if(!locinfo->lconv_mon_refcount) {
//             goto fail;
//         }
//
//         *locinfo->lconv_mon_refcount = 1;
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SINTLSYMBOL
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->int_curr_symbol = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->int_curr_symbol, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SCURRENCY
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->currency_symbol = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->currency_symbol, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SMONDECIMALSEP
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->mon_decimal_point = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->mon_decimal_point, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SMONTHOUSANDSEP
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->mon_thousands_sep = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->mon_thousands_sep, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SMONGROUPING
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         WideCharToMultiByte(CP_ACP, 0, wbuf, -1, buf, 256, NULL, NULL);
//         if(i>1)
//             i = i/2 + (buf[i-2]=='0'?0:1);
//         if(i && (locinfo->lconv->mon_grouping = malloc(i))) {
//             for(i=0; buf[i+1]==';'; i+=2)
//                 locinfo->lconv->mon_grouping[i/2] = buf[i]-'0';
//             locinfo->lconv->mon_grouping[i/2] = buf[i]-'0';
//             if(buf[i] != '0')
//                 locinfo->lconv->mon_grouping[i/2+1] = 127;
//         } else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SPOSITIVESIGN
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->positive_sign = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->positive_sign, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_SNEGATIVESIGN
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->negative_sign = malloc(i)))
//             WideCharToMultiByte(cp[LC_MONETARY], 0, wbuf, -1, locinfo->lconv->negative_sign, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_IINTLCURRDIGITS
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->int_frac_digits = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_ICURRDIGITS
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->frac_digits = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_IPOSSYMPRECEDES
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->p_cs_precedes = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_IPOSSEPBYSPACE
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->p_sep_by_space = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_INEGSYMPRECEDES
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->n_cs_precedes = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_INEGSEPBYSPACE
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->n_sep_by_space = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_IPOSSIGNPOSN
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->p_sign_posn = val;
//         else {
//             goto fail;
//         }
//
//         if(GetLocaleInfoEx(locale_sname[LC_MONETARY], LOCALE_INEGSIGNPOSN
//                           |LOCALE_NOUSEROVERRIDE|LOCALE_RETURN_NUMBER, (WCHAR *)&val, 2))
//             locinfo->lconv->n_sign_posn = val;
//         else {
//             goto fail;
//         }
//
//     } else {
//         if (locinfo->lconv != &cloc_lconv) {
//             locinfo->lconv->int_curr_symbol = cloc_lconv.int_curr_symbol;
//             locinfo->lconv->currency_symbol = cloc_lconv.currency_symbol;
//             locinfo->lconv->mon_decimal_point = cloc_lconv.mon_decimal_point;
//             locinfo->lconv->mon_thousands_sep = cloc_lconv.mon_thousands_sep;
//             locinfo->lconv->mon_grouping = cloc_lconv.mon_grouping;
//             locinfo->lconv->positive_sign = cloc_lconv.positive_sign;
//             locinfo->lconv->negative_sign = cloc_lconv.negative_sign;
//             locinfo->lconv->int_frac_digits = cloc_lconv.int_frac_digits;
//             locinfo->lconv->frac_digits = cloc_lconv.frac_digits;
//             locinfo->lconv->p_cs_precedes = cloc_lconv.p_cs_precedes;
//             locinfo->lconv->p_sep_by_space = cloc_lconv.p_sep_by_space;
//             locinfo->lconv->n_cs_precedes = cloc_lconv.n_cs_precedes;
//             locinfo->lconv->n_sep_by_space = cloc_lconv.n_sep_by_space;
//             locinfo->lconv->p_sign_posn = cloc_lconv.p_sign_posn;
//             locinfo->lconv->n_sign_posn = cloc_lconv.n_sign_posn;
//
//         }
//
//         if(!init_category_name("C", 1, locinfo, LC_MONETARY)) {
//             goto fail;
//         }
//     }
//
//     if(locale_name[LC_NUMERIC] &&
//             !init_category_name(locale_name[LC_NUMERIC],
//                 locale_len[LC_NUMERIC], locinfo, LC_NUMERIC)) {
//         goto fail;
//     }
//
//     if(!category_needs_update(LC_NUMERIC, old_locinfo,
//                 locale_sname[LC_NUMERIC], cp[LC_NUMERIC])) {
//         copy_threadlocinfo_category(locinfo, old_locinfo, LC_NUMERIC);
//         locinfo->lconv_num_refcount = old_locinfo->lconv_num_refcount;
//         if(locinfo->lconv_num_refcount)
//             InterlockedIncrement((LONG *)locinfo->lconv_num_refcount);
//         if(locinfo->lconv != &cloc_lconv && locinfo->lconv != old_locinfo->lconv) {
//             locinfo->lconv->decimal_point = old_locinfo->lconv->decimal_point;
//             locinfo->lconv->thousands_sep = old_locinfo->lconv->thousands_sep;
//             locinfo->lconv->grouping = old_locinfo->lconv->grouping;
//         }
//     } else if(locale_sname[LC_NUMERIC]) {
//         if(!update_threadlocinfo_category(locale_sname[LC_NUMERIC],
//                     cp[LC_NUMERIC], locinfo, LC_NUMERIC)) {
//             goto fail;
//         }
//
//         locinfo->lconv_num_refcount = malloc(sizeof(int));
//         if(!locinfo->lconv_num_refcount) {
//             goto fail;
//         }
//
//         *locinfo->lconv_num_refcount = 1;
//
//         i = GetLocaleInfoEx(locale_sname[LC_NUMERIC], LOCALE_SDECIMAL
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_NUMERIC], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->decimal_point = malloc(i)))
//             WideCharToMultiByte(cp[LC_NUMERIC], 0, wbuf, -1, locinfo->lconv->decimal_point, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_NUMERIC], LOCALE_STHOUSAND
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         i = WideCharToMultiByte(cp[LC_NUMERIC], 0, wbuf, -1, NULL, 0, NULL, NULL);
//         if(i && (locinfo->lconv->thousands_sep = malloc(i)))
//             WideCharToMultiByte(cp[LC_NUMERIC], 0, wbuf, -1, locinfo->lconv->thousands_sep, i, NULL, NULL);
//         else {
//             goto fail;
//         }
//
//         i = GetLocaleInfoEx(locale_sname[LC_NUMERIC], LOCALE_SGROUPING
//                 |LOCALE_NOUSEROVERRIDE, wbuf, 256);
//         WideCharToMultiByte(cp[LC_NUMERIC], 0, wbuf, -1, buf, 256, NULL, NULL);
//         if(i>1)
//             i = i/2 + (buf[i-2]=='0'?0:1);
//         if(i && (locinfo->lconv->grouping = malloc(i))) {
//             for(i=0; buf[i+1]==';'; i+=2)
//                 locinfo->lconv->grouping[i/2] = buf[i]-'0';
//             locinfo->lconv->grouping[i/2] = buf[i]-'0';
//             if(buf[i] != '0')
//                 locinfo->lconv->grouping[i/2+1] = 127;
//         } else {
//             goto fail;
//         }
//
//     } else {
//         if (locinfo->lconv != &cloc_lconv) {
//             locinfo->lconv->decimal_point = cloc_lconv.decimal_point;
//             locinfo->lconv->thousands_sep = cloc_lconv.thousands_sep;
//             locinfo->lconv->grouping = cloc_lconv.grouping;
//
//         }
//
//         if (!init_category_name("C", 1, locinfo, LC_NUMERIC)) {
//             goto fail;
//         }
//     }
//
//     if(locale_name[LC_TIME] &&
//             !init_category_name(locale_name[LC_TIME],
//                 locale_len[LC_TIME], locinfo, LC_TIME)) {
//         goto fail;
//     }
//
//     if(!category_needs_update(LC_TIME, old_locinfo,
//                 locale_sname[LC_TIME], cp[LC_TIME])) {
//         copy_threadlocinfo_category(locinfo, old_locinfo, LC_TIME);
//         locinfo->lc_time_curr = old_locinfo->lc_time_curr;
//         InterlockedIncrement(&locinfo->lc_time_curr->refcount);
//     } else if(locale_sname[LC_TIME]) {
//         if(!update_threadlocinfo_category(locale_sname[LC_TIME],
//                     cp[LC_TIME], locinfo, LC_TIME)) {
//             goto fail;
//         }
//
//         locinfo->lc_time_curr = create_time_data(locale_sname[LC_TIME]);
//         if(!locinfo->lc_time_curr) {
//             goto fail;
//         }
//     } else {
//         if(!init_category_name("C", 1, locinfo, LC_TIME)) {
//             goto fail;
//         }
//         locinfo->lc_time_curr = &cloc_time_data;
//         InterlockedIncrement(&locinfo->lc_time_curr->refcount);
//     }
//
//     for (i = 0; i < LC_MAX; i++)
//         free(locale_sname[i]);
//
//     return locinfo;
//
// fail:
//     free_locinfo(locinfo);
//
//     for (i = 0; i < LC_MAX; i++)
//         free(locale_sname[i]);
//
//     return NULL;
// }
//
// /*********************************************************************
//  *      _create_locale (MSVCRT.@)
//  */
// _locale_t CDECL _create_locale(int category, const char *locale)
// {
//     _locale_t loc;
//
//     loc = malloc(sizeof(_locale_tstruct));
//     if(!loc)
//         return NULL;
//
//     loc->locinfo = create_locinfo(category, locale, NULL);
//     if(!loc->locinfo) {
//         free(loc);
//         return NULL;
//     }
//
//     loc->mbcinfo = create_mbcinfo(loc->locinfo->lc_id[LC_CTYPE].wCodePage,
//             loc->locinfo->lc_handle[LC_CTYPE], NULL);
//     if(!loc->mbcinfo) {
//         free_locinfo(loc->locinfo);
//         free(loc);
//         return NULL;
//     }
//     return loc;
// }
//
//
// /*********************************************************************
//  *             setlocale (MSVCRT.@)
//  */
// char*  setlocale(int category, const char* locale)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     pthreadlocinfo locinfo = get_locinfo(), newlocinfo;
//     int locale_flags;
//
//     if(category<LC_MIN || category>LC_MAX)
//         return NULL;
//
//     if(!locale) {
//         if(category == LC_ALL)
//             return construct_lc_all(locinfo);
//
//         return locinfo->lc_category[category].locale;
//     }
//
//     /* Make sure that locinfo is not updated by e.g. stricmp function */
//     locale_flags = data->locale_flags;
//     data->locale_flags |= LOCALE_THREAD;
//     newlocinfo = create_locinfo(category, locale, locinfo);
//     data->locale_flags = locale_flags;
//     if(!newlocinfo) {
//         WARN("%d %s failed\n", category, locale);
//         return NULL;
//     }
//
//     if(locale[0] != 'C' || locale[1] != '\0')
//         initial_locale = FALSE;
//
//     if(data->locale_flags & LOCALE_THREAD)
//     {
//         if(data->locale_flags & LOCALE_FREE)
//             free_locinfo(data->locinfo);
//         data->locinfo = newlocinfo;
//     }
//     else
//     {
//         int i;
//
//         _lock_locales();
//         free_locinfo(MSVCRT_locale->locinfo);
//         MSVCRT_locale->locinfo = newlocinfo;
//
//         MSVCRT___lc_codepage = newlocinfo->lc_codepage;
//         MSVCRT___lc_collate_cp = newlocinfo->lc_collate_cp;
//         MSVCRT___mb_cur_max = newlocinfo->mb_cur_max;
//         MSVCRT__pctype = newlocinfo->pctype;
//         for(i=LC_MIN; i<=LC_MAX; i++)
//             MSVCRT___lc_handle[i] = MSVCRT_locale->locinfo->lc_handle[i];
//         _unlock_locales();
//         update_thread_locale(data);
//     }
//
//     if(category == LC_ALL)
//         return construct_lc_all(data->locinfo);
//
//     return data->locinfo->lc_category[category].locale;
// }
//
// /*********************************************************************
//  *		_wsetlocale (MSVCRT.@)
//  */
// wchar_t*  _wsetlocale(int category, const wchar_t* wlocale)
// {
//     static wchar_t current_lc_all[MAX_LOCALE_LENGTH];
//
//     char *locale = NULL;
//     const char *ret;
//     size_t len;
//
//     if(wlocale) {
//         len = wcstombs(NULL, wlocale, 0);
//         if(len == -1)
//             return NULL;
//
//         locale = malloc(++len);
//         if(!locale)
//             return NULL;
//
//         wcstombs(locale, wlocale, len);
//     }
//
//     _lock_locales();
//     ret = setlocale(category, locale);
//     free(locale);
//
//     if(ret && mbstowcs(current_lc_all, ret, MAX_LOCALE_LENGTH)==-1)
//         ret = NULL;
//
//     _unlock_locales();
//     return ret ? current_lc_all : NULL;
// }
//
//
// BOOL msvcrt_init_locale(void)
// {
//     int i;
//
//     _lock_locales();
//     MSVCRT_locale = _create_locale(0, "C");
//     _unlock_locales();
//     if(!MSVCRT_locale)
//         return FALSE;
//
//     MSVCRT___lc_codepage = MSVCRT_locale->locinfo->lc_codepage;
//     MSVCRT___lc_collate_cp = MSVCRT_locale->locinfo->lc_collate_cp;
//     MSVCRT___mb_cur_max = MSVCRT_locale->locinfo->mb_cur_max;
//     MSVCRT__pctype = MSVCRT_locale->locinfo->pctype;
//     for(i=LC_MIN; i<=LC_MAX; i++)
//         MSVCRT___lc_handle[i] = MSVCRT_locale->locinfo->lc_handle[i];
//     _setmbcp(_MB_CP_ANSI);
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
