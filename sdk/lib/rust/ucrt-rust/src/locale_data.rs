//! The locale subsystem's DATA and its leaf accessors. Replaces, in full:
//! `sdk/lib/ucrt/locale/ctype.cpp`, `nlsdata.cpp`, `localeconv.cpp`,
//! `glstatus.cpp`, `lconv_unsigned_char_initialization.cpp` and
//! `lcidtoname_downlevel.cpp`.
//!
//! Almost everything here is a static that ~160 other ucrt objects reach by C
//! name, and several point into each other (`__acrt_initial_locale_data` ->
//! `__newctype + 128`, `__acrt_lconv_c`, `__lc_time_c`, ...). So the rule for
//! every item is that the exported symbol's MEMORY is exactly the C object's --
//! same bytes, same relocations, same writability -- and that is checked, not
//! assumed: `tools/compare_coff_data.py` diffs each one against Microsoft's
//! compiled object (bytes, plus every relocation's target and addend).
//!
//! `_wctype` and `__newctype` take their values from `crate::ctype_table`,
//! which was generated from public Unicode data and graded against ctype.cpp
//! while it still exists; the two case maps are computed below. The LCID
//! tables are the NLS facts lcidtoname_downlevel.cpp lists, row for row.
//!
//! ## Details that are silent if wrong
//!
//! * **`_wctype` has 258 entries, not 257**: slot 0 is EOF, then U+0000 through
//!   U+0100 inclusive. `_pwctype` points ONE in; `_pctype` points 128 into
//!   `__newctype` (so `_pctype[-1]` is EOF's zero, and `_pctype[-128..-2]` are
//!   real zeros, not out of bounds).
//! * **`__newclmap`/`__newcumap` map the "negative" slots to 0x80..0xFF**, not
//!   to zero: `pclmap[(signed char)c]` must return the byte unchanged for every
//!   char, and only `A-Z` / `a-z` move.
//! * **`__acrt_lconv_static_*` are the SENTINELS the lconv freeing code compares
//!   against** ("pointer to original static to avoid freeing"): a locale's
//!   lconv member that still points at one of them is never passed to free.
//!   So the C lconv must point at exactly these four objects, not at equal
//!   literals elsewhere.
//! * **The C lconv's `char` members are `CHAR_MAX` = 127**, because the tree
//!   builds with a signed `char`; `__initialize_lconv_for_unsigned_char`
//!   rewrites them to `(char)255` for a `-J` client, and nothing inside
//!   ucrtbase calls it (see that function).
//! * **`__globallocalestatus` is `~_GLOBAL_LOCALE_BIT` = -2**, i.e. every bit
//!   set except bit 0 -- not 0 and not 1.
//! * **`__acrt_current_locale_data` starts NULL** (it is `.bss`; the
//!   `dual_state_global` is one zero-initialised pointer with state isolation
//!   compiled out) and `__mb_cur_max` starts at 1.
//! * **`__lc_time_c` is const and its `refcount` is 0, `ww_caltype` 1**; its
//!   wide locale name is `L"en-US"` even though this is the "C" locale.
//! * **The name-to-LCID binary search cannot find `kok` or `kok-IN`.**
//!   `LocaleNameToIndexTable` lists `kok`, `kok-in` BEFORE `ko-kr`, but under
//!   `__ascii_wcsnicmp` `'-' < 'k'`, so the table is not sorted there and the
//!   search walks past them: both answer 0 from Microsoft's C. Kept, because
//!   this is graded against that C; every other name is reachable.
//! * **The LCID binary search subtracts UNSIGNED and compares SIGNED**:
//!   `int testIndex = lcid - table.lcid`, so an LCID more than 2^31 above an
//!   entry reads as "less than" it. Harmless for real LCIDs; kept.
//! * **`__acrt_DownlevelLCIDToLocaleName` returns the length INCLUDING the
//!   terminator**, answers a size query when `cch == 0` (any buffer, even
//!   NULL), and returns 0 -- writing nothing -- when the buffer is too small
//!   rather than truncating. 0, `LOCALE_USER_DEFAULT` and
//!   `LOCALE_SYSTEM_DEFAULT` are refused before the table is consulted.

use crate::locale::{
    CrtLocaleData, CrtLocaleDataPublic, CrtLocalePointers, CrtLocaleRefcount, LcTimeData, Lconv,
    GLOBAL_LOCALE_BIT, LOCALE_NAME_MAX_LENGTH,
};
use crate::locale_abi::__acrt_initial_multibyte_data;
use crate::locale_update::{__acrt_getptd, __acrt_update_locale_info};
use core::ffi::{c_char, c_int, c_uint, c_ushort, c_void};
use core::ptr::{null, null_mut};

extern "C" {
    /// string/wcsnicmp.cpp -- ASCII-only case-insensitive compare, returning the
    /// difference of the first differing lower-cased units.
    fn __ascii_wcsnicmp(lhs: *const u16, rhs: *const u16, count: usize) -> c_int;
    fn wcsnlen(s: *const u16, max: usize) -> usize;
    fn wcscpy_s(dest: *mut u16, size: usize, src: *const u16) -> c_int;
    /// corecrt.h: `__declspec(noreturn) void _invoke_watson(...)`.
    fn _invoke_watson(
        expression: *const u16,
        function_name: *const u16,
        file_name: *const u16,
        line_number: c_uint,
        reserved: usize,
    ) -> !;
}

/// `CP_ACP`
const CP_ACP: c_uint = 0;
/// `CHAR_MAX` for the signed `char` this tree builds with (named apart from the
/// repo's other `CHAR_MAX`s so the constant lint never has to reconcile a
/// signedness question).
const LCONV_SIGNED_CHAR_MAX: c_char = 127;
/// `(char)UCHAR_MAX`
const LCONV_UNSIGNED_CHAR_MAX: c_char = 255u8 as c_char;
/// winnt.h `LOCALE_USER_DEFAULT` / `LOCALE_SYSTEM_DEFAULT`.
const LOCALE_USER_DEFAULT: u32 = 0x0400;
const LOCALE_SYSTEM_DEFAULT: u32 = 0x0800;

