use crate::prelude::*;

// ── time.c's strftime TEMPLATE, resolved for THIS DLL ──────────────────────────────
//
// `time.c:888` selects the whole strftime engine's character type with a preprocessor
// ladder, and at `_MSVCR_VER <= 90` — which 0 satisfies — the engine is NARROW:
//
//     #if _MSVCR_VER <= 90
//     #define STRFTIME_CHAR char
//     #define STRFTIME_TD(td, name) td->str.names.name
//     #else                                  <- wide; not this build
//
// So `wcsftime` here is not a wide engine: `wcsftime_helper` converts the format down
// with `_wcstombs_l`, runs the narrow `strftime_impl`, and converts the result back
// with `_mbstowcs_l`. Getting this backwards would silently read the `wstr` half of
// `__lc_time_data` — a union arm that at this configuration nothing fills.
//
// These stay MACROS rather than becoming functions on purpose: `STRFTIME_TD` selects a
// FIELD by name, which no function signature can express, and a field selector lowered
// to an out-of-line `fn` is the §9.65 shape besides.
type STRFTIME_CHAR = c_char;

/// `time.c:890` — `#define STRFTIME_TD(td, name) td->str.names.name`.
/// bindgen spells the union arm `str_` (`str` is a Rust keyword).
macro_rules! STRFTIME_TD {
    ($td:expr, $name:ident) => {
        (*$td).str_.names.$name
    };
}
/// The same selector at a subscript. The C writes `STRFTIME_TD(td, name)[i]` and relies
/// on array-to-pointer decay; the Rust field IS an array, so it indexes directly.
macro_rules! STRFTIME_TD_IDX {
    ($td:expr, $name:ident, $i:expr) => {
        (*$td).str_.names.$name[$i]
    };
}
/// `time.c:896` — `#define strftime_str(a,b,c,d) strftime_nstr(a,b,c,d,SIZE_MAX)`.
macro_rules! strftime_str {
    ($a:expr, $b:expr, $c:expr, $d:expr $(,)?) => {
        strftime_nstr($a, $b, $c, $d, SIZE_MAX)
    };
}
/// `<stdint.h>`'s `SIZE_MAX`, which the macro above passes as "no length limit".
const SIZE_MAX: size_t = size_t::MAX;

// ── what time.c calls and does not define ──────────────────────────────────────────
// Transcribed from the headers time.c itself includes, never inferred from the call
// site — see prelude.rs for why an `extern` block is the one place a wrong type is
// invisible.
extern "C" {
    /// `stdlib.h`: `long strtol(const char *nptr, char **endptr, int base);`
    fn strtol(nptr: *const c_char, endptr: *mut *mut c_char, base: c_int) -> __msvcrt_long;
    /// `locale.h`: msvcrt's locale-aware wide->multibyte conversion.
    fn _wcstombs_l(
        mbstr: *mut c_char,
        wcstr: *const wchar_t,
        count: size_t,
        locale: _locale_t,
    ) -> size_t;
    /// `locale.h`: the inverse, used by `wcsftime_helper` to bring the result back up.
    fn _mbstowcs_l(
        wcstr: *mut wchar_t,
        mbstr: *const c_char,
        count: size_t,
        locale: _locale_t,
    ) -> size_t;
    /// `stdlib.h`: `void *malloc(size_t)` / `void free(void *)`.
    fn malloc(size: size_t) -> *mut c_void;
    fn free(ptr: *mut c_void);
}


/// `char x[4]` from a 3-byte spelling: the C writes `{"Sun", ...}` into a
/// `char[7][4]`, where each element is the three characters plus the NUL the array's
/// extra byte supplies. `*b"Sun"` is `[u8; 3]` — one byte short and the wrong
/// signedness — so spell the padding out.
const fn cstr4(s: &[u8; 3]) -> [c_char; 4] {
    [s[0] as c_char, s[1] as c_char, s[2] as c_char, 0]
}

// ---- file-scope preamble (translated) ----
static mut init_time: LONGLONG = 0;

pub static MonthLengths: [[i32; 12]; 2] = [
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

pub static MAX_SECONDS: i32 = 59;

// `time.c:69` is a THREE-armed ladder and this DLL takes the FIRST arm:
//
//     #if _MSVCR_VER == 0
//     #define MIN_GMTIME64_TIME 0
//     #define MAX_GMTIME64_TIME _MAX__TIME64_T
//     #elif _MSVCR_VER >= 140                  <- not this one
//     #else                                    <- nor this one
//     #define MIN_GMTIME64_TIME -43200
//     #define MAX_GMTIME64_TIME (_MAX__TIME64_T + 46800)
//     #endif
//
// The translation took the `#else`, which is not a compile error and not a wrong
// *number* — it is the wrong ARM, and it widens `_gmtime64_s`'s accepted range at
// both ends: 43,200 seconds of pre-epoch input that msvcrt rejects with EINVAL, and
// 46,800 seconds past `_MAX__TIME64_T` that it also rejects. `--seal-facts` protects
// constant OCCURRENCES; it cannot see which `#if` arm defined the constant, so this
// class is unguarded and has to be checked by hand against the `_MSVCR_VER` the
// build really uses. `_MAX__TIME64_T` itself is taken from the island rather than
// spelled out — it is 0x7_93406FFF, and the arithmetic to expand that by hand is
// exactly where the original went wrong.
pub const MIN_GMTIME64_TIME: i64 = 0;
pub const MAX_GMTIME64_TIME: i64 = _MAX__TIME64_T as i64;

// ── the four EXPORTED globals ──────────────────────────────────────────────────
// `#[no_mangle]`, and that is the whole flip for them. msvcrt.dll exports `_daylight`,
// `_timezone`, `_dstbias` and `_tzname` as DATA (msvcrt.spec maps each to the
// `MSVCRT_*` name below), so without it the C copies in `msvcrt_static` stay live and
// win: `_tzset` would update Rust's private copy while every app reading the exported
// `_timezone` saw the initial 28800 forever. A missing `#[no_mangle]` on a data symbol
// is silent in a way a missing one on a function is not — nothing fails to link,
// because the C definition is right there to satisfy the import.
#[no_mangle]
pub static mut MSVCRT___daylight: i32 = 1;
#[no_mangle]
pub static mut MSVCRT___timezone: __msvcrt_long = 28800;
#[no_mangle]
pub static mut MSVCRT__dstbias: __msvcrt_long = -3600;

/// `time.c:118` — `static char tzname_std[64] = "PST";`. 64 bytes, not 4:
/// "some apps (notably Mozilla) insist on writing to these".
static mut tzname_std: [c_char; 64] = {
    let mut v = [0 as c_char; 64];
    let init = [b'P' as c_char, b'S' as c_char, b'T' as c_char];
    let mut i = 0;
    while i < init.len() { v[i] = init[i]; i += 1; }
    v
};

/// `time.c:118` — `static char tzname_dst[64] = "PDT";`. 64 bytes, not 4:
/// "some apps (notably Mozilla) insist on writing to these".
static mut tzname_dst: [c_char; 64] = {
    let mut v = [0 as c_char; 64];
    let init = [b'P' as c_char, b'D' as c_char, b'T' as c_char];
    let mut i = 0;
    while i < init.len() { v[i] = init[i]; i += 1; }
    v
};

// `char *MSVCRT__tzname[2] = { tzname_std, tzname_dst };` — the two buffers, NOT two
// NULLs. The translation emitted `null_mut()` for both, which is not a compile error:
// `_tzset`'s very first act on the TZ-string path is `lstrcpynA(MSVCRT__tzname[0], ...)`,
// so the flip would have written through a null pointer on the first call. The C
// comment above the buffers explains why they are 64 bytes and not 4 — "some apps
// (notably Mozilla) insist on writing to these" — which is the same reason they must
// be real storage here.
#[no_mangle]
pub static mut MSVCRT__tzname: [*mut i8; 2] = [
    core::ptr::addr_of!(tzname_std) as *mut i8,
    core::ptr::addr_of!(tzname_dst) as *mut i8,
];

static mut tzi: TIME_ZONE_INFORMATION = TIME_ZONE_INFORMATION {
    Bias: 0,
    StandardName: [0; 32],
    StandardDate: SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    },
    StandardBias: 0,
    DaylightName: [0; 32],
    DaylightDate: SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    },
    DaylightBias: 0,
};

pub const SECSPERDAY: i32 = 86400;
pub const SECS_1601_TO_1970: u64 = (369 * 365 + 89) * SECSPERDAY as u64;
pub const TICKSPERSEC: i32 = 10000000;
pub const TICKSPERMSEC: i32 = 10000;
pub const TICKS_1601_TO_1970: u64 = SECS_1601_TO_1970 * TICKSPERSEC as u64;
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_clock() {
    let mut systime: LARGE_INTEGER = core::mem::zeroed();
    NtQuerySystemTime(core::ptr::addr_of_mut!(systime));
    init_time = systime.QuadPart;
}
unsafe extern "C" fn IsLeapYear(Year: i32) -> i32 {
    ((Year % 4 == 0) && (Year % 100 != 0 || Year % 400 == 0)) as i32
}
unsafe extern "C" fn write_invalid_msvcrt_tm(tm: *mut tm) {
    (*tm).tm_sec = -1;
    (*tm).tm_min = -1;
    (*tm).tm_hour = -1;
    (*tm).tm_mday = -1;
    (*tm).tm_mon = -1;
    (*tm).tm_year = -1;
    (*tm).tm_wday = -1;
    (*tm).tm_yday = -1;
    (*tm).tm_isdst = -1;
}