/// A `const` C object holding raw pointers. Rust will not put a pointer-bearing
/// value in an immutable `static` without `Sync`; the wrapper is transparent, so
/// the symbol's memory is exactly the inner C object's.
#[repr(transparent)]
pub struct SyncConst<T>(pub T);
// SAFETY: every `SyncConst` below is immutable data pointing at immutable data.
unsafe impl<T> Sync for SyncConst<T> {}

/// Widen an ASCII literal plus its terminator to UTF-16 at compile time. A
/// `const fn` evaluated by the compiler, never a runtime copy loop; a non-ASCII
/// byte is a build error rather than a mis-encoded name.
const fn ascii_to_wide<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    assert!(b.len() + 1 == N);
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        assert!(b[i] < 0x80);
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

/// `L"..."` as a `wchar_t*`: a NUL-terminated UTF-16 static of its own.
macro_rules! w {
    ($s:literal) => {{
        static W: [u16; $s.len() + 1] = ascii_to_wide::<{ $s.len() + 1 }>($s);
        (&raw const W).cast::<u16>().cast_mut()
    }};
}

/// `"..."` as a `char*`.
macro_rules! n {
    ($s:literal) => {
        $s.as_ptr().cast_mut()
    };
}

// ════════════════════════════════════════════════════════════════════════════
// ctype.cpp
// ════════════════════════════════════════════════════════════════════════════

/// The wide classification table: EOF, then U+0000..=U+0100. Exported as data
/// (`@ extern _wctype` in ucrtbase.spec).
#[no_mangle]
pub static _wctype: [u16; 258] = crate::ctype_table::UCRT_WCTYPE;

/// The "C" locale's narrow classification table; index `128 + c` for
/// `c` in -128..=255.
#[no_mangle]
pub static __newctype: [u16; 384] = crate::ctype_table::UCRT_NEWCTYPE;

/// Slot `i` of a "C" locale case map: the byte itself for the 128 negative-char
/// slots and for 0x00..=0xFF, with only the one ASCII case range moved.
const fn c_case_map(upper: bool) -> [u8; 384] {
    let mut out = [0u8; 384];
    let mut i = 0;
    while i < 384 {
        // (i - 128) as an unsigned char: slots 0..128 are 0x80..=0xFF.
        let c = (i as u8).wrapping_sub(128);
        out[i] = if i < 128 {
            c
        } else if upper && c.is_ascii_lowercase() {
            c - (b'a' - b'A')
        } else if !upper && c.is_ascii_uppercase() {
            c + (b'a' - b'A')
        } else {
            c
        };
        i += 1;
    }
    out
}

/// The "C" locale's to-lower map, indexed like `__newctype`.
#[no_mangle]
pub static __newclmap: [u8; 384] = c_case_map(false);

/// The "C" locale's to-upper map, indexed like `__newctype`.
#[no_mangle]
pub static __newcumap: [u8; 384] = c_case_map(true);

/// Pointer to the narrow table for `char`s. A mutable pointer to const data,
/// as in the C (`unsigned short const* _pctype`).
#[no_mangle]
pub static mut _pctype: *const u16 = (&raw const __newctype).cast::<u16>().wrapping_add(128);

/// Pointer to the wide table for `wchar_t`s.
#[no_mangle]
pub static mut _pwctype: *const u16 = (&raw const _wctype).cast::<u16>().wrapping_add(1);

/// Exported. Reads the `_pwctype` VARIABLE, so a client that has stored to it
/// sees its own value back.
#[no_mangle]
pub unsafe extern "C" fn __pwctype_func() -> *const c_ushort {
    // SAFETY: a plain read of a pointer-sized global, as the C does.
    unsafe { _pwctype }
}

/// Exported. The current thread's locale's narrow table, resynchronising the
/// thread with the global locale first. A leaf, so no `_LocaleUpdate`.
#[no_mangle]
pub unsafe extern "C" fn __pctype_func() -> *const c_ushort {
    // SAFETY: __acrt_getptd never returns NULL (it terminates instead), and
    // __acrt_update_locale_info leaves a valid locale pointer in `locale_info`.
    unsafe {
        let ptd = __acrt_getptd();
        let mut locale_info = (*ptd).locale_info;
        __acrt_update_locale_info(ptd, &mut locale_info);
        (*locale_info).public.locale_pctype
    }
}

// ════════════════════════════════════════════════════════════════════════════
// nlsdata.cpp
// ════════════════════════════════════════════════════════════════════════════

/// Not exported by ucrtbase (it exports `___mb_cur_max_func`); internal readers
/// still use it by name.
#[no_mangle]
pub static mut __mb_cur_max: c_int = 1;

/// The current thread's `mb_cur_max`, by address.
#[no_mangle]
pub unsafe extern "C" fn __p___mb_cur_max() -> *mut c_int {
    // SAFETY: as __pctype_func.
    unsafe {
        let ptd = __acrt_getptd();
        let mut locale_info = (*ptd).locale_info;
        __acrt_update_locale_info(ptd, &mut locale_info);
        &raw mut (*locale_info).public.locale_mb_cur_max
    }
}

/// `L"C"` -- mutable in the C (`wchar_t[]`), and the locale-name pointer every
/// category of the initial locale shares.
#[no_mangle]
pub static mut __acrt_wide_c_locale_string: [u16; 2] = [b'C' as u16, 0];

/// The "C" locale's LC_TIME data.
#[no_mangle]
pub static __lc_time_c: SyncConst<LcTimeData> = SyncConst(LcTimeData {
    wday_abbr: [
        n!(c"Sun"),
        n!(c"Mon"),
        n!(c"Tue"),
        n!(c"Wed"),
        n!(c"Thu"),
        n!(c"Fri"),
        n!(c"Sat"),
    ],
    wday: [
        n!(c"Sunday"),
        n!(c"Monday"),
        n!(c"Tuesday"),
        n!(c"Wednesday"),
        n!(c"Thursday"),
        n!(c"Friday"),
        n!(c"Saturday"),
    ],
    month_abbr: [
        n!(c"Jan"),
        n!(c"Feb"),
        n!(c"Mar"),
        n!(c"Apr"),
        n!(c"May"),
        n!(c"Jun"),
        n!(c"Jul"),
        n!(c"Aug"),
        n!(c"Sep"),
        n!(c"Oct"),
        n!(c"Nov"),
        n!(c"Dec"),
    ],
    month: [
        n!(c"January"),
        n!(c"February"),
        n!(c"March"),
        n!(c"April"),
        n!(c"May"),
        n!(c"June"),
        n!(c"July"),
        n!(c"August"),
        n!(c"September"),
        n!(c"October"),
        n!(c"November"),
        n!(c"December"),
    ],
    ampm: [n!(c"AM"), n!(c"PM")],
    ww_sdatefmt: n!(c"MM/dd/yy"),
    ww_ldatefmt: n!(c"dddd, MMMM dd, yyyy"),
    ww_timefmt: n!(c"HH:mm:ss"),
    ww_caltype: 1,
    refcount: 0,
    w_wday_abbr: [
        w!("Sun"),
        w!("Mon"),
        w!("Tue"),
        w!("Wed"),
        w!("Thu"),
        w!("Fri"),
        w!("Sat"),
    ],
    w_wday: [
        w!("Sunday"),
        w!("Monday"),
        w!("Tuesday"),
        w!("Wednesday"),
        w!("Thursday"),
        w!("Friday"),
        w!("Saturday"),
    ],
    w_month_abbr: [
        w!("Jan"),
        w!("Feb"),
        w!("Mar"),
        w!("Apr"),
        w!("May"),
        w!("Jun"),
        w!("Jul"),
        w!("Aug"),
        w!("Sep"),
        w!("Oct"),
        w!("Nov"),
        w!("Dec"),
    ],
    w_month: [
        w!("January"),
        w!("February"),
        w!("March"),
        w!("April"),
        w!("May"),
        w!("June"),
        w!("July"),
        w!("August"),
        w!("September"),
        w!("October"),
        w!("November"),
        w!("December"),
    ],
    w_ampm: [w!("AM"), w!("PM")],
    w_ww_sdatefmt: w!("MM/dd/yy"),
    w_ww_ldatefmt: w!("dddd, MMMM dd, yyyy"),
    w_ww_timefmt: w!("HH:mm:ss"),
    w_ww_locale_name: w!("en-US"),
});

/// One `lc_category[]` row of the initial locale: only the wide name is set.
const fn c_category(wlocale: *mut u16) -> CrtLocaleRefcount {
    CrtLocaleRefcount {
        locale: null_mut(),
        wlocale,
        refcount: null_mut(),
        wrefcount: null_mut(),
    }
}

/// The initial locale: the "C" locale, in use until the first `setlocale`.
/// Mutable (its refcount and fields are updated like any other locale's).
#[no_mangle]
pub static mut __acrt_initial_locale_data: CrtLocaleData = {
    let c_name = (&raw mut __acrt_wide_c_locale_string).cast::<u16>();
    CrtLocaleData {
        public: CrtLocaleDataPublic {
            locale_pctype: (&raw const __newctype).cast::<c_ushort>().wrapping_add(128),
            locale_mb_cur_max: 1,
            locale_lc_codepage: CP_ACP,
        },
        refcount: 1,
        lc_collate_cp: CP_ACP,
        lc_time_cp: CP_ACP,
        lc_clike: 1,
        lc_category: [
            CrtLocaleRefcount {
                locale: null_mut(),
                wlocale: null_mut(),
                refcount: null_mut(),
                wrefcount: null_mut(),
            }, // CRT_LC_ALL
            c_category(c_name), // CRT_LC_COLLATE
            c_category(c_name), // CRT_LC_CTYPE
            c_category(c_name), // CRT_LC_MONETARY
            c_category(c_name), // CRT_LC_NUMERIC
            c_category(c_name), // CRT_LC_TIME
        ],
        lconv_intl_refcount: null_mut(),
        lconv_num_refcount: null_mut(),
        lconv_mon_refcount: null_mut(),
        lconv: (&raw mut __acrt_lconv_c).cast::<c_void>(),
        ctype1_refcount: null_mut(),
        ctype1: null_mut(),
        pclmap: (&raw const __newclmap).cast::<u8>().wrapping_add(128),
        pcumap: (&raw const __newcumap).cast::<u8>().wrapping_add(128),
        lc_time_curr: (&raw const __lc_time_c).cast::<c_void>(),
        locale_name: [null_mut(); 6],
    }
};

/// `dual_state_global<__crt_locale_data*>`: one pointer, NULL until the locale
/// is first changed.
#[no_mangle]
pub static mut __acrt_current_locale_data: *mut CrtLocaleData = null_mut();

/// The `_locale_t` of the initial locale.
#[no_mangle]
pub static mut __acrt_initial_locale_pointers: CrtLocalePointers = CrtLocalePointers {
    locinfo: &raw mut __acrt_initial_locale_data,
    mbcinfo: &raw mut __acrt_initial_multibyte_data,
};

// ════════════════════════════════════════════════════════════════════════════
// localeconv.cpp
// ════════════════════════════════════════════════════════════════════════════

/// "Pointer to original static to avoid freeing": the lconv free paths compare
/// member pointers against these four addresses. See the module docs.
#[no_mangle]
pub static mut __acrt_lconv_static_decimal: [c_char; 2] = [b'.' as c_char, 0];
#[no_mangle]
pub static mut __acrt_lconv_static_null: [c_char; 1] = [0];
#[no_mangle]
pub static mut __acrt_lconv_static_W_decimal: [u16; 2] = [b'.' as u16, 0];
#[no_mangle]
pub static mut __acrt_lconv_static_W_null: [u16; 1] = [0];