// ── HAND-REPAIRED, and this one is worth reading before the next wave ─────────────
//
// The paid translation of THIS function came back with every constant in the wrong
// slot: `_lock(0)` for `_lock(_TIME_LOCK as c_int)`, `memset(&tzi, 0, 3)` for `sizeof(tzi)`,
// `*tz == 3600` for `*tz == '-'`, `tzi.Bias * CP_ACP` for `tzi.Bias * 60`. Not one
// invented value — the multiset was exactly right — every value simply landed
// somewhere else.
//
// `--seal-facts` was ON and did not catch it, because its invariant is a CLOSED SET
// (nothing missing, nothing invented) and a permutation satisfies that. What actually
// happened is in fact_seal.py's recovery path: the model ignored the envelope and
// wrote real spellings from memory, the recovery re-tokenized those spellings in
// SOURCE ORDER, and restoration became an identity — so the seal passed through the
// model's guesses while reporting success. 35 tokens, none honoured.
//
// Restored from `dll/win32/msvcrt/time.c:127` line by line. Two things here look like
// mistakes and are NOT: the second `WideCharToMultiByte` failure arm really does zero
// `MSVCRT__tzname[0]` rather than `[1]` (upstream reads that way, and a reference is
// only useful if it is faithful), and `MSVCRT___daylight` really is assigned the CHAR
// at `*tz` rather than a boolean.
#[no_mangle]
pub unsafe extern "C" fn _tzset() {
    let mut tz: *mut c_char = getenv(b"TZ\0".as_ptr() as *const c_char);
    let mut error: BOOL = 0;

    _lock(_TIME_LOCK as c_int);
    if !tz.is_null() && *tz.offset(0) != 0 {
        let mut neg_zone: BOOL = FALSE as BOOL;

        memset(
            core::ptr::addr_of_mut!(tzi) as *mut c_void,
            0,
            core::mem::size_of::<TIME_ZONE_INFORMATION>(),
        );

        /* Parse timezone information: tzn[+|-]hh[:mm[:ss]][dzn] */
        lstrcpynA(MSVCRT__tzname[0], tz, 3);
        tz = tz.offset(3);

        if *tz == b'-' as c_char {
            neg_zone = TRUE as BOOL;
            tz = tz.offset(1);
        } else if *tz == b'+' as c_char {
            tz = tz.offset(1);
        }
        MSVCRT___timezone = strtol(tz, core::ptr::addr_of_mut!(tz), 10) * 3600;
        if *tz == b':' as c_char {
            MSVCRT___timezone += strtol(tz.offset(1), core::ptr::addr_of_mut!(tz), 10) * 60;
            if *tz == b':' as c_char {
                MSVCRT___timezone += strtol(tz.offset(1), core::ptr::addr_of_mut!(tz), 10);
            }
        }
        if neg_zone != 0 {
            MSVCRT___timezone = -MSVCRT___timezone;
        }

        MSVCRT___daylight = *tz as c_int;
        lstrcpynA(MSVCRT__tzname[1], tz, 3);
    } else if GetTimeZoneInformation(core::ptr::addr_of_mut!(tzi)) != TIME_ZONE_ID_INVALID {
        MSVCRT___timezone = tzi.Bias * 60;
        if tzi.StandardDate.wMonth != 0 {
            MSVCRT___timezone += tzi.StandardBias * 60;
        }

        if tzi.DaylightDate.wMonth != 0 {
            MSVCRT___daylight = 1;
            MSVCRT__dstbias = (tzi.DaylightBias - tzi.StandardBias) * 60;
        } else {
            MSVCRT___daylight = 0;
            MSVCRT__dstbias = 0;
        }

        if WideCharToMultiByte(
            CP_ACP,
            0,
            tzi.StandardName.as_ptr(),
            -1,
            MSVCRT__tzname[0],
            core::mem::size_of_val(&tzname_std) as c_int,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(error),
        ) == 0
            || error != 0
        {
            *MSVCRT__tzname[0] = 0;
        }
        if WideCharToMultiByte(
            CP_ACP,
            0,
            tzi.DaylightName.as_ptr(),
            -1,
            MSVCRT__tzname[1],
            core::mem::size_of_val(&tzname_dst) as c_int,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(error),
        ) == 0
            || error != 0
        {
            *MSVCRT__tzname[0] = 0;
        }
    }
    _unlock(_TIME_LOCK as c_int);
}
unsafe extern "C" fn _tzset_init() {
    static mut init: BOOL = (FALSE) as i32;

    if init == 0 {
        _lock(_TIME_LOCK as c_int);
        if init == 0 {
            _tzset();
            init = (TRUE) as i32;
        }
        _unlock(_TIME_LOCK as c_int);
    }
}
unsafe extern "C" fn is_dst(st: *const SYSTEMTIME) -> BOOL {
    let mut tmp: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut out: SYSTEMTIME = core::mem::zeroed();

    if MSVCRT___daylight == 0 {
        return (FALSE) as i32;
    }

    if tzi.DaylightDate.wMonth != 0 {
        tmp = tzi;
    } else if (*st).wYear >= 2007 {
        memset(
            &mut tmp as *mut _ as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<TIME_ZONE_INFORMATION>(),
        );
        tmp.StandardDate.wMonth = 11;
        tmp.StandardDate.wDay = 1;
        tmp.StandardDate.wHour = 2;
        tmp.DaylightDate.wMonth = 3;
        tmp.DaylightDate.wDay = 2;
        tmp.DaylightDate.wHour = 2;
    } else {
        memset(
            &mut tmp as *mut _ as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<TIME_ZONE_INFORMATION>(),
        );
        tmp.StandardDate.wMonth = 10;
        tmp.StandardDate.wDay = 5;
        tmp.StandardDate.wHour = 2;
        tmp.DaylightDate.wMonth = 4;
        tmp.DaylightDate.wDay = 1;
        tmp.DaylightDate.wHour = 2;
    }

    tmp.Bias = 0;
    tmp.StandardBias = 0;
    tmp.DaylightBias = MSVCRT__dstbias / 60;
    if SystemTimeToTzSpecificLocalTime(&tmp, st, &mut out) == 0 {
        return (FALSE) as i32;
    }

    return memcmp(
        st as *const core::ffi::c_void,
        &out as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>(),
    );
}
unsafe extern "C" fn mktime_helper(mstm: *mut tm, local: BOOL) -> __time64_t {
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut ft: FILETIME = core::mem::zeroed();
    let mut ret: __time64_t = 0;
    let mut i: i32;
    let mut use_dst: BOOL = (FALSE) as i32;

    ret = ((*mstm).tm_year + (*mstm).tm_mon / 12) as i64;
    (*mstm).tm_mon %= 12;
    if (*mstm).tm_mon < 0 {
        (*mstm).tm_mon += 12;
        ret -= 1;
    }

    if ret < 70 || ret > 1100 {
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    memset(
        core::ptr::addr_of_mut!(st) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<SYSTEMTIME>(),
    );
    st.wDay = 1;
    st.wMonth = ((*mstm).tm_mon + 1) as u16;
    st.wYear = (ret + 1900) as u16;

    if SystemTimeToFileTime(core::ptr::addr_of_mut!(st), core::ptr::addr_of_mut!(ft)) == 0 {
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    ret = (ft.dwHighDateTime as __time64_t) << 32 | ft.dwLowDateTime as __time64_t;
    ret += (*mstm).tm_sec as __time64_t * (TICKSPERSEC) as i64;
    ret += (*mstm).tm_min as __time64_t * 60 * (TICKSPERSEC) as i64;
    ret += (*mstm).tm_hour as __time64_t * 60 * 60 * (TICKSPERSEC) as i64;
    ret += ((*mstm).tm_mday - 1) as __time64_t * (SECSPERDAY) as i64 * (TICKSPERSEC) as i64;

    ft.dwLowDateTime = (ret & 0xffffffff) as u32;
    ft.dwHighDateTime = (ret >> 32) as u32;
    FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(st));

    if local != 0 {
        _tzset_init();
        use_dst = is_dst(core::ptr::addr_of_mut!(st));
        if ((*mstm).tm_isdst <= -1 && use_dst != 0) || ((*mstm).tm_isdst >= 1) {
            let mut tmp: SYSTEMTIME = core::mem::zeroed();

            ret += MSVCRT__dstbias as __time64_t * (TICKSPERSEC) as i64;

            ft.dwLowDateTime = (ret & 0xffffffff) as u32;
            ft.dwHighDateTime = (ret >> 32) as u32;
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(tmp));

            if is_dst(core::ptr::addr_of_mut!(tmp)) == 0 {
                st = tmp;
                use_dst = (FALSE) as i32;
            } else {
                use_dst = (TRUE) as i32;
            }
        } else if (*mstm).tm_isdst == 0 && use_dst != 0 {
            ret -= MSVCRT__dstbias as __time64_t * (TICKSPERSEC) as i64;
            ft.dwLowDateTime = (ret & 0xffffffff) as u32;
            ft.dwHighDateTime = (ret >> 32) as u32;
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(st));
            ret += MSVCRT__dstbias as __time64_t * (TICKSPERSEC) as i64;
        }
        ret += MSVCRT___timezone as __time64_t * (TICKSPERSEC) as i64;
    }

    (*mstm).tm_sec = st.wSecond as i32;
    (*mstm).tm_min = st.wMinute as i32;
    (*mstm).tm_hour = st.wHour as i32;
    (*mstm).tm_mday = st.wDay as i32;
    (*mstm).tm_mon = st.wMonth as i32 - 1;
    (*mstm).tm_year = st.wYear as i32 - 1900;
    (*mstm).tm_wday = st.wDayOfWeek as i32;
    (*mstm).tm_yday = 0;
    i = 0;
    while i < st.wMonth as i32 - 1 {
        (*mstm).tm_yday += MonthLengths[IsLeapYear(st.wYear as i32) as usize][i as usize];
        i += 1;
    }
    (*mstm).tm_yday += st.wDay as i32 - 1;
    (*mstm).tm_isdst = if use_dst != 0 { 1 } else { 0 };

    if ret < (TICKS_1601_TO_1970) as i64 {
        *_errno() = (EINVAL) as i32;
        return -1;
    }
    ret = (ret - (TICKS_1601_TO_1970) as i64) / (TICKSPERSEC) as i64;
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mktime64(mstm: *mut tm) -> __time64_t {
    mktime_helper(mstm, (TRUE) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _mktime32(mstm: *mut tm) -> __time32_t {
    let ret = _mktime64(mstm);
    if ret == (ret as __time32_t) as i64 {
        ret as __time32_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mkgmtime64(time: *mut tm) -> __time64_t {
    mktime_helper(time, (FALSE) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _mkgmtime32(time: *mut tm) -> __time32_t {
    let ret = _mkgmtime64(time);
    if ret == (ret as __time32_t) as i64 { ret as __time32_t } else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn _localtime64_s(res: *mut tm, secs: *const __time64_t) -> i32 {
    let mut i: i32;
    let mut ft: FILETIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut time: ULONGLONG;

    if res.is_null() || secs.is_null() || *secs < 0 || *secs > (_MAX__TIME64_T) as i64 {
        if !res.is_null() {
            write_invalid_msvcrt_tm(res);
        }

        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    _tzset_init();
    // Wrapping, and both operations, because THE C RELIES ON THE WRAPAROUND. `*secs -
    // __timezone` is signed and goes negative for any instant inside the first few hours
    // of 1970 in a zone west of Greenwich; C then converts that to `ULONGLONG` (a huge
    // number), multiplies mod 2^64, and adds the epoch offset — and the modular arithmetic
    // lands on exactly the right FILETIME. Rust's `*` and `+` are checked in a debug
    // profile, so the literal translation aborts on `_localtime64(&0)` under UTC+8. The
    // arithmetic is identical in release; spelling it wrapping makes the reliance explicit
    // and makes both profiles agree.
    time = ((*secs - MSVCRT___timezone as __time64_t) as ULONGLONG)
        .wrapping_mul(TICKSPERSEC as ULONGLONG)
        .wrapping_add(TICKS_1601_TO_1970);

    ft.dwHighDateTime = (time >> 32) as u32;
    ft.dwLowDateTime = time as u32;
    FileTimeToSystemTime(&ft, &mut st);

    (*res).tm_isdst = if is_dst(&st) != 0 { 1 } else { 0 };
    if (*res).tm_isdst != 0 {
        // Same wraparound, one level sharper: `__dstbias` is NEGATIVE (-3600 for a
        // one-hour daylight shift), so `(ULONGLONG)__dstbias` is near 2^64 and the
        // multiply overflows on purpose. Subtracting the wrapped product is how the C
        // ADDS the hour.
        time = time
            .wrapping_sub((MSVCRT__dstbias as ULONGLONG).wrapping_mul(TICKSPERSEC as ULONGLONG));
        ft.dwHighDateTime = (time >> 32) as u32;
        ft.dwLowDateTime = time as u32;
        FileTimeToSystemTime(&ft, &mut st);
    }

    (*res).tm_sec = st.wSecond as i32;
    (*res).tm_min = st.wMinute as i32;
    (*res).tm_hour = st.wHour as i32;
    (*res).tm_mday = st.wDay as i32;
    (*res).tm_year = st.wYear as i32 - 1900;
    (*res).tm_mon = st.wMonth as i32 - 1;
    (*res).tm_wday = st.wDayOfWeek as i32;

    (*res).tm_yday = 0;
    i = 0;
    while i < st.wMonth as i32 - 1 {
        (*res).tm_yday += MonthLengths[IsLeapYear(st.wYear as i32) as usize][i as usize];
        i += 1;
    }
    (*res).tm_yday += st.wDay as i32 - 1;

    0
}

#[no_mangle]
pub unsafe extern "C" fn _localtime64(secs: *const __time64_t) -> *mut tm {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    if (*data).time_buffer.is_null() {
        (*data).time_buffer = malloc(core::mem::size_of::<tm>()) as *mut tm;
    }

    if _localtime64_s((*data).time_buffer, secs) != 0 {
        return core::ptr::null_mut();
    }
    (*data).time_buffer
}

#[no_mangle]
pub unsafe extern "C" fn _localtime32(secs: *const __time32_t) -> *mut tm {
    let mut secs64: __time64_t;

    if secs.is_null() {
        return core::ptr::null_mut();
    }

    secs64 = (*secs) as i64;
    _localtime64(&secs64)
}

#[no_mangle]
pub unsafe extern "C" fn _localtime32_s(time: *mut tm, secs: *const __time32_t) -> i32 {
    let mut secs64: __time64_t;

    if time.is_null() || secs.is_null() || *secs < 0 {
        if !time.is_null() {
            write_invalid_msvcrt_tm(time);
        }

        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    secs64 = *secs as __time64_t;
    return _localtime64_s(time, &secs64);
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime64_s(res: *mut tm, secs: *const __time64_t) -> errno_t {
    let mut i: i32;
    let mut ft: FILETIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut time: u64;

    if res.is_null() || secs.is_null() || *secs < MIN_GMTIME64_TIME || *secs > MAX_GMTIME64_TIME {
        if !res.is_null() {
            write_invalid_msvcrt_tm(res);
        }

        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    time = (*secs as u64) * (TICKSPERSEC as u64) + TICKS_1601_TO_1970;

    ft.dwHighDateTime = (time >> 32) as u32;
    ft.dwLowDateTime = time as u32;

    FileTimeToSystemTime(&ft, &mut st);

    (*res).tm_sec = st.wSecond as i32;
    (*res).tm_min = st.wMinute as i32;
    (*res).tm_hour = st.wHour as i32;
    (*res).tm_mday = st.wDay as i32;
    (*res).tm_year = (st.wYear as i32) - 1900;
    (*res).tm_mon = (st.wMonth as i32) - 1;
    (*res).tm_wday = st.wDayOfWeek as i32;
    (*res).tm_yday = 0;
    i = 0;
    while i < (st.wMonth as i32) - 1 {
        (*res).tm_yday += MonthLengths[IsLeapYear(st.wYear as i32) as usize][i as usize];
        i += 1;
    }

    (*res).tm_yday += (st.wDay as i32) - 1;
    (*res).tm_isdst = 0;

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime64(secs: *const __time64_t) -> *mut tm {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    if (*data).time_buffer.is_null() {
        (*data).time_buffer = malloc(core::mem::size_of::<tm>()) as *mut tm;
    }

    if _gmtime64_s((*data).time_buffer, secs) != 0 {
        return core::ptr::null_mut();
    }
    (*data).time_buffer
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime32_s(res: *mut tm, secs: *const __time32_t) -> errno_t {
    let mut secs64: __time64_t;

    if !secs.is_null() {
        secs64 = *secs as __time64_t;
        return _gmtime64_s(res, &secs64);
    }
    return _gmtime64_s(res, core::ptr::null());
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime32(secs: *const __time32_t) -> *mut tm {
    let mut secs64: __time64_t;

    if secs.is_null() {
        return core::ptr::null_mut();
    }

    secs64 = (*secs) as i64;
    _gmtime64(&secs64)
}

#[no_mangle]
pub unsafe extern "C" fn _strdate(date: *mut i8) -> *mut i8 {
    let mut st: SYSTEMTIME = core::mem::zeroed();

    GetLocalTime(&mut st);
    *date.offset(0) = (b'0' as i8) + (st.wMonth / 10) as i8;
    *date.offset(1) = (b'0' as i8) + (st.wMonth % 10) as i8;
    *date.offset(2) = b'/' as i8;
    *date.offset(3) = (b'0' as i8) + (st.wDay / 10) as i8;
    *date.offset(4) = (b'0' as i8) + (st.wDay % 10) as i8;
    *date.offset(5) = b'/' as i8;
    *date.offset(6) = (b'0' as i8) + ((st.wYear / 10) % 10) as i8;
    *date.offset(7) = (b'0' as i8) + (st.wYear % 10) as i8;
    *date.offset(8) = 0;
    date
}

#[no_mangle]
pub unsafe extern "C" fn _strdate_s(date: *mut i8, size: usize) -> i32 {
    if !date.is_null() && size != 0 {
        *date.offset(0 as isize) = b'\0' as i8;
    }

    if date.is_null() {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    if size < 9 {
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }

    _strdate(date);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wstrdate(date: *mut wchar_t) -> *mut wchar_t {
    // `L"MM'/'dd'/'yy"` — 12 code units plus a WIDE NUL, so 26 bytes. The terminator is two
    // zero bytes; one leaves GetDateFormatW reading the last unit's high half out of bounds.
    GetDateFormatW(LOCALE_NEUTRAL, 0, core::ptr::null_mut(), win32macros_rust::wide_bytes!(b"M\0M\0'\0/\0'\0d\0d\0'\0/\0'\0y\0y\0\0\0") as *const wchar_t, date, 9);
    date
}

#[no_mangle]
pub unsafe extern "C" fn _wstrdate_s(date: *mut wchar_t, size: usize) -> i32 {
    if !date.is_null() && size != 0 {
        *date.offset(0 as isize) = '\0' as wchar_t;
    }

    if date.is_null() {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    if size < 9 {
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }

    _wstrdate(date);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _strtime(time: *mut u8) -> *mut u8 {
    let mut st: SYSTEMTIME = core::mem::zeroed();

    GetLocalTime(&mut st);
    *time.offset(0) = b'0' + (st.wHour / 10) as u8;
    *time.offset(1) = b'0' + (st.wHour % 10) as u8;
    *time.offset(2) = b':';
    *time.offset(3) = b'0' + (st.wMinute / 10) as u8;
    *time.offset(4) = b'0' + (st.wMinute % 10) as u8;
    *time.offset(5) = b':';
    *time.offset(6) = b'0' + (st.wSecond / 10) as u8;
    *time.offset(7) = b'0' + (st.wSecond % 10) as u8;
    *time.offset(8) = 0;
    time
}

#[no_mangle]
pub unsafe extern "C" fn _strtime_s(time: *mut u8, size: usize) -> i32 {
    if !time.is_null() && size != 0 {
        *time.add(0) = b'\0';
    }

    if time.is_null() {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    if size < 9 {
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }

    _strtime(time);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wstrtime(time: *mut wchar_t) -> *mut wchar_t {
    // `L"HH':'mm':'ss"` — same wide-NUL rule as `_wstrdate` above.
    GetTimeFormatW(LOCALE_NEUTRAL, 0, core::ptr::null_mut(), win32macros_rust::wide_bytes!(b"H\0H\0'\0:\0'\0m\0m\0'\0:\0'\0s\0s\0\0\0") as *const wchar_t, time, 9);
    time
}

#[no_mangle]
pub unsafe extern "C" fn _wstrtime_s(time: *mut wchar_t, size: usize) -> i32 {
    if !time.is_null() && size != 0 {
        *time.add(0) = 0;
    }

    if time.is_null() {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    if size < 9 {
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }

    _wstrtime(time);
    0
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> clock_t {
    let mut systime: LARGE_INTEGER = core::mem::zeroed();
    NtQuerySystemTime(core::ptr::addr_of_mut!(systime));
    ((systime.QuadPart - init_time) / ((TICKSPERSEC / (CLOCKS_PER_SEC) as i32)) as i64) as i32
}

// `wrapping_sub`, not `-`. The C is `(double)(time1 - time2)`, and the subtraction happens
// in the TIME_T's own width before the promotion — so `_difftime32(INT32_MAX, INT32_MIN)`
// overflows a 32-bit int and `_difftime64` overflows a 64-bit one. C calls that undefined
// and every compiler emits a plain `sub` that wraps; Rust's `-` is a CHECKED subtraction
// that panics in a debug profile, so the literal translation is a live abort inside
// msvcrt.dll. Same class as `_abs64`'s negation in math.rs, and found the same way — by a
// row in the debug profile, which is the only place the check is compiled in.
#[no_mangle]
pub unsafe extern "C" fn _difftime64(time1: __time64_t, time2: __time64_t) -> f64 {
    time1.wrapping_sub(time2) as f64
}

#[no_mangle]
pub unsafe extern "C" fn _difftime32(time1: __time32_t, time2: __time32_t) -> f64 {
    time1.wrapping_sub(time2) as f64
}

#[no_mangle]
pub unsafe extern "C" fn _ftime64(buf: *mut __timeb64) {
    let mut tzinfo: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut ft: FILETIME = core::mem::zeroed();
    let time: u64;

    _tzset_init();

    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(ft));

    time = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);

    (*buf).time = (time / (TICKSPERSEC) as u64) as __time64_t - (SECS_1601_TO_1970) as i64;
    (*buf).millitm = ((time % (TICKSPERSEC) as u64) / (TICKSPERMSEC) as u64) as u16;
    (*buf).timezone = (MSVCRT___timezone / 60) as i16;
    (*buf).dstflag = (GetTimeZoneInformation(core::ptr::addr_of_mut!(tzinfo)) == TIME_ZONE_ID_DAYLIGHT) as i16;
}

#[no_mangle]
pub unsafe extern "C" fn _ftime64_s(buf: *mut __timeb64) -> i32 {
    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    _ftime64(buf);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _ftime32(buf: *mut __timeb32) {
    let mut buf64: __timeb64 = core::mem::zeroed();

    _ftime64(&mut buf64);
    (*buf).time = (buf64.time) as i32;
    (*buf).millitm = buf64.millitm;
    (*buf).timezone = buf64.timezone;
    (*buf).dstflag = buf64.dstflag;
}

#[no_mangle]
pub unsafe extern "C" fn _ftime32_s(buf: *mut __timeb32) -> i32 {
    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    _ftime32(buf);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _time64(buf: *mut __time64_t) -> __time64_t {
    let mut curtime: __time64_t;
    let mut tb: __timeb64 = core::mem::zeroed();

    _ftime64(&mut tb);

    curtime = tb.time;
    if !buf.is_null() {
        *buf = curtime;
        curtime
    } else {
        curtime
    }
}

#[no_mangle]
pub unsafe extern "C" fn _time32(buf: *mut __time32_t) -> __time32_t {
    let mut tb: __timeb64 = core::mem::zeroed();

    _ftime64(&mut tb);

    let curtime = tb.time as __time32_t;
    if !buf.is_null() {
        *buf = curtime;
        curtime
    } else {
        curtime
    }
}

#[no_mangle]
pub unsafe extern "C" fn __p__daylight() -> *mut i32 {
    core::ptr::addr_of_mut!(MSVCRT___daylight)
}

#[no_mangle]
pub unsafe extern "C" fn __p__dstbias() -> *mut __msvcrt_long {
    core::ptr::addr_of_mut!(MSVCRT__dstbias)
}

#[no_mangle]
pub unsafe extern "C" fn __p__timezone() -> *mut __msvcrt_long {
    &mut MSVCRT___timezone
}

#[no_mangle]
pub unsafe extern "C" fn _get_tzname(
    ret: *mut usize,
    buf: *mut c_char,
    bufsize: usize,
    index: i32,
) -> i32 {
    let timezone: *mut c_char;

    match index {
        0 => {
            timezone = tzname_std.as_mut_ptr();
        }
        1 => {
            timezone = tzname_dst.as_mut_ptr();
        }
        _ => {
            *_errno() = (EINVAL) as i32;
            return (EINVAL) as i32;
        }
    }

    if ret.is_null()
        || (buf.is_null() && bufsize > 0)
        || (!buf.is_null() && bufsize == 0)
    {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    *ret = strlen(timezone) + 1;
    if buf.is_null() && bufsize == 0 {
        return 0;
    }
    if *ret > bufsize {
        *buf.offset(0 as isize) = 0;
        return (ERANGE) as i32;
    }

    strcpy(buf, timezone);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn __p__tzname() -> *mut *mut i8 {
    MSVCRT__tzname.as_mut_ptr()
}
unsafe extern "C" fn strftime_nstr(
    str: *mut STRFTIME_CHAR,
    pos: *mut usize,
    max: usize,
    src: *const STRFTIME_CHAR,
    len: usize,
) -> BOOL {
    let mut src_ptr = src;
    let mut remaining = len;

    while *src_ptr != 0 && remaining != 0 {
        if *pos >= max {
            *str = 0;
            *_errno() = (ERANGE) as i32;
            return (FALSE) as i32;
        }

        *str.offset(*pos as isize) = *src_ptr;
        src_ptr = src_ptr.offset(1);
        *pos += 1;
        remaining -= 1;
    }
    (TRUE) as i32
}
unsafe extern "C" fn strftime_int(
    str: *mut STRFTIME_CHAR,
    pos: *mut usize,
    max: usize,
    src: i32,
    prec: i32,
    l: i32,
    h: i32,
) -> BOOL {
    let len: usize;

    if MSVCRT_CHECK_PMT!((src >= l && src <= h)) == 0 {
        *str = 0;
        return (FALSE) as i32;
    }

    len = _snprintf(
        str.offset(*pos as isize),
        max.wrapping_sub(*pos),
        b"%0*d\0".as_ptr() as *const i8,
        prec,
        src,
    ) as usize;
    if len == (-1 as isize) as usize {
        *str = 0;
        *_errno() = (ERANGE) as i32;
        return (FALSE) as i32;
    }

    *pos = (*pos).wrapping_add(len);
    (TRUE) as i32
}
unsafe extern "C" fn strftime_format(
    str: *mut STRFTIME_CHAR,
    pos: *mut usize,
    max: usize,
    mstm: *const tm,
    time_data: *mut __lc_time_data,
    format: *const STRFTIME_CHAR,
) -> BOOL {
    let mut count: usize;
    let mut ret: BOOL = (TRUE) as i32;
    let mut format_ptr = format;

    while *format_ptr != 0 && ret != 0 {
        count = 1;
        // `format[0] == format[count]`, NOT `format[count] == format[count]`. The C counts
        // the RUN of the current picture character (`MM` is one field of width 2, `dddd`
        // is the long weekday); the translation compared an element to ITSELF, which is
        // unconditionally true, so `count` ran away until the read left the mapping. Every
        // caller reaches this through `%c`, `%x` or `%X` — the three conversions that
        // format from `lc_time_data`'s pictures — so `strftime(buf, n, "%c", tm)` hung.
        //
        // It survived the flip because nothing had ever driven a picture conversion: the
        // slot those pictures live in (`locinfo->lc_time_curr`) was NULL in the harness
        // until time.c became a reference TU, so the first row that reached this line is
        // the one that found it.
        while *format_ptr == *format_ptr.offset(count as isize) {
            count += 1;
        }

        match *format_ptr as u8 {
            b'\'' => {
                if count % 2 == 0 {
                    break;
                }

                format_ptr = format_ptr.offset(count as isize);
                count = 0;
                while *format_ptr.offset(count as isize) != 0 && *format_ptr.offset(count as isize) != b'\'' as STRFTIME_CHAR {
                    count += 1;
                }

                ret = strftime_nstr(str, pos, max, format_ptr, count);
                if ret == 0 {
                    return (FALSE) as i32;
                }
                if *format_ptr.offset(count as isize) == b'\'' as STRFTIME_CHAR {
                    count += 1;
                }
            }
            b'd' => {
                if count > 2 {
                    if MSVCRT_CHECK_PMT!(((*mstm).tm_wday >= 0 && (*mstm).tm_wday <= 6)) == 0 {
                        *str = 0;
                        return (FALSE) as i32;
                    }
                }
                match count {
                    1 | 2 => {
                        ret = strftime_int(str, pos, max, (*mstm).tm_mday, if count == 1 { 0 } else { 2 }, 1, 31);
                    }
                    3 => {
                        ret = strftime_str!(str, pos, max, STRFTIME_TD!(time_data, short_wday)[(*mstm).tm_wday as usize]);
                    }
                    _ => {
                        ret = strftime_nstr(str, pos, max, format_ptr, count - 4);
                        if ret != 0 {
                            ret = strftime_str!(str, pos, max, STRFTIME_TD!(time_data, wday)[(*mstm).tm_wday as usize]);
                        }
                    }
                }
            }
            b'M' => {
                if count > 2 {
                    if MSVCRT_CHECK_PMT!(((*mstm).tm_mon >= 0 && (*mstm).tm_mon <= 11)) == 0 {
                        *str = 0;
                        return (FALSE) as i32;
                    }
                }
                match count {
                    1 | 2 => {
                        ret = strftime_int(str, pos, max, (*mstm).tm_mon + 1, if count == 1 { 0 } else { 2 }, 1, 12);
                    }
                    3 => {
                        ret = strftime_str!(str, pos, max, STRFTIME_TD!(time_data, short_mon)[(*mstm).tm_mon as usize]);
                    }
                    _ => {
                        ret = strftime_nstr(str, pos, max, format_ptr, count - 4);
                        if ret != 0 {
                            ret = strftime_str!(str, pos, max, STRFTIME_TD!(time_data, mon)[(*mstm).tm_mon as usize]);
                        }
                    }
                }
            }
            b'y' => {
                if count > 1 {
                    if MSVCRT_CHECK_PMT!(((*mstm).tm_year >= 0)) == 0 {
                        *str = 0;
                        return (FALSE) as i32;
                    }
                }

                match count {
                    1 => {
                        ret = strftime_nstr(str, pos, max, format_ptr, 1);
                    }
                    2 | 3 => {
                        ret = strftime_nstr(str, pos, max, format_ptr, count - 2);
                        if ret != 0 {
                            ret = strftime_int(str, pos, max, ((*mstm).tm_year + 1900) % 100, 2, 0, 99);
                        }
                    }
                    _ => {
                        ret = strftime_nstr(str, pos, max, format_ptr, count - 4);
                        if ret != 0 {
                            ret = strftime_int(str, pos, max, (*mstm).tm_year + 1900, 4, 0, 9999);
                        }
                    }
                }
            }
            b'h' => {
                if MSVCRT_CHECK_PMT!(((*mstm).tm_hour >= 0 && (*mstm).tm_hour <= 23)) == 0 {
                    *str = 0;
                    return (FALSE) as i32;
                }
                if count > 2 {
                    ret = strftime_nstr(str, pos, max, format_ptr, count - 2);
                }
                if ret != 0 {
                    ret = strftime_int(str, pos, max, ((*mstm).tm_hour + 11) % 12 + 1, if count == 1 { 0 } else { 2 }, 1, 12);
                }
            }
            b'H' => {
                if count > 2 {
                    ret = strftime_nstr(str, pos, max, format_ptr, count - 2);
                }
                if ret != 0 {
                    ret = strftime_int(str, pos, max, (*mstm).tm_hour, if count == 1 { 0 } else { 2 }, 0, 23);
                }
            }
            b'm' => {
                if count > 2 {
                    ret = strftime_nstr(str, pos, max, format_ptr, count - 2);
                }
                if ret != 0 {
                    ret = strftime_int(str, pos, max, (*mstm).tm_min, if count == 1 { 0 } else { 2 }, 0, 59);
                }
            }
            b's' => {
                if count > 2 {
                    ret = strftime_nstr(str, pos, max, format_ptr, count - 2);
                }
                if ret != 0 {
                    ret = strftime_int(str, pos, max, (*mstm).tm_sec, if count == 1 { 0 } else { 2 }, 0, MAX_SECONDS);
                }
            }
            b'a' | b'A' | b't' => {
                if MSVCRT_CHECK_PMT!(((*mstm).tm_hour >= 0 && (*mstm).tm_hour <= 23)) == 0 {
                    *str = 0;
                    return (FALSE) as i32;
                }
                ret = strftime_nstr(str, pos, max, if (*mstm).tm_hour < 12 { STRFTIME_TD!(time_data, am) } else { STRFTIME_TD!(time_data, pm) }, if *format_ptr == b't' as STRFTIME_CHAR && count == 1 { 1 } else { SIZE_MAX });
            }
            _ => {
                ret = strftime_nstr(str, pos, max, format_ptr, count);
            }
        }
        format_ptr = format_ptr.offset(count as isize);
    }

    ret
}
unsafe extern "C" fn strftime_impl(
    str: *mut STRFTIME_CHAR,
    max: usize,
    format: *const STRFTIME_CHAR,
    mstm: *const tm,
    time_data: *mut __lc_time_data,
    loc: _locale_t,
) -> usize {
    let mut ret: usize;
    let mut tmp: usize = 0;
    let mut alternate: BOOL;
    let year: i32 = if !mstm.is_null() {
        (*mstm).tm_year + 1900
    } else {
        -1
    };

    if str.is_null() || format.is_null() {
        if !str.is_null() && max != 0 {
            *str = 0;
        }
        *_errno() = (EINVAL) as i32;
        return 0;
    }

    let mut time_data = time_data;
    if time_data.is_null() {
        time_data = if !loc.is_null() {
            (*(*loc).locinfo).lc_time_curr
        } else {
            (*get_locinfo()).lc_time_curr
        };
    }

    ret = 0;
    let mut format = format;
    loop {
        if (*format) == 0 || ret >= max {
            break;
        }

        if *format != '%' as STRFTIME_CHAR {
            if _isleadbyte_l(((*format) as u8) as i32, loc) != 0 {
                *str.add(ret) = *format;
                ret += 1;
                format = format.add(1);
                if ret == max {
                    format = format.add(1);
                    continue;
                }
                if MSVCRT_CHECK_PMT!(*str.add(ret) != 0) == 0 {
                    *str = 0;
                    return 0;
                }
            }
            *str.add(ret) = *format;
            ret += 1;
            format = format.add(1);
            continue;
        }

        format = format.add(1);
        if *format == '#' as STRFTIME_CHAR {
            alternate = (TRUE) as i32;
            format = format.add(1);
        } else {
            alternate = (FALSE) as i32;
        }

        if MSVCRT_CHECK_PMT!(!mstm.is_null()) == 0 {
            *str = 0;
            return 0;
        }

        match *format as u8 as char {
            'c' => {
                if strftime_format(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    mstm,
                    time_data,
                    if alternate != 0 {
                        STRFTIME_TD!(time_data, date)
                    } else {
                        STRFTIME_TD!(time_data, short_date)
                    },
                ) == 0
                {
                    return 0;
                }
                if ret < max {
                    *str.add(ret) = ' ' as STRFTIME_CHAR;
                    ret += 1;
                }
                if strftime_format(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    mstm,
                    time_data,
                    STRFTIME_TD!(time_data, time),
                ) == 0
                {
                    return 0;
                }
            }
            'x' => {
                if strftime_format(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    mstm,
                    time_data,
                    if alternate != 0 {
                        STRFTIME_TD!(time_data, date)
                    } else {
                        STRFTIME_TD!(time_data, short_date)
                    },
                ) == 0
                {
                    return 0;
                }
            }
            'X' => {
                if strftime_format(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    mstm,
                    time_data,
                    STRFTIME_TD!(time_data, time),
                ) == 0
                {
                    return 0;
                }
            }
            'a' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_wday >= 0 && (*mstm).tm_wday <= 6,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_str!(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    STRFTIME_TD_IDX!(time_data, short_wday, (*mstm).tm_wday as usize),
                ) == 0
                {
                    return 0;
                }
            }
            'A' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_wday >= 0 && (*mstm).tm_wday <= 6,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_str!(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    STRFTIME_TD_IDX!(time_data, wday, (*mstm).tm_wday as usize),
                ) == 0
                {
                    return 0;
                }
            }
            'b' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_mon >= 0 && (*mstm).tm_mon <= 11,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_str!(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    STRFTIME_TD_IDX!(time_data, short_mon, (*mstm).tm_mon as usize),
                ) == 0
                {
                    return 0;
                }
            }
            'B' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_mon >= 0 && (*mstm).tm_mon <= 11,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_str!(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    STRFTIME_TD_IDX!(time_data, mon, (*mstm).tm_mon as usize),
                ) == 0
                {
                    return 0;
                }
            }
            'd' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_mday,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    1,
                    31,
                ) == 0
                {
                    return 0;
                }
            }
            'H' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_hour,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    0,
                    23,
                ) == 0
                {
                    return 0;
                }
            }
            'I' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_hour >= 0 && (*mstm).tm_hour <= 23,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    ((*mstm).tm_hour + 11) % 12 + 1,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    1,
                    12,
                ) == 0
                {
                    return 0;
                }
            }
            'j' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_yday + 1,
                    if alternate != 0 {
                        0
                    } else {
                        3
                    },
                    1,
                    366,
                ) == 0
                {
                    return 0;
                }
            }
            'm' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_mon + 1,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    1,
                    12,
                ) == 0
                {
                    return 0;
                }
            }
            'M' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_min,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    0,
                    59,
                ) == 0
                {
                    return 0;
                }
            }
            'p' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_hour >= 0 && (*mstm).tm_hour <= 23,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_str!(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    if (*mstm).tm_hour < 12 {
                        STRFTIME_TD!(time_data, am)
                    } else {
                        STRFTIME_TD!(time_data, pm)
                    },
                ) == 0
                {
                    return 0;
                }
            }
            'S' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_sec,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    0,
                    MAX_SECONDS,
                ) == 0
                {
                    return 0;
                }
            }
            'w' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    (*mstm).tm_wday,
                    0,
                    0,
                    6,
                ) == 0
                {
                    return 0;
                }
            }
            'y' => {
                if MSVCRT_CHECK_PMT!(year >= 1900) == 0 {
                    *str = 0;
                    return 0;
                }
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    year % 100,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    0,
                    99,
                ) == 0
                {
                    return 0;
                }
            }
            'Y' => {
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    year,
                    if alternate != 0 {
                        0
                    } else {
                        4
                    },
                    0,
                    9999,
                ) == 0
                {
                    return 0;
                }
            }
            'z' | 'Z' => {
                _tzset();
                if _get_tzname(
                    core::ptr::addr_of_mut!(tmp),
                    str.add(ret),
                    max - ret,
                    if (*mstm).tm_isdst != 0 {
                        1
                    } else {
                        0
                    },
                ) != 0
                {
                    return 0;
                }
                ret += tmp - 1;
            }
            'U' | 'W' => {
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_wday >= 0 && (*mstm).tm_wday <= 6,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                if MSVCRT_CHECK_PMT!(
                    (*mstm).tm_yday >= 0 && (*mstm).tm_yday <= 365,
                ) == 0 {
                    *str = 0;
                    return 0;
                }
                tmp = if *format as u8 as char == 'U' {
                    (*mstm).tm_wday as usize
                } else if (*mstm).tm_wday == 0 {
                    6
                } else {
                    ((*mstm).tm_wday - 1) as usize
                };

                tmp = ((*mstm).tm_yday as usize) / 7
                    + if tmp <= ((*mstm).tm_yday as usize % 7) {
                        1
                    } else {
                        0
                    };
                if strftime_int(
                    str,
                    core::ptr::addr_of_mut!(ret),
                    max,
                    tmp as i32,
                    if alternate != 0 {
                        0
                    } else {
                        2
                    },
                    0,
                    53,
                ) == 0
                {
                    return 0;
                }
            }
            '%' => {
                *str.add(ret) = '%' as STRFTIME_CHAR;
                ret += 1;
            }
            _ => {
                MSVCRT_INVALID_PMT!(b"unknown format\0".as_ptr() as *const i8, (EINVAL) as i32);
                *str = 0;
                return 0;
            }
        }

        format = format.add(1);
    }

    if ret == max {
        if max != 0 {
            *str = 0;
        }
        *_errno() = (ERANGE) as i32;
        return 0;
    }

    *str.add(ret) = 0;
    ret
}

unsafe extern "C" fn strftime_helper(
    str: *mut i8,
    max: size_t,
    format: *const i8,
    mstm: *const tm,
    time_data: *mut __lc_time_data,
    loc: _locale_t,
) -> size_t {
    // TRACE("(%p %Iu %s %p %p %p)\n", str, max, format, mstm, time_data, loc);
    strftime_impl(str, max, format, mstm, time_data, loc)
}

#[no_mangle]
pub unsafe extern "C" fn _Strftime(
    str: *mut i8,
    max: size_t,
    format: *const i8,
    mstm: *const tm,
    time_data: *mut core::ffi::c_void,
) -> size_t {
    strftime_helper(str, max, format, mstm, (time_data).cast(), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn strftime(
    str: *mut i8,
    max: size_t,
    format: *const i8,
    mstm: *const tm,
) -> size_t {
    strftime_helper(str, max, format, mstm, core::ptr::null_mut(), core::ptr::null_mut())
}
unsafe extern "C" fn wcsftime_helper(
    str: *mut wchar_t,
    max: usize,
    format: *const wchar_t,
    mstm: *const tm,
    time_data: *mut __lc_time_data,
    loc: _locale_t,
) -> usize {
    let mut s: *mut i8;
    let mut fmt: *mut i8;
    let mut len: usize;

    len = _wcstombs_l(core::ptr::null_mut(), format, 0, loc) + 1;
    fmt = (malloc(len)).cast();
    if fmt.is_null() {
        return 0;
    }
    _wcstombs_l(fmt, format, len, loc);

    s = (malloc(max * 4)).cast();
    if !s.is_null() {
        if strftime_impl(s, max * 4, fmt, mstm, time_data, loc) == 0 {
            *s.add(0) = 0;
        }
        len = _mbstowcs_l(str, s, max, loc);
        free((s).cast());
    } else {
        len = 0;
    }

    free((fmt).cast());
    len
}

#[no_mangle]
pub unsafe extern "C" fn _wcsftime_l(
    str: *mut wchar_t,
    max: size_t,
    format: *const wchar_t,
    mstm: *const tm,
    loc: _locale_t,
) -> size_t {
    wcsftime_helper(str, max, format, mstm, core::ptr::null_mut(), loc)
}

#[no_mangle]
pub unsafe extern "C" fn wcsftime(
    str: *mut wchar_t,
    max: size_t,
    format: *const wchar_t,
    mstm: *const tm,
) -> size_t {
    wcsftime_helper(str, max, format, mstm, core::ptr::null_mut(), core::ptr::null_mut())
}
unsafe extern "C" fn asctime_buf(buf: *mut i8, mstm: *const tm) -> *mut i8 {
    static wday: [[i8; 4]; 7] = [
        cstr4(b"Sun"), cstr4(b"Mon"), cstr4(b"Tue"), cstr4(b"Wed"),
        cstr4(b"Thu"), cstr4(b"Fri"), cstr4(b"Sat"),
    ];
    static month: [[i8; 4]; 12] = [
        cstr4(b"Jan"), cstr4(b"Feb"), cstr4(b"Mar"), cstr4(b"Apr"),
        cstr4(b"May"), cstr4(b"Jun"), cstr4(b"Jul"), cstr4(b"Aug"),
        cstr4(b"Sep"), cstr4(b"Oct"), cstr4(b"Nov"), cstr4(b"Dec"),
    ];

    if mstm.is_null() || (*mstm).tm_sec < 0 || (*mstm).tm_sec > 59
            || (*mstm).tm_min < 0 || (*mstm).tm_min > 59
            || (*mstm).tm_hour < 0 || (*mstm).tm_hour > 23
            || (*mstm).tm_mon < 0 || (*mstm).tm_mon > 11
            || (*mstm).tm_wday < 0 || (*mstm).tm_wday > 6
            || (*mstm).tm_year < 0 || (*mstm).tm_mday < 0
            || (*mstm).tm_mday > MonthLengths[IsLeapYear(1900 + (*mstm).tm_year) as usize][(*mstm).tm_mon as usize] {
        *_errno() = (EINVAL) as i32;
        return core::ptr::null_mut();
    }

    _snprintf(buf, 26, b"%s %s %02d %02d:%02d:%02d %c%03d\n\0".as_ptr() as *const i8,
            wday[(*mstm).tm_wday as usize].as_ptr(), month[(*mstm).tm_mon as usize].as_ptr(),
            (*mstm).tm_mday, (*mstm).tm_hour, (*mstm).tm_min,
            (*mstm).tm_sec, '1' as i32 + ((*mstm).tm_year + 900) / 1000,
            (900 + (*mstm).tm_year) % 1000);
    buf
}

#[no_mangle]
pub unsafe extern "C" fn asctime(mstm: *const tm) -> *mut i8 {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    if (*data).asctime_buffer.is_null() {
        (*data).asctime_buffer = (malloc(26)).cast();
        if (*data).asctime_buffer.is_null() {
            *_errno() = (ENOMEM) as i32;
            return core::ptr::null_mut();
        }
    }

    asctime_buf((*data).asctime_buffer, mstm)
}

#[no_mangle]
pub unsafe extern "C" fn asctime_s(time: *mut i8, size: usize, mstm: *const tm) -> i32 {
    if MSVCRT_CHECK_PMT!(time != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if size != 0 {
        *time.offset(0 as isize) = 0;
    }
    if MSVCRT_CHECK_PMT!(size >= 26) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(mstm != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_sec >= 0 && (*mstm).tm_sec < 60) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_min >= 0 && (*mstm).tm_min < 60) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_hour >= 0 && (*mstm).tm_hour < 24) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_mon >= 0 && (*mstm).tm_mon < 12) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_wday >= 0 && (*mstm).tm_wday < 7) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_year >= 0) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_mday >= 0) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((*mstm).tm_mday <= MonthLengths[IsLeapYear(1900 + (*mstm).tm_year) as usize][(*mstm).tm_mon as usize]) == 0 {
        return (EINVAL) as i32;
    }

    asctime_buf(time, mstm);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _wasctime(mstm: *const tm) -> *mut wchar_t {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    let mut buffer: [i8; 26] = [0; 26];

    if (*data).wasctime_buffer.is_null() {
        (*data).wasctime_buffer = malloc(26 * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        if (*data).wasctime_buffer.is_null() {
            *_errno() = (ENOMEM) as i32;
            return core::ptr::null_mut();
        }
    }

    if asctime_buf(buffer.as_mut_ptr(), mstm).is_null() {
        return core::ptr::null_mut();
    }

    MultiByteToWideChar(CP_ACP, 0, buffer.as_ptr(), -1, ((*data).wasctime_buffer as *mut u8).cast(), 26);
    (*data).wasctime_buffer
}

#[no_mangle]
pub unsafe extern "C" fn _wasctime_s(time: *mut wchar_t, size: usize, mstm: *const tm) -> i32 {
    let mut buffer: [i8; 26] = [0; 26];
    let ret: i32;

    if MSVCRT_CHECK_PMT!(time != core::ptr::null_mut()) == 0 { return (EINVAL) as i32; }
    if size != 0 { *time.offset(0) = 0; }
    if MSVCRT_CHECK_PMT!(size >= 26) == 0 { return (EINVAL) as i32; }
    if MSVCRT_CHECK_PMT!(mstm != core::ptr::null()) == 0 { return (EINVAL) as i32; }

    ret = asctime_s(buffer.as_mut_ptr(), core::mem::size_of_val(&buffer), mstm);
    if ret != 0 {
        return ret;
    }
    MultiByteToWideChar(CP_ACP, 0, buffer.as_ptr(), -1, time, size as i32);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _ctime64(time: *const __time64_t) -> *mut i8 {
    let t = _localtime64(time);
    if t.is_null() {
        return core::ptr::null_mut();
    }
    return asctime(t);
}

#[no_mangle]
pub unsafe extern "C" fn _ctime64_s(res: *mut i8, len: usize, time: *const __time64_t) -> errno_t {
    let mut t: tm = core::mem::zeroed();
    let ret: i32;

    if MSVCRT_CHECK_PMT!(res != core::ptr::null_mut()) == 0 { return (EINVAL) as i32; }
    if MSVCRT_CHECK_PMT!(len >= 26) == 0 { return (EINVAL) as i32; }
    *res.offset(0 as isize) = b'\0' as i8;
    if MSVCRT_CHECK_PMT!(time != core::ptr::null()) == 0 { return (EINVAL) as i32; }
    if MSVCRT_CHECK_PMT!(*time > 0) == 0 { return (EINVAL) as i32; }

    ret = _localtime64_s(&mut t, time);
    if ret != 0 {
        return ret;
    }
    return asctime_s(res, len, &t);
}

#[no_mangle]
pub unsafe extern "C" fn _ctime32(time: *const __time32_t) -> *mut i8 {
    let t = _localtime32(time);
    if t.is_null() {
        return core::ptr::null_mut();
    }
    asctime(t)
}

#[no_mangle]
pub unsafe extern "C" fn _ctime32_s(res: *mut i8, len: usize, time: *const __time32_t) -> errno_t {
    let mut t: tm = core::mem::zeroed();
    let ret: i32;

    if MSVCRT_CHECK_PMT!(res != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(len >= 26) == 0 {
        return (EINVAL) as i32;
    }
    *res.offset(0 as isize) = b'\0' as i8;
    if MSVCRT_CHECK_PMT!(time != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(*time > 0) == 0 {
        return (EINVAL) as i32;
    }

    ret = _localtime32_s(core::ptr::addr_of_mut!(t), time);
    if ret != 0 {
        return ret;
    }
    return asctime_s(res, len, core::ptr::addr_of!(t));
}

#[no_mangle]
pub unsafe extern "C" fn _wctime64(time: *const __time64_t) -> *mut wchar_t {
    return _wasctime(_localtime64(time));
}

#[no_mangle]
pub unsafe extern "C" fn _wctime32(time: *const __time32_t) -> *mut wchar_t {
    return _wasctime(_localtime32(time));
}

#[no_mangle]
pub unsafe extern "C" fn _wctime64_s(
    buf: *mut wchar_t,
    size: usize,
    time: *const __time64_t,
) -> i32 {
    let mut tm: tm = core::mem::zeroed();
    let mut ret: i32;

    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(size != 0) == 0 {
        return (EINVAL) as i32;
    }
    *buf.offset(0 as isize) = 0;
    if MSVCRT_CHECK_PMT!(time != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(*time >= 0) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(*time <= (_MAX__TIME64_T) as i64) == 0 {
        return (EINVAL) as i32;
    }

    ret = _localtime64_s(&mut tm, time);
    if ret != 0 {
        return ret;
    }

    return _wasctime_s(buf, size, &tm);
}
#[no_mangle]
pub unsafe extern "C" fn _wctime32_s(buf: *mut wchar_t, size: usize,
        time: *const __time32_t) -> i32 {
    let mut tm: tm = core::mem::zeroed();
    let ret: i32;

    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 { return (EINVAL) as i32; }
    if MSVCRT_CHECK_PMT!(size != 0) == 0 { return (EINVAL) as i32; }
    *buf.offset(0) = 0;
    if MSVCRT_CHECK_PMT!(time != core::ptr::null()) == 0 { return (EINVAL) as i32; }
    if MSVCRT_CHECK_PMT!(*time >= 0) == 0 { return (EINVAL) as i32; }

    ret = _localtime32_s(&mut tm, time);
    if ret != 0 {
        return ret;
    }

    return _wasctime_s(buf, size, &tm);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll date/time functions
//  *
//  * Copyright 1996,1998 Marcus Meissner
//  * Copyright 1996 Jukka Iivonen
//  * Copyright 1997,2000 Uwe Bonnes
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2004 Hans Leidekker
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
// #include <locale.h>
// #include <stdlib.h>
// #include <time.h>
// #include <sys/timeb.h>
//
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "winbase.h"
// #include "winnls.h"
// #include "winternl.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// #undef _ctime32
// #undef _difftime32
// #undef _gmtime32
// #undef _localtime32
// #undef _mktime32
// #undef _time32
// #undef _wctime32
//
// static LONGLONG init_time;
//
// void msvcrt_init_clock(void)
// {
//     LARGE_INTEGER systime;
//
//     NtQuerySystemTime(&systime);
//     init_time = systime.QuadPart;
// }
//
// static const int MonthLengths[2][12] =
// {
//     { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 },
//     { 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 }
// };
//
// static const int MAX_SECONDS = 59;
//
// #if _MSVCR_VER == 0
// #define MIN_GMTIME64_TIME 0
// #define MAX_GMTIME64_TIME _MAX__TIME64_T
// #elif _MSVCR_VER >= 140
// #define MIN_GMTIME64_TIME -43200
// #define MAX_GMTIME64_TIME (_MAX__TIME64_T + 1605600)
// #else
// #define MIN_GMTIME64_TIME -43200
// #define MAX_GMTIME64_TIME (_MAX__TIME64_T + 46800)
// #endif
//
// static inline BOOL IsLeapYear(int Year)
// {
//     return Year % 4 == 0 && (Year % 100 != 0 || Year % 400 == 0);
// }
//
// static inline void write_invalid_msvcrt_tm( struct tm *tm )
// {
//     tm->tm_sec = -1;
//     tm->tm_min = -1;
//     tm->tm_hour = -1;
//     tm->tm_mday = -1;
//     tm->tm_mon = -1;
//     tm->tm_year = -1;
//     tm->tm_wday = -1;
//     tm->tm_yday = -1;
//     tm->tm_isdst = -1;
// }
//
// /*********************************************************************
//  *		_daylight (MSVCRT.@)
//  */
// int MSVCRT___daylight = 1;
//
// /*********************************************************************
//  *		_timezone (MSVCRT.@)
//  */
// __msvcrt_long MSVCRT___timezone = 28800;
//
// /*********************************************************************
//  *		_dstbias (MSVCRT.@)
//  */
// __msvcrt_long MSVCRT__dstbias = -3600;
//
// /*********************************************************************
//  *		_tzname (MSVCRT.@)
//  * NOTES
//  *  Some apps (notably Mozilla) insist on writing to these, so the buffer
//  *  must be large enough.
//  */
// static char tzname_std[64] = "PST";
// static char tzname_dst[64] = "PDT";
// char *MSVCRT__tzname[2] = { tzname_std, tzname_dst };
//
// static TIME_ZONE_INFORMATION tzi = {0};
// /*********************************************************************
//  *		_tzset (MSVCRT.@)
//  */
// void CDECL _tzset(void)
// {
//     char *tz = getenv("TZ");
//     BOOL error;
//
//     _lock(_TIME_LOCK);
//     if(tz && tz[0]) {
//         BOOL neg_zone = FALSE;
//
//         memset(&tzi, 0, sizeof(tzi));
//
//         /* Parse timezone information: tzn[+|-]hh[:mm[:ss]][dzn] */
//         lstrcpynA(MSVCRT__tzname[0], tz, 3);
//         tz += 3;
//
//         if(*tz == '-') {
//             neg_zone = TRUE;
//             tz++;
//         }else if(*tz == '+') {
//             tz++;
//         }
//         MSVCRT___timezone = strtol(tz, &tz, 10)*3600;
//         if(*tz == ':') {
//             MSVCRT___timezone += strtol(tz+1, &tz, 10)*60;
//             if(*tz == ':')
//                 MSVCRT___timezone += strtol(tz+1, &tz, 10);
//         }
//         if(neg_zone)
//             MSVCRT___timezone = -MSVCRT___timezone;
//
//         MSVCRT___daylight = *tz;
//         lstrcpynA(MSVCRT__tzname[1], tz, 3);
//     }else if(GetTimeZoneInformation(&tzi) != TIME_ZONE_ID_INVALID) {
//         MSVCRT___timezone = tzi.Bias*60;
//         if(tzi.StandardDate.wMonth)
//             MSVCRT___timezone += tzi.StandardBias*60;
//
//         if(tzi.DaylightDate.wMonth) {
//             MSVCRT___daylight = 1;
//             MSVCRT__dstbias = (tzi.DaylightBias-tzi.StandardBias)*60;
//         }else {
//             MSVCRT___daylight = 0;
//             MSVCRT__dstbias = 0;
//         }
//
//         if(!WideCharToMultiByte(CP_ACP, 0, tzi.StandardName, -1, MSVCRT__tzname[0],
//                     sizeof(tzname_std), NULL, &error) || error)
//             *MSVCRT__tzname[0] = 0;
//         if(!WideCharToMultiByte(CP_ACP, 0, tzi.DaylightName, -1, MSVCRT__tzname[1],
//                     sizeof(tzname_dst), NULL, &error) || error)
//             *MSVCRT__tzname[0] = 0;
//     }
//     _unlock(_TIME_LOCK);
// }
//
// static void _tzset_init(void)
// {
//     static BOOL init = FALSE;
//
//     if(!init) {
//         _lock(_TIME_LOCK);
//         if(!init) {
//             _tzset();
//             init = TRUE;
//         }
//         _unlock(_TIME_LOCK);
//     }
// }
//
// static BOOL is_dst(const SYSTEMTIME *st)
// {
//     TIME_ZONE_INFORMATION tmp;
//     SYSTEMTIME out;
//
//     if(!MSVCRT___daylight)
//         return FALSE;
//
//     if(tzi.DaylightDate.wMonth) {
//         tmp = tzi;
//     }else if(st->wYear >= 2007) {
//         memset(&tmp, 0, sizeof(tmp));
//         tmp.StandardDate.wMonth = 11;
//         tmp.StandardDate.wDay = 1;
//         tmp.StandardDate.wHour = 2;
//         tmp.DaylightDate.wMonth = 3;
//         tmp.DaylightDate.wDay = 2;
//         tmp.DaylightDate.wHour = 2;
//     }else {
//         memset(&tmp, 0, sizeof(tmp));
//         tmp.StandardDate.wMonth = 10;
//         tmp.StandardDate.wDay = 5;
//         tmp.StandardDate.wHour = 2;
//         tmp.DaylightDate.wMonth = 4;
//         tmp.DaylightDate.wDay = 1;
//         tmp.DaylightDate.wHour = 2;
//     }
//
//     tmp.Bias = 0;
//     tmp.StandardBias = 0;
//     tmp.DaylightBias = MSVCRT__dstbias/60;
//     if(!SystemTimeToTzSpecificLocalTime(&tmp, st, &out))
//         return FALSE;
//
//     return memcmp(st, &out, sizeof(SYSTEMTIME));
// }
//
// #define SECSPERDAY        86400
// /* 1601 to 1970 is 369 years plus 89 leap days */
// #define SECS_1601_TO_1970  ((369 * 365 + 89) * (ULONGLONG)SECSPERDAY)
// #define TICKSPERSEC       10000000
// #define TICKSPERMSEC      10000
// #define TICKS_1601_TO_1970 (SECS_1601_TO_1970 * TICKSPERSEC)
//
// static __time64_t mktime_helper(struct tm *mstm, BOOL local)
// {
//     SYSTEMTIME st;
//     FILETIME ft;
//     __time64_t ret = 0;
//     int i;
//     BOOL use_dst = FALSE;
//
//     ret = mstm->tm_year + mstm->tm_mon/12;
//     mstm->tm_mon %= 12;
//     if(mstm->tm_mon < 0) {
//         mstm->tm_mon += 12;
//         ret--;
//     }
//
//     if(ret<70 || ret>1100) {
//         *_errno() = EINVAL;
//         return -1;
//     }
//
//     memset(&st, 0, sizeof(SYSTEMTIME));
//     st.wDay = 1;
//     st.wMonth = mstm->tm_mon+1;
//     st.wYear = ret+1900;
//
//     if(!SystemTimeToFileTime(&st, &ft)) {
//         *_errno() = EINVAL;
//         return -1;
//     }
//
//     ret = ((__time64_t)ft.dwHighDateTime<<32)+ft.dwLowDateTime;
//     ret += (__time64_t)mstm->tm_sec*TICKSPERSEC;
//     ret += (__time64_t)mstm->tm_min*60*TICKSPERSEC;
//     ret += (__time64_t)mstm->tm_hour*60*60*TICKSPERSEC;
//     ret += (__time64_t)(mstm->tm_mday-1)*SECSPERDAY*TICKSPERSEC;
//
//     ft.dwLowDateTime = ret & 0xffffffff;
//     ft.dwHighDateTime = ret >> 32;
//     FileTimeToSystemTime(&ft, &st);
//
//     if(local) {
//         _tzset_init();
//         use_dst = is_dst(&st);
//         if((mstm->tm_isdst<=-1 && use_dst) || (mstm->tm_isdst>=1)) {
//             SYSTEMTIME tmp;
//
//             ret += (__time64_t)MSVCRT__dstbias*TICKSPERSEC;
//
//             ft.dwLowDateTime = ret & 0xffffffff;
//             ft.dwHighDateTime = ret >> 32;
//             FileTimeToSystemTime(&ft, &tmp);
//
//             if(!is_dst(&tmp)) {
//                 st = tmp;
//                 use_dst = FALSE;
//             }else {
//                 use_dst = TRUE;
//             }
//         }else if(mstm->tm_isdst==0 && use_dst) {
//             ret -= (__time64_t)MSVCRT__dstbias*TICKSPERSEC;
//             ft.dwLowDateTime = ret & 0xffffffff;
//             ft.dwHighDateTime = ret >> 32;
//             FileTimeToSystemTime(&ft, &st);
//             ret += (__time64_t)MSVCRT__dstbias*TICKSPERSEC;
//         }
//         ret += (__time64_t)MSVCRT___timezone*TICKSPERSEC;
//     }
//
//     mstm->tm_sec = st.wSecond;
//     mstm->tm_min = st.wMinute;
//     mstm->tm_hour = st.wHour;
//     mstm->tm_mday = st.wDay;
//     mstm->tm_mon = st.wMonth-1;
//     mstm->tm_year = st.wYear-1900;
//     mstm->tm_wday = st.wDayOfWeek;
//     for(i=mstm->tm_yday=0; i<st.wMonth-1; i++)
//         mstm->tm_yday += MonthLengths[IsLeapYear(st.wYear)][i];
//     mstm->tm_yday += st.wDay-1;
//     mstm->tm_isdst = use_dst ? 1 : 0;
//
//     if(ret < TICKS_1601_TO_1970) {
//         *_errno() = EINVAL;
//         return -1;
//     }
//     ret = (ret-TICKS_1601_TO_1970)/TICKSPERSEC;
//     return ret;
// }
//
// /**********************************************************************
//  *		_mktime64 (MSVCRT.@)
//  */
// __time64_t CDECL _mktime64(struct tm *mstm)
// {
//     return mktime_helper(mstm, TRUE);
// }
//
// /**********************************************************************
//  *		_mktime32 (MSVCRT.@)
//  */
// __time32_t CDECL _mktime32(struct tm *mstm)
// {
//     __time64_t ret = _mktime64( mstm );
//     return ret == (__time32_t)ret ? ret : -1;
// }
//
// /**********************************************************************
//  *		_mkgmtime64 (MSVCRT.@)
//  *
//  * time->tm_isdst value is ignored
//  */
// __time64_t CDECL _mkgmtime64(struct tm *time)
// {
//     return mktime_helper(time, FALSE);
// }
//
// /**********************************************************************
//  *		_mkgmtime32 (MSVCRT.@)
//  */
// __time32_t CDECL _mkgmtime32(struct tm *time)
// {
//     __time64_t ret = _mkgmtime64(time);
//     return ret == (__time32_t)ret ? ret : -1;
// }
//
// /*********************************************************************
//  *      _localtime64_s (MSVCRT.@)
//  */
// int CDECL _localtime64_s(struct tm *res, const __time64_t *secs)
// {
//     int i;
//     FILETIME ft;
//     SYSTEMTIME st;
//     ULONGLONG time;
//
//     if (!res || !secs || *secs < 0 || *secs > _MAX__TIME64_T)
//     {
//         if (res)
//             write_invalid_msvcrt_tm(res);
//
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     _tzset_init();
//     time = (*secs - MSVCRT___timezone) * (ULONGLONG)TICKSPERSEC + TICKS_1601_TO_1970;
//
//     ft.dwHighDateTime = (UINT)(time >> 32);
//     ft.dwLowDateTime  = (UINT)time;
//     FileTimeToSystemTime(&ft, &st);
//
//     res->tm_isdst = is_dst(&st) ? 1 : 0;
//     if(res->tm_isdst) {
//         time -= MSVCRT__dstbias * (ULONGLONG)TICKSPERSEC;
//         ft.dwHighDateTime = (UINT)(time >> 32);
//         ft.dwLowDateTime  = (UINT)time;
//         FileTimeToSystemTime(&ft, &st);
//     }
//
//     res->tm_sec  = st.wSecond;
//     res->tm_min  = st.wMinute;
//     res->tm_hour = st.wHour;
//     res->tm_mday = st.wDay;
//     res->tm_year = st.wYear - 1900;
//     res->tm_mon  = st.wMonth - 1;
//     res->tm_wday = st.wDayOfWeek;
//     for (i = res->tm_yday = 0; i < st.wMonth - 1; i++)
//         res->tm_yday += MonthLengths[IsLeapYear(st.wYear)][i];
//     res->tm_yday += st.wDay - 1;
//
//     return 0;
// }
//
// /*********************************************************************
//  *      _localtime64 (MSVCRT.@)
//  */
// struct tm*  _localtime64(const __time64_t* secs)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     if(!data->time_buffer)
//         data->time_buffer = malloc(sizeof(struct tm));
//
//     if(_localtime64_s(data->time_buffer, secs))
//         return NULL;
//     return data->time_buffer;
// }
//
// /*********************************************************************
//  *      _localtime32 (MSVCRT.@)
//  */
// struct tm*  _localtime32(const __time32_t* secs)
// {
//     __time64_t secs64;
//
//     if(!secs)
//         return NULL;
//
//     secs64 = *secs;
//     return _localtime64( &secs64 );
// }
//
// /*********************************************************************
//  *      _localtime32_s (MSVCRT.@)
//  */
// int CDECL _localtime32_s(struct tm *time, const __time32_t *secs)
// {
//     __time64_t secs64;
//
//     if (!time || !secs || *secs < 0)
//     {
//         if (time)
//             write_invalid_msvcrt_tm(time);
//
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     secs64 = *secs;
//     return _localtime64_s(time, &secs64);
// }
//
// /*********************************************************************
//  *      _gmtime64 (MSVCRT.@)
//  */
// int CDECL _gmtime64_s(struct tm *res, const __time64_t *secs)
// {
//     int i;
//     FILETIME ft;
//     SYSTEMTIME st;
//     ULONGLONG time;
//
//     TRACE("res %p, secs %p (%I64d).\n", res, secs, secs ? *secs : 0);
//
//     if (!res || !secs || *secs < MIN_GMTIME64_TIME || *secs > MAX_GMTIME64_TIME) {
//         if (res) {
//             write_invalid_msvcrt_tm(res);
//         }
//
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     time = *secs * (ULONGLONG)TICKSPERSEC + TICKS_1601_TO_1970;
//
//     ft.dwHighDateTime = (UINT)(time >> 32);
//     ft.dwLowDateTime  = (UINT)time;
//
//     FileTimeToSystemTime(&ft, &st);
//
//     res->tm_sec  = st.wSecond;
//     res->tm_min  = st.wMinute;
//     res->tm_hour = st.wHour;
//     res->tm_mday = st.wDay;
//     res->tm_year = st.wYear - 1900;
//     res->tm_mon  = st.wMonth - 1;
//     res->tm_wday = st.wDayOfWeek;
//     for (i = res->tm_yday = 0; i < st.wMonth - 1; i++) {
//         res->tm_yday += MonthLengths[IsLeapYear(st.wYear)][i];
//     }
//
//     res->tm_yday += st.wDay - 1;
//     res->tm_isdst = 0;
//
//     return 0;
// }
//
// /*********************************************************************
//  *      _gmtime64 (MSVCRT.@)
//  */
// struct tm*  _gmtime64(const __time64_t *secs)
// {
//     thread_data_t * const data = msvcrt_get_thread_data();
//
//     if(!data->time_buffer)
//         data->time_buffer = malloc(sizeof(struct tm));
//
//     if(_gmtime64_s(data->time_buffer, secs))
//         return NULL;
//     return data->time_buffer;
// }
//
// /*********************************************************************
//  *      _gmtime32_s (MSVCRT.@)
//  */
// int CDECL _gmtime32_s(struct tm *res, const __time32_t *secs)
// {
//     __time64_t secs64;
//
//     if(secs) {
//         secs64 = *secs;
//         return _gmtime64_s(res, &secs64);
//     }
//     return _gmtime64_s(res, NULL);
// }
//
// /*********************************************************************
//  *      _gmtime32 (MSVCRT.@)
//  */
// struct tm*  _gmtime32(const __time32_t* secs)
// {
//     __time64_t secs64;
//
//     if(!secs)
//         return NULL;
//
//     secs64 = *secs;
//     return _gmtime64( &secs64 );
// }
//
// /**********************************************************************
//  *		_strdate (MSVCRT.@)
//  */
// char*  _strdate(char* date)
// {
//     SYSTEMTIME st;
//
//     GetLocalTime(&st);
//     date[0] = '0' + st.wMonth / 10;
//     date[1] = '0' + st.wMonth % 10;
//     date[2] = '/';
//     date[3] = '0' + st.wDay / 10;
//     date[4] = '0' + st.wDay % 10;
//     date[5] = '/';
//     date[6] = '0' + st.wYear / 10 % 10;
//     date[7] = '0' + st.wYear % 10;
//     date[8] = 0;
//     return date;
// }
//
// /**********************************************************************
//  *              _strdate_s (MSVCRT.@)
//  */
// int CDECL _strdate_s(char* date, size_t size)
// {
//     if(date && size)
//         date[0] = '\0';
//
//     if(!date) {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if(size < 9) {
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//
//     _strdate(date);
//     return 0;
// }
//
// /**********************************************************************
//  *		_wstrdate (MSVCRT.@)
//  */
// wchar_t*  _wstrdate(wchar_t* date)
// {
//     GetDateFormatW(LOCALE_NEUTRAL, 0, NULL, L"MM'/'dd'/'yy", date, 9);
//     return date;
// }
//
// /**********************************************************************
//  *              _wstrdate_s (MSVCRT.@)
//  */
// int CDECL _wstrdate_s(wchar_t* date, size_t size)
// {
//     if(date && size)
//         date[0] = '\0';
//
//     if(!date) {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if(size < 9) {
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//
//     _wstrdate(date);
//     return 0;
// }
//
// /*********************************************************************
//  *		_strtime (MSVCRT.@)
//  */
// char*  _strtime(char* time)
// {
//     SYSTEMTIME st;
//
//     GetLocalTime(&st);
//     time[0] = '0' + st.wHour / 10;
//     time[1] = '0' + st.wHour % 10;
//     time[2] = ':';
//     time[3] = '0' + st.wMinute / 10;
//     time[4] = '0' + st.wMinute % 10;
//     time[5] = ':';
//     time[6] = '0' + st.wSecond / 10;
//     time[7] = '0' + st.wSecond % 10;
//     time[8] = 0;
//     return time;
// }
//
// /*********************************************************************
//  *              _strtime_s (MSVCRT.@)
//  */
// int CDECL _strtime_s(char* time, size_t size)
// {
//     if(time && size)
//         time[0] = '\0';
//
//     if(!time) {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if(size < 9) {
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//
//     _strtime(time);
//     return 0;
// }
//
// /*********************************************************************
//  *		_wstrtime (MSVCRT.@)
//  */
// wchar_t*  _wstrtime(wchar_t* time)
// {
//     GetTimeFormatW(LOCALE_NEUTRAL, 0, NULL, L"HH':'mm':'ss", time, 9);
//     return time;
// }
//
// /*********************************************************************
//  *              _wstrtime_s (MSVCRT.@)
//  */
// int CDECL _wstrtime_s(wchar_t* time, size_t size)
// {
//     if(time && size)
//         time[0] = '\0';
//
//     if(!time) {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if(size < 9) {
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//
//     _wstrtime(time);
//     return 0;
// }
//
// /*********************************************************************
//  *		clock (MSVCRT.@)
//  */
// clock_t CDECL clock(void)
// {
//     LARGE_INTEGER systime;
//
//     NtQuerySystemTime(&systime);
//     return (systime.QuadPart - init_time) / (TICKSPERSEC / CLOCKS_PER_SEC);
// }
//
// /*********************************************************************
//  *		_difftime64 (MSVCRT.@)
//  */
// double CDECL _difftime64(__time64_t time1, __time64_t time2)
// {
//   return (double)(time1 - time2);
// }
//
// /*********************************************************************
//  *		_difftime32 (MSVCRT.@)
//  */
// double CDECL _difftime32(__time32_t time1, __time32_t time2)
// {
//   return (double)(time1 - time2);
// }
//
// /*********************************************************************
//  *		_ftime64 (MSVCRT.@)
//  */
// void CDECL _ftime64(struct __timeb64 *buf)
// {
//   TIME_ZONE_INFORMATION tzinfo;
//   FILETIME ft;
//   ULONGLONG time;
//
//   _tzset_init();
//
//   GetSystemTimeAsFileTime(&ft);
//
//   time = ((ULONGLONG)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
//
//   buf->time = time / TICKSPERSEC - SECS_1601_TO_1970;
//   buf->millitm = (time % TICKSPERSEC) / TICKSPERMSEC;
//   buf->timezone = MSVCRT___timezone / 60;
//   buf->dstflag = GetTimeZoneInformation( &tzinfo) == TIME_ZONE_ID_DAYLIGHT;
// }
//
// /*********************************************************************
//  *		_ftime64_s (MSVCRT.@)
//  */
// int CDECL _ftime64_s(struct __timeb64 *buf)
// {
//     if (!MSVCRT_CHECK_PMT( buf != NULL )) return EINVAL;
//     _ftime64(buf);
//     return 0;
// }
//
// /*********************************************************************
//  *		_ftime32 (MSVCRT.@)
//  */
// void CDECL _ftime32(struct __timeb32 *buf)
// {
//     struct __timeb64 buf64;
//
//     _ftime64( &buf64 );
//     buf->time     = buf64.time;
//     buf->millitm  = buf64.millitm;
//     buf->timezone = buf64.timezone;
//     buf->dstflag  = buf64.dstflag;
// }
//
// /*********************************************************************
//  *		_ftime32_s (MSVCRT.@)
//  */
// int CDECL _ftime32_s(struct __timeb32 *buf)
// {
//     if (!MSVCRT_CHECK_PMT( buf != NULL )) return EINVAL;
//     _ftime32(buf);
//     return 0;
// }
//
// /*********************************************************************
//  *		_time64 (MSVCRT.@)
//  */
// __time64_t CDECL _time64(__time64_t *buf)
// {
//     __time64_t curtime;
//     struct __timeb64 tb;
//
//     _ftime64(&tb);
//
//     curtime = tb.time;
//     return buf ? *buf = curtime : curtime;
// }
//
// /*********************************************************************
//  *		_time32 (MSVCRT.@)
//  */
// __time32_t CDECL _time32(__time32_t *buf)
// {
//     __time32_t curtime;
//     struct __timeb64 tb;
//
//     _ftime64(&tb);
//
//     curtime = tb.time;
//     return buf ? *buf = curtime : curtime;
// }
//
// /*********************************************************************
//  *		__p__daylight (MSVCRT.@)
//  */
// int *  __p__daylight(void)
// {
// 	return &MSVCRT___daylight;
// }
//
// /*********************************************************************
//  *		__p__dstbias (MSVCRT.@)
//  */
// __msvcrt_long *  __p__dstbias(void)
// {
//     return &MSVCRT__dstbias;
// }
//
//
// /*********************************************************************
//  *		__p__timezone (MSVCRT.@)
//  */
// __msvcrt_long *  __p__timezone(void)
// {
// 	return &MSVCRT___timezone;
// }
//
// /*********************************************************************
//  *		_get_tzname (MSVCRT.@)
//  */
// int CDECL _get_tzname(size_t *ret, char *buf, size_t bufsize, int index)
// {
//     char *timezone;
//
//     switch(index)
//     {
//     case 0:
//         timezone = tzname_std;
//         break;
//     case 1:
//         timezone = tzname_dst;
//         break;
//     default:
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if(!ret || (!buf && bufsize > 0) || (buf && !bufsize))
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     *ret = strlen(timezone)+1;
//     if(!buf && !bufsize)
//         return 0;
//     if(*ret > bufsize)
//     {
//         buf[0] = 0;
//         return ERANGE;
//     }
//
//     strcpy(buf, timezone);
//     return 0;
// }
//
// /*********************************************************************
//  *		__p_tzname (MSVCRT.@)
//  */
// char **  __p__tzname(void)
// {
// 	return MSVCRT__tzname;
// }
//
// #define STRFTIME_CHAR char
// #define STRFTIME_TD(td, name) td->str.names.name
//
// #define strftime_str(a,b,c,d) strftime_nstr(a,b,c,d,SIZE_MAX)
// static inline BOOL strftime_nstr(STRFTIME_CHAR *str, size_t *pos,
//         size_t max, const STRFTIME_CHAR *src, size_t len)
// {
//     while(*src && len)
//     {
//         if(*pos >= max) {
//             *str = 0;
//             *_errno() = ERANGE;
//             return FALSE;
//         }
//
//         str[*pos] = *src;
//         src++;
//         *pos += 1;
//         len--;
//     }
//     return TRUE;
// }
//
// static inline BOOL strftime_int(STRFTIME_CHAR *str, size_t *pos, size_t max,
//         int src, int prec, int l, int h)
// {
//     size_t len;
//
//     if(!MSVCRT_CHECK_PMT(src>=l && src<=h)) {
//         *str = 0;
//         return FALSE;
//     }
//
//     len = _snprintf(str+*pos, max-*pos, "%0*d", prec, src);
//     if(len == -1) {
//         *str = 0;
//         *_errno() = ERANGE;
//         return FALSE;
//     }
//
//     *pos += len;
//     return TRUE;
// }
//
// static inline BOOL strftime_format(STRFTIME_CHAR *str, size_t *pos, size_t max,
//         const struct tm *mstm, __lc_time_data *time_data, const STRFTIME_CHAR *format)
// {
//     size_t count;
//     BOOL ret = TRUE;
//
//     while(*format && ret)
//     {
//         count = 1;
//         while(format[0] == format[count]) count++;
//
//         switch(*format) {
//         case '\'':
//             if(count % 2 == 0) break;
//
//             format += count;
//             count = 0;
//             while(format[count] && format[count] != '\'') count++;
//
//             ret = strftime_nstr(str, pos, max, format, count);
//             if(!ret) return FALSE;
//             if(format[count] == '\'') count++;
//             break;
//         case 'd':
//             if(count > 2)
//             {
//                 if(!MSVCRT_CHECK_PMT(mstm->tm_wday>=0 && mstm->tm_wday<=6))
//                 {
//                     *str = 0;
//                     return FALSE;
//                 }
//             }
//             switch(count) {
//             case 1:
//             case 2:
//                 ret = strftime_int(str, pos, max, mstm->tm_mday, count==1 ? 0 : 2, 1, 31);
//                 break;
//             case 3:
//                 ret = strftime_str(str, pos, max, STRFTIME_TD(time_data, short_wday)[mstm->tm_wday]);
//                 break;
//             default:
//                 ret = strftime_nstr(str, pos, max, format, count-4);
//                 if(ret)
//                     ret = strftime_str(str, pos, max, STRFTIME_TD(time_data, wday)[mstm->tm_wday]);
//                 break;
//             }
//             break;
//         case 'M':
//             if(count > 2)
//             {
//                 if(!MSVCRT_CHECK_PMT(mstm->tm_mon>=0 && mstm->tm_mon<=11))
//                 {
//                     *str = 0;
//                     return FALSE;
//                 }
//             }
//             switch(count) {
//             case 1:
//             case 2:
//                 ret = strftime_int(str, pos, max, mstm->tm_mon+1, count==1 ? 0 : 2, 1, 12);
//                 break;
//             case 3:
//                 ret = strftime_str(str, pos, max, STRFTIME_TD(time_data, short_mon)[mstm->tm_mon]);
//                 break;
//             default:
//                 ret = strftime_nstr(str, pos, max, format, count-4);
//                 if(ret)
//                     ret = strftime_str(str, pos, max, STRFTIME_TD(time_data, mon)[mstm->tm_mon]);
//                 break;
//             }
//             break;
//         case 'y':
//             if(count > 1)
//             {
//                 if(!MSVCRT_CHECK_PMT(mstm->tm_year >= 0))
//                 {
//                     *str = 0;
//                     return FALSE;
//                 }
//             }
//
//             switch(count) {
//             case 1:
//                 ret = strftime_nstr(str, pos, max, format, 1);
//                 break;
//             case 2:
//             case 3:
//                 ret = strftime_nstr(str, pos, max, format, count-2);
//                 if(ret)
//                     ret = strftime_int(str, pos, max, (mstm->tm_year+1900)%100, 2, 0, 99);
//                 break;
//             default:
//                 ret = strftime_nstr(str, pos, max, format, count-4);
//                 if(ret)
//                     ret = strftime_int(str, pos, max, mstm->tm_year+1900, 4, 0, 9999);
//                 break;
//             }
//             break;
//         case 'h':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_hour>=0 && mstm->tm_hour<=23))
//             {
//                 *str = 0;
//                 return FALSE;
//             }
//             if(count > 2)
//                 ret = strftime_nstr(str, pos, max, format, count-2);
//             if(ret)
//                 ret = strftime_int(str, pos, max, (mstm->tm_hour + 11) % 12 + 1,
//                         count == 1 ? 0 : 2, 1, 12);
//             break;
//         case 'H':
//             if(count > 2)
//                 ret = strftime_nstr(str, pos, max, format, count-2);
//             if(ret)
//                 ret = strftime_int(str, pos, max, mstm->tm_hour, count == 1 ? 0 : 2, 0, 23);
//             break;
//         case 'm':
//             if(count > 2)
//                 ret = strftime_nstr(str, pos, max, format, count-2);
//             if(ret)
//                 ret = strftime_int(str, pos, max, mstm->tm_min, count == 1 ? 0 : 2, 0, 59);
//             break;
//         case 's':
//             if(count > 2)
//                 ret = strftime_nstr(str, pos, max, format, count-2);
//             if(ret)
//                 ret = strftime_int(str, pos, max, mstm->tm_sec, count == 1 ? 0 : 2, 0, MAX_SECONDS);
//             break;
//         case 'a':
//         case 'A':
//         case 't':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_hour>=0 && mstm->tm_hour<=23))
//             {
//                 *str = 0;
//                 return FALSE;
//             }
//             ret = strftime_nstr(str, pos, max,
//                     mstm->tm_hour < 12 ? STRFTIME_TD(time_data, am) : STRFTIME_TD(time_data, pm),
//                     (*format == 't' && count == 1) ? 1 : SIZE_MAX);
//             break;
//         default:
//             ret = strftime_nstr(str, pos, max, format, count);
//             break;
//         }
//         format += count;
//     }
//
//     return ret;
// }
//
//
// static size_t strftime_impl(STRFTIME_CHAR *str, size_t max,
//         const STRFTIME_CHAR *format, const struct tm *mstm,
//         __lc_time_data *time_data, _locale_t loc)
// {
//     size_t ret, tmp;
//     BOOL alternate;
//     int year = mstm ? mstm->tm_year + 1900 : -1;
//
//     if(!str || !format) {
//         if(str && max)
//             *str = 0;
//         *_errno() = EINVAL;
//         return 0;
//     }
//
//     if(!time_data)
//         time_data = loc ? loc->locinfo->lc_time_curr : get_locinfo()->lc_time_curr;
//
//     for(ret=0; *format && ret<max; format++) {
//         if(*format != '%') {
//             if(_isleadbyte_l((unsigned char)*format, loc)) {
//                 str[ret++] = *(format++);
//                 if(ret == max) continue;
//                 if(!MSVCRT_CHECK_PMT(str[ret]))
//                     goto einval_error;
//             }
//             str[ret++] = *format;
//             continue;
//         }
//
//         format++;
//         if(*format == '#') {
//             alternate = TRUE;
//             format++;
//         }else {
//             alternate = FALSE;
//         }
//
//         if(!MSVCRT_CHECK_PMT(mstm))
//             goto einval_error;
//
//         switch(*format) {
//         case 'c':
//             if(!strftime_format(str, &ret, max, mstm, time_data,
//                     alternate ? STRFTIME_TD(time_data, date) : STRFTIME_TD(time_data, short_date)))
//                 return 0;
//             if(ret < max)
//                 str[ret++] = ' ';
//             if(!strftime_format(str, &ret, max, mstm, time_data, STRFTIME_TD(time_data, time)))
//                 return 0;
//             break;
//         case 'x':
//             if(!strftime_format(str, &ret, max, mstm, time_data,
//                     alternate ? STRFTIME_TD(time_data, date) : STRFTIME_TD(time_data, short_date)))
//                 return 0;
//             break;
//         case 'X':
//             if(!strftime_format(str, &ret, max, mstm, time_data, STRFTIME_TD(time_data, time)))
//                 return 0;
//             break;
//         case 'a':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_wday>=0 && mstm->tm_wday<=6))
//                 goto einval_error;
//             if(!strftime_str(str, &ret, max, STRFTIME_TD(time_data, short_wday)[mstm->tm_wday]))
//                 return 0;
//             break;
//         case 'A':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_wday>=0 && mstm->tm_wday<=6))
//                 goto einval_error;
//             if(!strftime_str(str, &ret, max, STRFTIME_TD(time_data, wday)[mstm->tm_wday]))
//                 return 0;
//             break;
//         case 'b':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_mon>=0 && mstm->tm_mon<=11))
//                 goto einval_error;
//             if(!strftime_str(str, &ret, max, STRFTIME_TD(time_data, short_mon)[mstm->tm_mon]))
//                 return 0;
//             break;
//         case 'B':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_mon>=0 && mstm->tm_mon<=11))
//                 goto einval_error;
//             if(!strftime_str(str, &ret, max, STRFTIME_TD(time_data, mon)[mstm->tm_mon]))
//                 return 0;
//             break;
//         case 'd':
//             if(!strftime_int(str, &ret, max, mstm->tm_mday, alternate ? 0 : 2, 1, 31))
//                 return 0;
//             break;
//         case 'H':
//             if(!strftime_int(str, &ret, max, mstm->tm_hour, alternate ? 0 : 2, 0, 23))
//                 return 0;
//             break;
//         case 'I':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_hour>=0 && mstm->tm_hour<=23))
//                 goto einval_error;
//             if(!strftime_int(str, &ret, max, (mstm->tm_hour + 11) % 12 + 1,
//                         alternate ? 0 : 2, 1, 12))
//                 return 0;
//             break;
//         case 'j':
//             if(!strftime_int(str, &ret, max, mstm->tm_yday+1, alternate ? 0 : 3, 1, 366))
//                 return 0;
//             break;
//         case 'm':
//             if(!strftime_int(str, &ret, max, mstm->tm_mon+1, alternate ? 0 : 2, 1, 12))
//                 return 0;
//             break;
//         case 'M':
//             if(!strftime_int(str, &ret, max, mstm->tm_min, alternate ? 0 : 2, 0, 59))
//                 return 0;
//             break;
//         case 'p':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_hour>=0 && mstm->tm_hour<=23))
//                 goto einval_error;
//             if(!strftime_str(str, &ret, max, mstm->tm_hour<12 ?
//                         STRFTIME_TD(time_data, am) : STRFTIME_TD(time_data, pm)))
//                 return 0;
//             break;
//         case 'S':
//             if(!strftime_int(str, &ret, max, mstm->tm_sec, alternate ? 0 : 2, 0, MAX_SECONDS))
//                 return 0;
//             break;
//         case 'w':
//             if(!strftime_int(str, &ret, max, mstm->tm_wday, 0, 0, 6))
//                 return 0;
//             break;
//         case 'y':
//             if(!MSVCRT_CHECK_PMT(year>=1900))
//                 goto einval_error;
//             if(!strftime_int(str, &ret, max, year%100, alternate ? 0 : 2, 0, 99))
//                 return 0;
//             break;
//         case 'Y':
//             if(!strftime_int(str, &ret, max, year, alternate ? 0 : 4, 0, 9999))
//                 return 0;
//             break;
//         case 'z':
//         case 'Z':
//             _tzset();
//             if(_get_tzname(&tmp, str+ret, max-ret, mstm->tm_isdst ? 1 : 0))
//                 return 0;
//             ret += tmp-1;
//             break;
//         case 'U':
//         case 'W':
//             if(!MSVCRT_CHECK_PMT(mstm->tm_wday>=0 && mstm->tm_wday<=6))
//                 goto einval_error;
//             if(!MSVCRT_CHECK_PMT(mstm->tm_yday>=0 && mstm->tm_yday<=365))
//                 goto einval_error;
//             if(*format == 'U')
//                 tmp = mstm->tm_wday;
//             else if(!mstm->tm_wday)
//                 tmp = 6;
//             else
//                 tmp = mstm->tm_wday-1;
//
//             tmp = mstm->tm_yday/7 + (tmp<=mstm->tm_yday%7);
//             if(!strftime_int(str, &ret, max, tmp, alternate ? 0 : 2, 0, 53))
//                 return 0;
//             break;
//         case '%':
//             str[ret++] = '%';
//             break;
//         default:
//             WARN("unknown format %c\n", *format);
//             MSVCRT_INVALID_PMT("unknown format", EINVAL);
//             goto einval_error;
//         }
//     }
//
//     if(ret == max) {
//         if(max)
//             *str = 0;
//         *_errno() = ERANGE;
//         return 0;
//     }
//
//     str[ret] = 0;
//     return ret;
//
// einval_error:
//     *str = 0;
//     return 0;
// }
//
// static size_t strftime_helper(char *str, size_t max, const char *format,
//         const struct tm *mstm, __lc_time_data *time_data, _locale_t loc)
// {
//     TRACE("(%p %Iu %s %p %p %p)\n", str, max, format, mstm, time_data, loc);
//     return strftime_impl(str, max, format, mstm, time_data, loc);
// }
//
//
// /*********************************************************************
//  *		_Strftime (MSVCRT.@)
//  */
// size_t CDECL _Strftime(char *str, size_t max, const char *format,
//         const struct tm *mstm, void *time_data)
// {
//     return strftime_helper(str, max, format, mstm, time_data, NULL);
// }
//
// /*********************************************************************
//  *		strftime (MSVCRT.@)
//  */
// size_t CDECL strftime( char *str, size_t max, const char *format,
//                                      const struct tm *mstm )
// {
//     return strftime_helper(str, max, format, mstm, NULL, NULL);
// }
//
// static size_t wcsftime_helper( wchar_t *str, size_t max,
//         const wchar_t *format, const struct tm *mstm,
//         __lc_time_data *time_data, _locale_t loc )
// {
//     char *s, *fmt;
//     size_t len;
//
//     TRACE("%p %Iu %s %p %p %p\n", str, max, debugstr_w(format), mstm, time_data, loc);
//
//     len = _wcstombs_l( NULL, format, 0, loc ) + 1;
//     if (!(fmt = malloc( len ))) return 0;
//     _wcstombs_l(fmt, format, len, loc);
//
//     if ((s = malloc( max*4 )))
//     {
//         if (!strftime_impl( s, max*4, fmt, mstm, time_data, loc )) s[0] = 0;
//         len = _mbstowcs_l( str, s, max, loc );
//         free( s );
//     }
//     else len = 0;
//
//     free( fmt );
//     return len;
// }
//
// /*********************************************************************
//  *              _wcsftime_l (MSVCRT.@)
//  */
// size_t CDECL _wcsftime_l( wchar_t *str, size_t max,
//         const wchar_t *format, const struct tm *mstm, _locale_t loc )
// {
//     return wcsftime_helper(str, max, format, mstm, NULL, loc);
// }
//
// /*********************************************************************
//  *     wcsftime (MSVCRT.@)
//  */
// size_t CDECL wcsftime( wchar_t *str, size_t max,
//                                      const wchar_t *format, const struct tm *mstm )
// {
//     return wcsftime_helper(str, max, format, mstm, NULL, NULL);
// }
//
//
// static char* asctime_buf(char *buf, const struct tm *mstm)
// {
//     static const char wday[7][4] = {"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"};
//     static const char month[12][4] = {"Jan", "Feb", "Mar", "Apr", "May",
//         "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"};
//
//     if (!mstm || mstm->tm_sec<0 || mstm->tm_sec>59
//             || mstm->tm_min<0 || mstm->tm_min>59
//             || mstm->tm_hour<0 || mstm->tm_hour>23
//             || mstm->tm_mon<0 || mstm->tm_mon>11
//             || mstm->tm_wday<0 || mstm->tm_wday>6
//             || mstm->tm_year<0 || mstm->tm_mday<0
//             || mstm->tm_mday>MonthLengths[IsLeapYear(1900+mstm->tm_year)][mstm->tm_mon]) {
//         *_errno() = EINVAL;
//         return NULL;
//     }
//
//     _snprintf(buf, 26, "%s %s %02d %02d:%02d:%02d %c%03d\n", wday[mstm->tm_wday],
//             month[mstm->tm_mon], mstm->tm_mday, mstm->tm_hour, mstm->tm_min,
//             mstm->tm_sec, '1'+(mstm->tm_year+900)/1000, (900+mstm->tm_year)%1000);
//     return buf;
// }
//
// /*********************************************************************
//  *		asctime (MSVCRT.@)
//  */
// char *  asctime(const struct tm *mstm)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     /* asctime returns date in format that always has exactly 26 characters */
//     if (!data->asctime_buffer) {
//         data->asctime_buffer = malloc(26);
//         if (!data->asctime_buffer) {
//             *_errno() = ENOMEM;
//             return NULL;
//         }
//     }
//
//     return asctime_buf(data->asctime_buffer, mstm);
// }
//
// /*********************************************************************
//  *      asctime_s (MSVCRT.@)
//  */
// int CDECL asctime_s(char* time, size_t size, const struct tm *mstm)
// {
//     if (!MSVCRT_CHECK_PMT(time != NULL)) return EINVAL;
//     if (size) time[0] = 0;
//     if (!MSVCRT_CHECK_PMT(size >= 26)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_sec >= 0 && mstm->tm_sec < 60)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_min >= 0 && mstm->tm_min < 60)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_hour >= 0 && mstm->tm_hour < 24)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_mon >= 0 && mstm->tm_mon < 12)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_wday >= 0 && mstm->tm_wday < 7)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_year >= 0)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_mday >= 0)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm->tm_mday <= MonthLengths[IsLeapYear(1900+mstm->tm_year)][mstm->tm_mon])) return EINVAL;
//
//     asctime_buf(time, mstm);
//     return 0;
// }
//
// /*********************************************************************
//  *		_wasctime (MSVCRT.@)
//  */
// wchar_t *  _wasctime(const struct tm *mstm)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     char buffer[26];
//
//     if(!data->wasctime_buffer) {
//         data->wasctime_buffer = malloc(26*sizeof(wchar_t));
//         if(!data->wasctime_buffer) {
//             *_errno() = ENOMEM;
//             return NULL;
//         }
//     }
//
//     if(!asctime_buf(buffer, mstm))
//         return NULL;
//
//     MultiByteToWideChar(CP_ACP, 0, buffer, -1, data->wasctime_buffer, 26);
//     return data->wasctime_buffer;
// }
//
// /*********************************************************************
//  *      _wasctime_s (MSVCRT.@)
//  */
// int CDECL _wasctime_s(wchar_t* time, size_t size, const struct tm *mstm)
// {
//     char buffer[26];
//     int ret;
//
//     if (!MSVCRT_CHECK_PMT(time != NULL)) return EINVAL;
//     if (size) time[0] = 0;
//     if (!MSVCRT_CHECK_PMT(size >= 26)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mstm != NULL)) return EINVAL;
//
//     ret = asctime_s(buffer, sizeof(buffer), mstm);
//     if(ret)
//         return ret;
//     MultiByteToWideChar(CP_ACP, 0, buffer, -1, time, size);
//     return 0;
// }
//
// /*********************************************************************
//  *		_ctime64 (MSVCRT.@)
//  */
// char *  _ctime64(const __time64_t *time)
// {
//     struct tm *t;
//     t = _localtime64( time );
//     if (!t) return NULL;
//     return asctime( t );
// }
//
// /*********************************************************************
//  *		_ctime64_s (MSVCRT.@)
//  */
// errno_t CDECL _ctime64_s(char *res, size_t len, const __time64_t *time)
// {
//     struct tm t;
//     int ret;
//
//     if (!MSVCRT_CHECK_PMT( res != NULL )) return EINVAL;
//     if (!MSVCRT_CHECK_PMT( len >= 26 )) return EINVAL;
//     res[0] = '\0';
//     if (!MSVCRT_CHECK_PMT( time != NULL )) return EINVAL;
//     if (!MSVCRT_CHECK_PMT( *time > 0 )) return EINVAL;
//
//     ret = _localtime64_s( &t, time );
//     if (ret)
//         return ret;
//     return asctime_s( res, len, &t );
// }
//
// /*********************************************************************
//  *		_ctime32 (MSVCRT.@)
//  */
// char *  _ctime32(const __time32_t *time)
// {
//     struct tm *t;
//     t = _localtime32( time );
//     if (!t) return NULL;
//     return asctime( t );
// }
//
// /*********************************************************************
//  *		_ctime32_s (MSVCRT.@)
//  */
// errno_t CDECL _ctime32_s(char *res, size_t len, const __time32_t *time)
// {
//     struct tm t;
//     int ret;
//
//     if (!MSVCRT_CHECK_PMT( res != NULL )) return EINVAL;
//     if (!MSVCRT_CHECK_PMT( len >= 26 )) return EINVAL;
//     res[0] = '\0';
//     if (!MSVCRT_CHECK_PMT( time != NULL )) return EINVAL;
//     if (!MSVCRT_CHECK_PMT( *time > 0 )) return EINVAL;
//
//     ret = _localtime32_s( &t, time );
//     if (ret)
//         return ret;
//     return asctime_s( res, len, &t );
// }
//
// /*********************************************************************
//  *		_wctime64 (MSVCRT.@)
//  */
// wchar_t *  _wctime64(const __time64_t *time)
// {
//     return _wasctime( _localtime64(time) );
// }
//
// /*********************************************************************
//  *		_wctime32 (MSVCRT.@)
//  */
// wchar_t *  _wctime32(const __time32_t *time)
// {
//     return _wasctime( _localtime32(time) );
// }
//
// /*********************************************************************
//  *              _wctime64_s (MSVCRT.@)
//  */
// int CDECL _wctime64_s(wchar_t *buf,
//         size_t size, const __time64_t *time)
// {
//     struct tm tm;
//     int ret;
//
//     if(!MSVCRT_CHECK_PMT(buf != NULL)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(size != 0)) return EINVAL;
//     buf[0] = 0;
//     if(!MSVCRT_CHECK_PMT(time != NULL)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(*time >= 0)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(*time <= _MAX__TIME64_T)) return EINVAL;
//
//     ret = _localtime64_s(&tm, time);
//     if(ret != 0)
//         return ret;
//
//     return _wasctime_s(buf, size, &tm);
// }
//
// /*********************************************************************
//  *              _wctime32_s (MSVCRT.@)
//  */
// int CDECL _wctime32_s(wchar_t *buf, size_t size,
//         const __time32_t *time)
// {
//     struct tm tm;
//     int ret;
//
//     if(!MSVCRT_CHECK_PMT(buf != NULL)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(size != 0)) return EINVAL;
//     buf[0] = 0;
//     if(!MSVCRT_CHECK_PMT(time != NULL)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(*time >= 0)) return EINVAL;
//
//     ret = _localtime32_s(&tm, time);
//     if(ret != 0)
//         return ret;
//
//     return _wasctime_s(buf, size, &tm);
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