/// The "C" locale's lconv. Mutable: `__initialize_lconv_for_unsigned_char`
/// rewrites its `char` members.
#[no_mangle]
pub static mut __acrt_lconv_c: Lconv = {
    let decimal = (&raw mut __acrt_lconv_static_decimal).cast::<c_char>();
    let null_s = (&raw mut __acrt_lconv_static_null).cast::<c_char>();
    let w_decimal = (&raw mut __acrt_lconv_static_W_decimal).cast::<u16>();
    let w_null = (&raw mut __acrt_lconv_static_W_null).cast::<u16>();
    Lconv {
        decimal_point: decimal,
        thousands_sep: null_s,
        grouping: null_s,
        int_curr_symbol: null_s,
        currency_symbol: null_s,
        mon_decimal_point: null_s,
        mon_thousands_sep: null_s,
        mon_grouping: null_s,
        positive_sign: null_s,
        negative_sign: null_s,
        int_frac_digits: LCONV_SIGNED_CHAR_MAX,
        frac_digits: LCONV_SIGNED_CHAR_MAX,
        p_cs_precedes: LCONV_SIGNED_CHAR_MAX,
        p_sep_by_space: LCONV_SIGNED_CHAR_MAX,
        n_cs_precedes: LCONV_SIGNED_CHAR_MAX,
        n_sep_by_space: LCONV_SIGNED_CHAR_MAX,
        p_sign_posn: LCONV_SIGNED_CHAR_MAX,
        n_sign_posn: LCONV_SIGNED_CHAR_MAX,
        w_decimal_point: w_decimal,
        w_thousands_sep: w_null,
        w_int_curr_symbol: w_null,
        w_currency_symbol: w_null,
        w_mon_decimal_point: w_null,
        w_mon_thousands_sep: w_null,
        w_positive_sign: w_null,
        w_negative_sign: w_null,
    }
};

/// Pointer to the current lconv structure.
#[no_mangle]
pub static mut __acrt_lconv: *mut Lconv = &raw mut __acrt_lconv_c;

/// Exported. The current thread's locale's lconv (after resynchronising the
/// thread with the global locale) -- `locale_info->lconv`, NOT `__acrt_lconv`.
#[no_mangle]
pub unsafe extern "C" fn localeconv() -> *mut Lconv {
    // SAFETY: as __pctype_func.
    unsafe {
        let ptd = __acrt_getptd();
        let mut locale_info = (*ptd).locale_info;
        __acrt_update_locale_info(ptd, &mut locale_info);
        (*locale_info).lconv.cast::<Lconv>()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// lconv_unsigned_char_initialization.cpp
// ════════════════════════════════════════════════════════════════════════════

/// Exported. Sets the "C" lconv's `char` members to `(char)UCHAR_MAX` for a
/// client compiled with `-J` (unsigned `char`), where `CHAR_MAX` is 255.
///
/// Nothing in ucrtbase calls it and it is in no initializer section here: in
/// Microsoft's CRT, a `-J` TU's `<locale.h>` references
/// `__do_unsigned_char_lconv_initialization` (sdk/include/ucrt/locale.h:67-73),
/// which pulls `lconv_unsigned_char_static.cpp` from the client's STATIC startup
/// library, and THAT object registers a call to this export in the client's own
/// initializer list. Neither the static object nor the linker section live in
/// this tree, so from ucrtbase's side this is just an exported function.
#[no_mangle]
pub unsafe extern "C" fn __initialize_lconv_for_unsigned_char() -> c_int {
    // SAFETY: plain stores into the C lconv, exactly as the C (unsynchronised).
    unsafe {
        let lc = &raw mut __acrt_lconv_c;
        (*lc).int_frac_digits = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).frac_digits = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).p_cs_precedes = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).p_sep_by_space = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).n_cs_precedes = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).n_sep_by_space = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).p_sign_posn = LCONV_UNSIGNED_CHAR_MAX;
        (*lc).n_sign_posn = LCONV_UNSIGNED_CHAR_MAX;
    }
    0
}

// ════════════════════════════════════════════════════════════════════════════
// glstatus.cpp
// ════════════════════════════════════════════════════════════════════════════

/// `~_GLOBAL_LOCALE_BIT`: per-thread locale is ENABLED by default (a client's
/// static `__globallocalestatus` of -1 disables it).
#[no_mangle]
pub static mut __globallocalestatus: c_int = !GLOBAL_LOCALE_BIT;

// ════════════════════════════════════════════════════════════════════════════
// lcidtoname_downlevel.cpp
// ════════════════════════════════════════════════════════════════════════════

/// One row of `LcidToLocaleNameTable` (`{ LCID lcid; wchar_t* localeName; }`).
#[repr(C)]
struct LcidToLocaleName {
    lcid: u32,
    locale_name: *mut u16,
}

/// One row of `LocaleNameToIndexTable` (`{ wchar_t* name; int index; }`).
#[repr(C)]
struct LocaleNameIndex {
    name: *mut u16,
    index: c_int,
}

const fn lcid(lcid: u32, locale_name: *mut u16) -> LcidToLocaleName {
    LcidToLocaleName { lcid, locale_name }
}

const fn idx(name: *mut u16, index: c_int) -> LocaleNameIndex {
    LocaleNameIndex { name, index }
}

// Windows XP's LCID -> locale-name map, sorted by LCID, and the index sorted
// (almost -- see `kok` in the module docs) by name.
static LCID_TO_LOCALE_NAME_TABLE: SyncConst<[LcidToLocaleName; 228]> = SyncConst([
    lcid(0x0001, w!("ar")),
    lcid(0x0002, w!("bg")),
    lcid(0x0003, w!("ca")),
    lcid(0x0004, w!("zh-CHS")),
    lcid(0x0005, w!("cs")),
    lcid(0x0006, w!("da")),
    lcid(0x0007, w!("de")),
    lcid(0x0008, w!("el")),
    lcid(0x0009, w!("en")),
    lcid(0x000A, w!("es")),
    lcid(0x000B, w!("fi")),
    lcid(0x000C, w!("fr")),
    lcid(0x000D, w!("he")),
    lcid(0x000E, w!("hu")),
    lcid(0x000F, w!("is")),
    lcid(0x0010, w!("it")),
    lcid(0x0011, w!("ja")),
    lcid(0x0012, w!("ko")),
    lcid(0x0013, w!("nl")),
    lcid(0x0014, w!("no")),
    lcid(0x0015, w!("pl")),
    lcid(0x0016, w!("pt")),
    lcid(0x0018, w!("ro")),
    lcid(0x0019, w!("ru")),
    lcid(0x001A, w!("hr")),
    lcid(0x001B, w!("sk")),
    lcid(0x001C, w!("sq")),
    lcid(0x001D, w!("sv")),
    lcid(0x001E, w!("th")),
    lcid(0x001F, w!("tr")),
    lcid(0x0020, w!("ur")),
    lcid(0x0021, w!("id")),
    lcid(0x0022, w!("uk")),
    lcid(0x0023, w!("be")),
    lcid(0x0024, w!("sl")),
    lcid(0x0025, w!("et")),
    lcid(0x0026, w!("lv")),
    lcid(0x0027, w!("lt")),
    lcid(0x0029, w!("fa")),
    lcid(0x002A, w!("vi")),
    lcid(0x002B, w!("hy")),
    lcid(0x002C, w!("az")),
    lcid(0x002D, w!("eu")),
    lcid(0x002F, w!("mk")),
    lcid(0x0036, w!("af")),
    lcid(0x0037, w!("ka")),
    lcid(0x0038, w!("fo")),
    lcid(0x0039, w!("hi")),
    lcid(0x003E, w!("ms")),
    lcid(0x003F, w!("kk")),
    lcid(0x0040, w!("ky")),
    lcid(0x0041, w!("sw")),
    lcid(0x0043, w!("uz")),
    lcid(0x0044, w!("tt")),
    lcid(0x0046, w!("pa")),
    lcid(0x0047, w!("gu")),
    lcid(0x0049, w!("ta")),
    lcid(0x004A, w!("te")),
    lcid(0x004B, w!("kn")),
    lcid(0x004E, w!("mr")),
    lcid(0x004F, w!("sa")),
    lcid(0x0050, w!("mn")),
    lcid(0x0056, w!("gl")),
    lcid(0x0057, w!("kok")),
    lcid(0x005A, w!("syr")),
    lcid(0x0065, w!("div")),
    lcid(0x007F, w!("")),
    lcid(0x0401, w!("ar-SA")),
    lcid(0x0402, w!("bg-BG")),
    lcid(0x0403, w!("ca-ES")),
    lcid(0x0404, w!("zh-TW")),
    lcid(0x0405, w!("cs-CZ")),
    lcid(0x0406, w!("da-DK")),
    lcid(0x0407, w!("de-DE")),
    lcid(0x0408, w!("el-GR")),
    lcid(0x0409, w!("en-US")),
    lcid(0x040B, w!("fi-FI")),
    lcid(0x040C, w!("fr-FR")),
    lcid(0x040D, w!("he-IL")),
    lcid(0x040E, w!("hu-HU")),
    lcid(0x040F, w!("is-IS")),
    lcid(0x0410, w!("it-IT")),
    lcid(0x0411, w!("ja-JP")),
    lcid(0x0412, w!("ko-KR")),
    lcid(0x0413, w!("nl-NL")),
    lcid(0x0414, w!("nb-NO")),
    lcid(0x0415, w!("pl-PL")),
    lcid(0x0416, w!("pt-BR")),
    lcid(0x0418, w!("ro-RO")),
    lcid(0x0419, w!("ru-RU")),
    lcid(0x041A, w!("hr-HR")),
    lcid(0x041B, w!("sk-SK")),
    lcid(0x041C, w!("sq-AL")),
    lcid(0x041D, w!("sv-SE")),
    lcid(0x041E, w!("th-TH")),
    lcid(0x041F, w!("tr-TR")),
    lcid(0x0420, w!("ur-PK")),
    lcid(0x0421, w!("id-ID")),
    lcid(0x0422, w!("uk-UA")),
    lcid(0x0423, w!("be-BY")),
    lcid(0x0424, w!("sl-SI")),
    lcid(0x0425, w!("et-EE")),
    lcid(0x0426, w!("lv-LV")),
    lcid(0x0427, w!("lt-LT")),
    lcid(0x0429, w!("fa-IR")),
    lcid(0x042A, w!("vi-VN")),
    lcid(0x042B, w!("hy-AM")),
    lcid(0x042C, w!("az-AZ-Latn")),
    lcid(0x042D, w!("eu-ES")),
    lcid(0x042F, w!("mk-MK")),
    lcid(0x0432, w!("tn-ZA")),
    lcid(0x0434, w!("xh-ZA")),
    lcid(0x0435, w!("zu-ZA")),
    lcid(0x0436, w!("af-ZA")),
    lcid(0x0437, w!("ka-GE")),
    lcid(0x0438, w!("fo-FO")),
    lcid(0x0439, w!("hi-IN")),
    lcid(0x043A, w!("mt-MT")),
    lcid(0x043B, w!("se-NO")),
    lcid(0x043E, w!("ms-MY")),
    lcid(0x043F, w!("kk-KZ")),
    lcid(0x0440, w!("ky-KG")),
    lcid(0x0441, w!("sw-KE")),
    lcid(0x0443, w!("uz-UZ-Latn")),
    lcid(0x0444, w!("tt-RU")),
    lcid(0x0445, w!("bn-IN")),
    lcid(0x0446, w!("pa-IN")),
    lcid(0x0447, w!("gu-IN")),
    lcid(0x0449, w!("ta-IN")),
    lcid(0x044A, w!("te-IN")),
    lcid(0x044B, w!("kn-IN")),
    lcid(0x044C, w!("ml-IN")),
    lcid(0x044E, w!("mr-IN")),
    lcid(0x044F, w!("sa-IN")),
    lcid(0x0450, w!("mn-MN")),
    lcid(0x0452, w!("cy-GB")),
    lcid(0x0456, w!("gl-ES")),
    lcid(0x0457, w!("kok-IN")),
    lcid(0x045A, w!("syr-SY")),
    lcid(0x0465, w!("div-MV")),
    lcid(0x046B, w!("quz-BO")),
    lcid(0x046C, w!("ns-ZA")),
    lcid(0x0481, w!("mi-NZ")),
    lcid(0x0801, w!("ar-IQ")),
    lcid(0x0804, w!("zh-CN")),
    lcid(0x0807, w!("de-CH")),
    lcid(0x0809, w!("en-GB")),
    lcid(0x080A, w!("es-MX")),
    lcid(0x080C, w!("fr-BE")),
    lcid(0x0810, w!("it-CH")),
    lcid(0x0813, w!("nl-BE")),
    lcid(0x0814, w!("nn-NO")),
    lcid(0x0816, w!("pt-PT")),
    lcid(0x081A, w!("sr-SP-Latn")),
    lcid(0x081D, w!("sv-FI")),
    lcid(0x082C, w!("az-AZ-Cyrl")),
    lcid(0x083B, w!("se-SE")),
    lcid(0x083E, w!("ms-BN")),
    lcid(0x0843, w!("uz-UZ-Cyrl")),
    lcid(0x086B, w!("quz-EC")),
    lcid(0x0C01, w!("ar-EG")),
    lcid(0x0C04, w!("zh-HK")),
    lcid(0x0C07, w!("de-AT")),
    lcid(0x0C09, w!("en-AU")),
    lcid(0x0C0A, w!("es-ES")),
    lcid(0x0C0C, w!("fr-CA")),
    lcid(0x0C1A, w!("sr-SP-Cyrl")),
    lcid(0x0C3B, w!("se-FI")),
    lcid(0x0C6B, w!("quz-PE")),
    lcid(0x1001, w!("ar-LY")),
    lcid(0x1004, w!("zh-SG")),
    lcid(0x1007, w!("de-LU")),
    lcid(0x1009, w!("en-CA")),
    lcid(0x100A, w!("es-GT")),
    lcid(0x100C, w!("fr-CH")),
    lcid(0x101A, w!("hr-BA")),
    lcid(0x103B, w!("smj-NO")),
    lcid(0x1401, w!("ar-DZ")),
    lcid(0x1404, w!("zh-MO")),
    lcid(0x1407, w!("de-LI")),
    lcid(0x1409, w!("en-NZ")),
    lcid(0x140A, w!("es-CR")),
    lcid(0x140C, w!("fr-LU")),
    lcid(0x141A, w!("bs-BA-Latn")),
    lcid(0x143B, w!("smj-SE")),
    lcid(0x1801, w!("ar-MA")),
    lcid(0x1809, w!("en-IE")),
    lcid(0x180A, w!("es-PA")),
    lcid(0x180C, w!("fr-MC")),
    lcid(0x181A, w!("sr-BA-Latn")),
    lcid(0x183B, w!("sma-NO")),
    lcid(0x1C01, w!("ar-TN")),
    lcid(0x1C09, w!("en-ZA")),
    lcid(0x1C0A, w!("es-DO")),
    lcid(0x1C1A, w!("sr-BA-Cyrl")),
    lcid(0x1C3B, w!("sma-SE")),
    lcid(0x2001, w!("ar-OM")),
    lcid(0x2009, w!("en-JM")),
    lcid(0x200A, w!("es-VE")),
    lcid(0x203B, w!("sms-FI")),
    lcid(0x2401, w!("ar-YE")),
    lcid(0x2409, w!("en-CB")),
    lcid(0x240A, w!("es-CO")),
    lcid(0x243B, w!("smn-FI")),
    lcid(0x2801, w!("ar-SY")),
    lcid(0x2809, w!("en-BZ")),
    lcid(0x280A, w!("es-PE")),
    lcid(0x2C01, w!("ar-JO")),
    lcid(0x2C09, w!("en-TT")),
    lcid(0x2C0A, w!("es-AR")),
    lcid(0x3001, w!("ar-LB")),
    lcid(0x3009, w!("en-ZW")),
    lcid(0x300A, w!("es-EC")),
    lcid(0x3401, w!("ar-KW")),
    lcid(0x3409, w!("en-PH")),
    lcid(0x340A, w!("es-CL")),
    lcid(0x3801, w!("ar-AE")),
    lcid(0x380A, w!("es-UY")),
    lcid(0x3C01, w!("ar-BH")),
    lcid(0x3C0A, w!("es-PY")),
    lcid(0x4001, w!("ar-QA")),
    lcid(0x400A, w!("es-BO")),
    lcid(0x440A, w!("es-SV")),
    lcid(0x480A, w!("es-HN")),
    lcid(0x4C0A, w!("es-NI")),
    lcid(0x500A, w!("es-PR")),
    lcid(0x7C04, w!("zh-CHT")),
    lcid(0x7C1A, w!("sr")),
]);

static LOCALE_NAME_TO_INDEX_TABLE: SyncConst<[LocaleNameIndex; 228]> = SyncConst([
    idx(w!(""), 66),
    idx(w!("af"), 44),
    idx(w!("af-za"), 113),
    idx(w!("ar"), 0),
    idx(w!("ar-ae"), 216),
    idx(w!("ar-bh"), 218),
    idx(w!("ar-dz"), 177),
    idx(w!("ar-eg"), 160),
    idx(w!("ar-iq"), 143),
    idx(w!("ar-jo"), 207),
    idx(w!("ar-kw"), 213),
    idx(w!("ar-lb"), 210),
    idx(w!("ar-ly"), 169),
    idx(w!("ar-ma"), 185),
    idx(w!("ar-om"), 196),
    idx(w!("ar-qa"), 220),
    idx(w!("ar-sa"), 67),
    idx(w!("ar-sy"), 204),
    idx(w!("ar-tn"), 191),
    idx(w!("ar-ye"), 200),
    idx(w!("az"), 41),
    idx(w!("az-az-cyrl"), 155),
    idx(w!("az-az-latn"), 107),
    idx(w!("be"), 33),
    idx(w!("be-by"), 99),
    idx(w!("bg"), 1),
    idx(w!("bg-bg"), 68),
    idx(w!("bn-in"), 125),
    idx(w!("bs-ba-latn"), 183),
    idx(w!("ca"), 2),
    idx(w!("ca-es"), 69),
    idx(w!("cs"), 4),
    idx(w!("cs-cz"), 71),
    idx(w!("cy-gb"), 135),
    idx(w!("da"), 5),
    idx(w!("da-dk"), 72),
    idx(w!("de"), 6),
    idx(w!("de-at"), 162),
    idx(w!("de-ch"), 145),
    idx(w!("de-de"), 73),
    idx(w!("de-li"), 179),
    idx(w!("de-lu"), 171),
    idx(w!("div"), 65),
    idx(w!("div-mv"), 139),
    idx(w!("el"), 7),
    idx(w!("el-gr"), 74),
    idx(w!("en"), 8),
    idx(w!("en-au"), 163),
    idx(w!("en-bz"), 205),
    idx(w!("en-ca"), 172),
    idx(w!("en-cb"), 201),
    idx(w!("en-gb"), 146),
    idx(w!("en-ie"), 186),
    idx(w!("en-jm"), 197),
    idx(w!("en-nz"), 180),
    idx(w!("en-ph"), 214),
    idx(w!("en-tt"), 208),
    idx(w!("en-us"), 75),
    idx(w!("en-za"), 192),
    idx(w!("en-zw"), 211),
    idx(w!("es"), 9),
    idx(w!("es-ar"), 209),
    idx(w!("es-bo"), 221),
    idx(w!("es-cl"), 215),
    idx(w!("es-co"), 202),
    idx(w!("es-cr"), 181),
    idx(w!("es-do"), 193),
    idx(w!("es-ec"), 212),
    idx(w!("es-es"), 164),
    idx(w!("es-gt"), 173),
    idx(w!("es-hn"), 223),
    idx(w!("es-mx"), 147),
    idx(w!("es-ni"), 224),
    idx(w!("es-pa"), 187),
    idx(w!("es-pe"), 206),
    idx(w!("es-pr"), 225),
    idx(w!("es-py"), 219),
    idx(w!("es-sv"), 222),
    idx(w!("es-uy"), 217),
    idx(w!("es-ve"), 198),
    idx(w!("et"), 35),
    idx(w!("et-ee"), 101),
    idx(w!("eu"), 42),
    idx(w!("eu-es"), 108),
    idx(w!("fa"), 38),
    idx(w!("fa-ir"), 104),
    idx(w!("fi"), 10),
    idx(w!("fi-fi"), 76),
    idx(w!("fo"), 46),
    idx(w!("fo-fo"), 115),
    idx(w!("fr"), 11),
    idx(w!("fr-be"), 148),
    idx(w!("fr-ca"), 165),
    idx(w!("fr-ch"), 174),
    idx(w!("fr-fr"), 77),
    idx(w!("fr-lu"), 182),
    idx(w!("fr-mc"), 188),
    idx(w!("gl"), 62),
    idx(w!("gl-es"), 136),
    idx(w!("gu"), 55),
    idx(w!("gu-in"), 127),
    idx(w!("he"), 12),
    idx(w!("he-il"), 78),
    idx(w!("hi"), 47),
    idx(w!("hi-in"), 116),
    idx(w!("hr"), 24),
    idx(w!("hr-ba"), 175),
    idx(w!("hr-hr"), 90),
    idx(w!("hu"), 13),
    idx(w!("hu-hu"), 79),
    idx(w!("hy"), 40),
    idx(w!("hy-am"), 106),
    idx(w!("id"), 31),
    idx(w!("id-id"), 97),
    idx(w!("is"), 14),
    idx(w!("is-is"), 80),
    idx(w!("it"), 15),
    idx(w!("it-ch"), 149),
    idx(w!("it-it"), 81),
    idx(w!("ja"), 16),
    idx(w!("ja-jp"), 82),
    idx(w!("ka"), 45),
    idx(w!("ka-ge"), 114),
    idx(w!("kk"), 49),
    idx(w!("kk-kz"), 120),
    idx(w!("kn"), 58),
    idx(w!("kn-in"), 130),
    idx(w!("ko"), 17),
    idx(w!("kok"), 63),
    idx(w!("kok-in"), 137),
    idx(w!("ko-kr"), 83),
    idx(w!("ky"), 50),
    idx(w!("ky-kg"), 121),
    idx(w!("lt"), 37),
    idx(w!("lt-lt"), 103),
    idx(w!("lv"), 36),
    idx(w!("lv-lv"), 102),
    idx(w!("mi-nz"), 142),
    idx(w!("mk"), 43),
    idx(w!("mk-mk"), 109),
    idx(w!("ml-in"), 131),
    idx(w!("mn"), 61),
    idx(w!("mn-mn"), 134),
    idx(w!("mr"), 59),
    idx(w!("mr-in"), 132),
    idx(w!("ms"), 48),
    idx(w!("ms-bn"), 157),
    idx(w!("ms-my"), 119),
    idx(w!("mt-mt"), 117),
    idx(w!("nb-no"), 85),
    idx(w!("nl"), 18),
    idx(w!("nl-be"), 150),
    idx(w!("nl-nl"), 84),
    idx(w!("nn-no"), 151),
    idx(w!("no"), 19),
    idx(w!("ns-za"), 141),
    idx(w!("pa"), 54),
    idx(w!("pa-in"), 126),
    idx(w!("pl"), 20),
    idx(w!("pl-pl"), 86),
    idx(w!("pt"), 21),
    idx(w!("pt-br"), 87),
    idx(w!("pt-pt"), 152),
    idx(w!("quz-bo"), 140),
    idx(w!("quz-ec"), 159),
    idx(w!("quz-pe"), 168),
    idx(w!("ro"), 22),
    idx(w!("ro-ro"), 88),
    idx(w!("ru"), 23),
    idx(w!("ru-ru"), 89),
    idx(w!("sa"), 60),
    idx(w!("sa-in"), 133),
    idx(w!("se-fi"), 167),
    idx(w!("se-no"), 118),
    idx(w!("se-se"), 156),
    idx(w!("sk"), 25),
    idx(w!("sk-sk"), 91),
    idx(w!("sl"), 34),
    idx(w!("sl-si"), 100),
    idx(w!("sma-no"), 190),
    idx(w!("sma-se"), 195),
    idx(w!("smj-no"), 176),
    idx(w!("smj-se"), 184),
    idx(w!("smn-fi"), 203),
    idx(w!("sms-fi"), 199),
    idx(w!("sq"), 26),
    idx(w!("sq-al"), 92),
    idx(w!("sr"), 227),
    idx(w!("sr-ba-cyrl"), 194),
    idx(w!("sr-ba-latn"), 189),
    idx(w!("sr-sp-cyrl"), 166),
    idx(w!("sr-sp-latn"), 153),
    idx(w!("sv"), 27),
    idx(w!("sv-fi"), 154),
    idx(w!("sv-se"), 93),
    idx(w!("sw"), 51),
    idx(w!("sw-ke"), 122),
    idx(w!("syr"), 64),
    idx(w!("syr-sy"), 138),
    idx(w!("ta"), 56),
    idx(w!("ta-in"), 128),
    idx(w!("te"), 57),
    idx(w!("te-in"), 129),
    idx(w!("th"), 28),
    idx(w!("th-th"), 94),
    idx(w!("tn-za"), 110),
    idx(w!("tr"), 29),
    idx(w!("tr-tr"), 95),
    idx(w!("tt"), 53),
    idx(w!("tt-ru"), 124),
    idx(w!("uk"), 32),
    idx(w!("uk-ua"), 98),
    idx(w!("ur"), 30),
    idx(w!("ur-pk"), 96),
    idx(w!("uz"), 52),
    idx(w!("uz-uz-cyrl"), 158),
    idx(w!("uz-uz-latn"), 123),
    idx(w!("vi"), 39),
    idx(w!("vi-vn"), 105),
    idx(w!("xh-za"), 111),
    idx(w!("zh-chs"), 3),
    idx(w!("zh-cht"), 226),
    idx(w!("zh-cn"), 144),
    idx(w!("zh-hk"), 161),
    idx(w!("zh-mo"), 178),
    idx(w!("zh-sg"), 170),
    idx(w!("zh-tw"), 70),
    idx(w!("zu-za"), 112),
]);

/// `GetTableIndexFromLocaleName`: binary search of the name index, -1 if absent.
unsafe fn table_index_from_locale_name(locale_name: *const u16) -> c_int {
    let table = &LOCALE_NAME_TO_INDEX_TABLE.0;
    let mut bottom: c_int = 0;
    let mut top: c_int = table.len() as c_int - 1;

    while bottom <= top {
        let middle = (bottom + top) / 2;
        // SAFETY: 0 <= bottom <= middle <= top < len. Raw indexing, as the C
        // does, so no bounds-check panic path is compiled into a CRT export.
        let entry = unsafe { &*table.as_ptr().add(middle as usize) };
        // SAFETY: the caller's name is NUL-terminated (or at least
        // LOCALE_NAME_MAX_LENGTH long); the table's are ours.
        let test_index =
            unsafe { __ascii_wcsnicmp(locale_name, entry.name, LOCALE_NAME_MAX_LENGTH) };

        if test_index == 0 {
            return entry.index;
        }

        if test_index < 0 {
            top = middle - 1;
        } else {
            bottom = middle + 1;
        }
    }

    -1
}

/// `GetTableIndexFromLcid`: binary search of the LCID table, -1 if absent. The
/// comparison is the C's `int testIndex = lcid - table.lcid` (unsigned
/// subtraction, signed test).
fn table_index_from_lcid(lcid: u32) -> c_int {
    let table = &LCID_TO_LOCALE_NAME_TABLE.0;
    let mut bottom: c_int = 0;
    let mut top: c_int = table.len() as c_int - 1;

    while bottom <= top {
        let middle = (bottom + top) / 2;
        // SAFETY: 0 <= bottom <= middle <= top < len (see above).
        let entry = unsafe { &*table.as_ptr().add(middle as usize) };
        let test_index = lcid.wrapping_sub(entry.lcid) as c_int;

        if test_index == 0 {
            return middle;
        }

        if test_index < 0 {
            top = middle - 1;
        } else {
            bottom = middle + 1;
        }
    }

    -1
}

/// The locale name's LCID from the XP table, or 0 (NULL, unknown, or one of
/// the two names the table's ordering hides -- see the module docs).
#[no_mangle]
pub unsafe extern "C" fn __acrt_DownlevelLocaleNameToLCID(locale_name: *const u16) -> u32 {
    if locale_name.is_null() {
        return 0;
    }

    // SAFETY: non-NULL, and the caller's contract is a NUL-terminated name.
    let index = unsafe { table_index_from_locale_name(locale_name) };

    let table = &LCID_TO_LOCALE_NAME_TABLE.0;
    if index < 0 || index as usize >= table.len() {
        return 0;
    }

    // SAFETY: 0 <= index < len, checked just above.
    unsafe { (*table.as_ptr().add(index as usize)).lcid }
}

/// The LCID's name from the XP table into `out` (`cch` units), returning the
/// length including the terminator, or 0. `cch == 0` is a size query.
#[no_mangle]
pub unsafe extern "C" fn __acrt_DownlevelLCIDToLocaleName(
    lcid: u32,
    out_locale_name: *mut u16,
    cch_locale_name: c_int,
) -> c_int {
    if lcid == 0 || lcid == LOCALE_USER_DEFAULT || lcid == LOCALE_SYSTEM_DEFAULT {
        return 0;
    }

    if (out_locale_name.is_null() && cch_locale_name > 0) || cch_locale_name < 0 {
        return 0;
    }

    let index = table_index_from_lcid(lcid);
    if index < 0 {
        return 0;
    }

    // SAFETY: a found index is in 0..len.
    let buffer = unsafe { (*LCID_TO_LOCALE_NAME_TABLE.0.as_ptr().add(index as usize)).locale_name };
    // SAFETY: every table name is a NUL-terminated static.
    let count = unsafe { wcsnlen(buffer, LOCALE_NAME_MAX_LENGTH) };

    if cch_locale_name > 0 {
        if count as c_int >= cch_locale_name {
            return 0;
        }

        // _ERRCHECK: Watson on any nonzero errno_t. Unreachable given the
        // checks above, but the C makes the call and tests it, so do we.
        // SAFETY: `out_locale_name` is non-NULL with `cch_locale_name` units.
        let e = unsafe { wcscpy_s(out_locale_name, cch_locale_name as usize, buffer) };
        if e != 0 {
            // SAFETY: the release-build expansion, _invoke_watson(NULL x3, 0, 0).
            unsafe { _invoke_watson(null(), null(), null(), 0, 0) };
        }
    }

    count as c_int + 1
}
