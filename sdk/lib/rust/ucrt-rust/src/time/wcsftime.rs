//! time/wcsftime.cpp: the `wcsftime` engine (`store_string`,
//! `store_number_without_lead_zeroes`, `store_number`, the ISO-week helpers,
//! `store_winword`/`expand_time` -- mutually recursive -- and
//! `_Wcsftime_l`/`_Wcsftime`/`_wcsftime_l`/`wcsftime`), plus
//! `_W_Getdays`/`_W_Getmonths`/`_W_Gettnames`.
//!
//! `strftime.rs` (time/strftime.cpp) is a thin narrow-string wrapper over
//! `_Wcsftime_l`, defined separately; everything that actually interprets a
//! format specifier lives here.
//!
//! ## Details that are silent if wrong
//! * **Every leaf `tm`-field-range check uses the HANDLER-CALLING validate**
//!   (`_VALIDATE_RETURN`, i.e. [`validate`] here); every COMPOUND specifier
//!   that recurses into `expand_time` for its pieces (`%c`, `%D`, `%F`, `%r`,
//!   `%R`, `%T`, `%h`, `%x`/`store_winword`) propagates a `false` with
//!   [`validate_noexc`] instead -- so a single bad field fires the
//!   invalid-parameter handler EXACTLY ONCE, at the innermost failing check,
//!   never once per recursion level on the way back out.
//! * **`%c`/`%r` special-case the C locale by identity, `lc_time == &__lc_time_c`,
//!   not by name** -- a locale that happens to carry byte-identical time
//!   strings to "C" but is a distinct allocation still takes the
//!   locale-provided-format branch.
//! * **`%c`'s alternate form (`%#c`) in the C locale still takes the C-locale
//!   branch** (`lc_time == &__lc_time_c && !alternate_form`) -- `#` on `%c`
//!   in the C locale falls through to `WW_LDATEFMT`/`store_winword` instead
//!   (there is no "long form" of `%a %b %e %T %Y`, so `#` there is
//!   observable only via which branch runs, not via any different output in
//!   this build, since `__lc_time_c`'s long/short date formats are
//!   identical -- see `locale_data.rs`).
//! * **`store_number`'s `pad_character == '\0'` (the alternate-form request)
//!   takes an ENTIRELY DIFFERENT code path** (`store_number_without_lead_zeroes`,
//!   variable width, no leading zeroes) rather than "same code, blank pad
//!   char" -- ` ` (space, for `%e`'s non-alternate form) IS a valid
//!   `pad_character` and goes through the fixed-width path.
//! * **`store_number_without_lead_zeroes`'s "buffer too small" branch moves
//!   `*out` BACKWARD** (`*out -= *count`) before zeroing `*count` -- a
//!   caller relying on `*out` after a such a failure sees it rewound to
//!   where it stood before EVEN THIS call started eating from `*count`, not
//!   left where digits were partially written.
//! * **`store_winword`'s literal-quote handling (`'`) treats an EVEN repeat
//!   count as a no-op skip and an ODD one as "copy verbatim to the next
//!   quote or end of string"** -- `''''` (four quotes) is a no-op, `'''`
//!   (three) copies one literal quote's worth of nothing usefully but still
//!   enters the copy branch (the `if (*format == L'\'') { format++; break;
//!   }` fires immediately on the fourth char... actually on the very next
//!   char scanned, which for `'''` is the string's own third quote -- ported
//!   as the literal char-by-char loop, not reasoned about further).
//! * **`store_winword`'s non-Gregorian-calendar branch (`ww_caltype != 1`)
//!   calls the real Win32 `GetDateFormatEx`/`GetTimeFormatEx` through their
//!   `__acrt_*` thunks and FALLS THROUGH to the WinWord-picture path if
//!   either fails** (`cch == 0` or the scoped allocation fails) -- not
//!   reachable with this tree's only compiled-in time locale (`__lc_time_c`,
//!   `ww_caltype == 1`), so this path is UNVERIFIED here (no non-Gregorian
//!   locale exists to exercise it); ported faithfully from the header
//!   anyway rather than omitted.
//! * **The stack-scoped allocator (`_malloca_crt_t`/`__crt_scoped_stack_ptr`)
//!   is ported as a plain heap `malloc`/`free`** -- observably identical
//!   (same bytes, same failure-is-null behavior), differing only in where
//!   the memory lives, which no caller can observe.
//! * **`_Wcsftime_l`'s overflow path sets `errno = ERANGE` WITHOUT invoking
//!   the handler** (a raw `errno = ERANGE;`, not `_VALIDATE_RETURN`) --
//!   "ran out of buffer" is deliberately not an invalid-parameter condition,
//!   so a caller can grow the buffer and retry. `*string` (the FIRST byte)
//!   is reset to NUL on every failure path, buffer overflow included; bytes
//!   beyond that written so far are left as whatever `expand_time`
//!   partially wrote before running out (not re-zeroed).
//! * **A `nullptr` `timeptr` is validated LAZILY, only once a `%` directive
//!   is actually reached in `format`** -- `_Wcsftime_l("no percent signs",
//!   ..., timeptr=nullptr, ...)` succeeds; the C's own comment on
//!   `_Strftime_l` says `timeptr` is validated at the top, but that
//!   validation lives in the NARROW wrapper, not here (`_Wcsftime_l` itself
//!   has no top-level `timeptr != nullptr` check at all).
//! * **`%Z`'s DST index comes from `timeptr->tm_isdst` truthiness, not from
//!   `_isindst`** -- the caller-supplied flag decides which of the two
//!   `__wide_tzname()` slots is printed, unlike `_localtime32_s`'s own
//!   internal DST computation.

use core::ffi::{c_int, c_long, c_void};

use super::Tm;
use super::tzset::SysTime;
use crate::locale::{CrtLocalePointers, LcTimeData};
use crate::locale_update::LocaleUpdate;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

type LocaleT = *mut CrtLocalePointers;


extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    fn _get_timezone(result: *mut c_long) -> c_int;
    fn _get_dstbias(result: *mut c_long) -> c_int;
    /// The "C" locale's `LC_TIME` data (`locale_data.rs`, feature
    /// `locale-data` -- not enabled together with `time`, so reached by
    /// `extern` rather than `use`, same reasoning as `tzset.rs`'s
    /// `___lc_codepage_func`). `#[repr(transparent)]` `SyncConst<LcTimeData>`
    /// wrapper, so the raw `LcTimeData` layout is exactly this symbol's
    /// memory.
    static __lc_time_c: LcTimeData;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
}

use super::tzset::__tzset;
use crate::timeset::__wide_tzname;

extern "system" {
    fn __acrt_GetDateFormatEx(
        locale_name: *const u16,
        flags: u32,
        date: *const SysTime,
        format: *const u16,
        buffer: *mut u16,
        buffer_count: c_int,
        calendar: *const u16,
    ) -> c_int;
    fn __acrt_GetTimeFormatEx(
        locale_name: *const u16,
        flags: u32,
        time: *const SysTime,
        format: *const u16,
        buffer: *mut u16,
        buffer_count: c_int,
    ) -> c_int;
}

/// A `&'static [u16]` wide-literal terminated array, for the fixed
/// separator/punctuation strings `expand_time` stores between its
/// sub-expansions. Mirrors `setlocale.rs`'s `w!` macro (not reused directly:
/// that macro lives in a module with its own `#[macro_export]`, and this
/// file only needs single-character literals).
macro_rules! w {
    ($s:literal) => {{
        const LEN: usize = $s.len();
        const ARR: [u16; LEN + 1] = {
            let bytes = $s.as_bytes();
            let mut out = [0u16; LEN + 1];
            let mut i = 0;
            while i < LEN {
                out[i] = bytes[i] as u16;
                i += 1;
            }
            out
        };
        ARR.as_ptr()
    }};
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}
unsafe fn validate_noexc(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
    }
    ok
}

// ── `_W_Getdays` / `_W_Getmonths` / `_W_Gettnames` ──────────────────────────

unsafe fn lc_time_curr_of(locale: LocaleT) -> *const LcTimeData {
    let lu = LocaleUpdate::new(locale);
    (*lu.locinfo()).lc_time_curr as *const LcTimeData
}

#[no_mangle]
pub unsafe extern "C" fn _W_Getdays() -> *mut u16 {
    let time_data = lc_time_curr_of(core::ptr::null_mut());

    let mut length: usize = 0;
    for n in 0..7 {
        length += wcslen((*time_data).w_wday_abbr[n]) + wcslen((*time_data).w_wday[n]) + 2;
    }

    let buffer = malloc((length + 1) * 2) as *mut u16;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let mut it = buffer;
    for n in 0..7 {
        *it = b':' as u16;
        it = it.add(1);
        wcscpy_s(it, (length + 1) - (it.offset_from(buffer) as usize), (*time_data).w_wday_abbr[n]);
        it = it.add(wcslen(it));
        *it = b':' as u16;
        it = it.add(1);
        wcscpy_s(it, (length + 1) - (it.offset_from(buffer) as usize), (*time_data).w_wday[n]);
        it = it.add(wcslen(it));
    }
    *it = 0;

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn _W_Getmonths() -> *mut u16 {
    let time_data = lc_time_curr_of(core::ptr::null_mut());

    let mut length: usize = 0;
    for n in 0..12 {
        length += wcslen((*time_data).w_month_abbr[n]) + wcslen((*time_data).w_month[n]) + 2;
    }

    let buffer = malloc((length + 1) * 2) as *mut u16;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let mut it = buffer;
    for n in 0..12 {
        *it = b':' as u16;
        it = it.add(1);
        wcscpy_s(it, (length + 1) - (it.offset_from(buffer) as usize), (*time_data).w_month_abbr[n]);
        it = it.add(wcslen(it));
        *it = b':' as u16;
        it = it.add(1);
        wcscpy_s(it, (length + 1) - (it.offset_from(buffer) as usize), (*time_data).w_month[n]);
        it = it.add(wcslen(it));
    }
    *it = 0;

    buffer
}

/// `strlen`/`strcpy_s` over `c_char` (narrow), local to this function: the
/// crate's other `strlen`/`strcpy_s` externs are declared over `u8`/`c_char`
/// inconsistently across modules (see `clashing_extern_declarations`'s own
/// note in `mod.rs`), so this file, which only needs them for this one
/// function, declares its own rather than risking a mismatch against a
/// sibling module's extern block.
unsafe fn strlen_narrow(s: *const core::ffi::c_char) -> usize {
    let mut n = 0usize;
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
        n += 1;
    }
    n
}

/// `_W_Gettnames`: packs a WHOLE COPY of `__crt_lc_time_data`, with every
/// pointer field re-pointed into the same allocation (a self-contained
/// blob), sized in a first "dry run" pass (`phase == 0`) and filled in the
/// second (`phase == 1`) -- ported as the identical two-pass shape, not
/// simplified to a single pass with a growable buffer, since the C's own
/// pointer arithmetic (`dest->STR = ((CHAR*)dest) + bytes/sizeof(CHAR)`)
/// depends on `dest` already being the final allocation address.
#[no_mangle]
pub unsafe extern "C" fn _W_Gettnames() -> *mut c_void {
    let src = lc_time_curr_of(core::ptr::null_mut());

    unsafe fn process_narrow(
        phase: i32,
        bytes: &mut usize,
        dest: *mut LcTimeData,
        dest_field: *mut *mut core::ffi::c_char,
        src_str: *const core::ffi::c_char,
    ) {
        // sizeof(char) == 1: the `while bytes % sizeof(CHAR) != 0` alignment
        // loop in the C is a no-op for the narrow instantiation.
        if phase == 1 {
            let dst = (dest as *mut u8).add(*bytes) as *mut core::ffi::c_char;
            *dest_field = dst;
            core::ptr::copy_nonoverlapping(src_str, dst, strlen_narrow(src_str) + 1);
        }
        *bytes += strlen_narrow(src_str) + 1;
    }

    unsafe fn process_wide(phase: i32, bytes: &mut usize, dest: *mut LcTimeData, dest_field: *mut *mut u16, src_str: *const u16) {
        while *bytes % 2 != 0 {
            *bytes += 1;
        }
        if phase == 1 {
            let dst = (dest as *mut u8).add(*bytes) as *mut u16;
            *dest_field = dst;
            core::ptr::copy_nonoverlapping(src_str, dst, wcslen(src_str) + 1);
        }
        *bytes += (wcslen(src_str) + 1) * 2;
    }

    let mut bytes: usize = core::mem::size_of::<LcTimeData>();
    let mut dest: *mut LcTimeData = core::ptr::null_mut();

    for phase in 0..2 {
        if phase == 1 {
            dest = malloc(bytes) as *mut LcTimeData;
            if dest.is_null() {
                return core::ptr::null_mut();
            }
            core::ptr::write_bytes(dest as *mut u8, 0, bytes);
            bytes = core::mem::size_of::<LcTimeData>();
        }

        for idx in 0..7 {
            process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).wday_abbr[idx] }, (*src).wday_abbr[idx]);
        }
        for idx in 0..7 {
            process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).wday[idx] }, (*src).wday[idx]);
        }
        for idx in 0..12 {
            process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).month_abbr[idx] }, (*src).month_abbr[idx]);
        }
        for idx in 0..12 {
            process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).month[idx] }, (*src).month[idx]);
        }
        for idx in 0..2 {
            process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).ampm[idx] }, (*src).ampm[idx]);
        }
        process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).ww_sdatefmt }, (*src).ww_sdatefmt);
        process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).ww_ldatefmt }, (*src).ww_ldatefmt);
        process_narrow(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).ww_timefmt }, (*src).ww_timefmt);

        if phase == 1 {
            (*dest).ww_caltype = (*src).ww_caltype;
            (*dest).refcount = 0;
        }

        for idx in 0..7 {
            process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_wday_abbr[idx] }, (*src).w_wday_abbr[idx]);
        }
        for idx in 0..7 {
            process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_wday[idx] }, (*src).w_wday[idx]);
        }
        for idx in 0..12 {
            process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_month_abbr[idx] }, (*src).w_month_abbr[idx]);
        }
        for idx in 0..12 {
            process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_month[idx] }, (*src).w_month[idx]);
        }
        for idx in 0..2 {
            process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_ampm[idx] }, (*src).w_ampm[idx]);
        }
        process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_ww_sdatefmt }, (*src).w_ww_sdatefmt);
        process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_ww_ldatefmt }, (*src).w_ww_ldatefmt);
        process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_ww_timefmt }, (*src).w_ww_timefmt);
        process_wide(phase, &mut bytes, dest, if dest.is_null() { core::ptr::null_mut() } else { &raw mut (*dest).w_ww_locale_name }, (*src).w_ww_locale_name);
    }

    dest as *mut c_void
}

// ── store_string / store_number* ────────────────────────────────────────────

unsafe fn store_string(mut input: *const u16, out: *mut *mut u16, count: *mut usize) {
    while *count != 0 && *input != 0 {
        **out = *input;
        *out = (*out).add(1);
        input = input.add(1);
        *count -= 1;
    }
}

unsafe fn store_number_without_lead_zeroes(mut value: i32, out: *mut *mut u16, count: *mut usize) {
    let mut out_it = *out;
    if *count > 1 {
        loop {
            *out_it = (value % 10) as u16 + b'0' as u16;
            out_it = out_it.add(1);
            value /= 10;
            *count -= 1;
            if !(value > 0 && *count > 1) {
                break;
            }
        }
    } else {
        *out = (*out).sub(*count);
        *count = 0;
        return;
    }

    let mut left = *out;
    let mut right = out_it.sub(1);
    *out = out_it;

    while left < right {
        let x = *right;
        *right = *left;
        right = right.sub(1);
        *left = x;
        left = left.add(1);
    }
}

unsafe fn store_number(value: i32, digits: i32, out: *mut *mut u16, count: *mut usize, pad_character: u16) {
    if pad_character == 0 {
        store_number_without_lead_zeroes(value, out, count);
        return;
    }

    if (digits as usize) < *count {
        let mut temp: i32 = 0;
        let mut v = value;
        let mut d: i32 = digits - 1;
        while d + 1 != 0 {
            if v != 0 {
                *(*out).offset(d as isize) = (b'0' as i32 + v % 10) as u16;
            } else {
                *(*out).offset(d as isize) = pad_character;
            }
            v /= 10;
            temp += 1;
            d -= 1;
        }
        *out = (*out).offset(temp as isize);
        *count -= temp as usize;
    } else {
        *count = 0;
    }
}

// ── ISO week helpers ─────────────────────────────────────────────────────

fn compute_week_of_year(wstart: i32, wday: i32, yday: i32) -> i32 {
    let adjusted_wday = (wday + 7 - wstart) % 7;
    (yday + 7 - adjusted_wday) / 7
}

fn compute_iso_week_internal(year: i32, wday: i32, yday: i32) -> i32 {
    const MONDAY: i32 = 1;
    const TUESDAY: i32 = 2;
    const WEDNESDAY: i32 = 3;
    const THURSDAY: i32 = 4;

    let week_number = compute_week_of_year(MONDAY, wday, yday);
    let is_leap_year = super::is_leap_year(year) as i32;

    let yunleap = yday - is_leap_year;
    let jan1 = (371 - yday + wday) % 7;
    let dec32 = (jan1 + is_leap_year + 365) % 7;

    if (364 <= yunleap && dec32 == TUESDAY) || (363 <= yunleap && dec32 == WEDNESDAY) || (362 <= yunleap && dec32 == THURSDAY) {
        -1
    } else if jan1 == TUESDAY || jan1 == WEDNESDAY || jan1 == THURSDAY {
        week_number + 1
    } else {
        week_number
    }
}

fn compute_iso_week(year: i32, wday: i32, yday: i32) -> i32 {
    let week_number = compute_iso_week_internal(year, wday, yday);
    if week_number == 0 {
        return compute_iso_week_internal(year - 1, wday + 7 - yday, if super::is_leap_year(year - 1) { 366 } else { 365 });
    }
    if week_number > 0 {
        return week_number;
    }
    1
}

fn compute_iso_year(year: i32, wday: i32, yday: i32) -> i32 {
    let week_number = compute_iso_week_internal(year, wday, yday);
    if week_number == 0 {
        return year - 1;
    }
    if week_number > 0 {
        return year;
    }
    year + 1
}

// ── store_winword / expand_time (mutually recursive) ────────────────────────

const WW_SDATEFMT: i32 = 0;
const WW_LDATEFMT: i32 = 1;
const WW_TIMEFMT: i32 = 2;

#[allow(clippy::too_many_arguments)]
unsafe fn store_winword(
    locale: LocaleT,
    field_code: i32,
    tmptr: *const Tm,
    out: *mut *mut u16,
    count: *mut usize,
    lc_time: *const LcTimeData,
) -> bool {
    let mut format: *const u16 = match field_code {
        WW_SDATEFMT => (*lc_time).w_ww_sdatefmt,
        WW_LDATEFMT => (*lc_time).w_ww_ldatefmt,
        _ => (*lc_time).w_ww_timefmt,
    };

    if (*lc_time).ww_caltype != 1 {
        let is_time_format = field_code == WW_TIMEFMT;

        let mut system_time = SysTime {
            w_year: ((*tmptr).tm_year + 1900) as u16,
            w_month: ((*tmptr).tm_mon + 1) as u16,
            w_day_of_week: 0,
            w_day: (*tmptr).tm_mday as u16,
            w_hour: (*tmptr).tm_hour as u16,
            w_minute: (*tmptr).tm_min as u16,
            w_second: (*tmptr).tm_sec as u16,
            w_milliseconds: 0,
        };

        let mut cch: c_int = if is_time_format {
            __acrt_GetTimeFormatEx((*lc_time).w_ww_locale_name, 0, &system_time, format, core::ptr::null_mut(), 0)
        } else {
            __acrt_GetDateFormatEx((*lc_time).w_ww_locale_name, 0, &system_time, format, core::ptr::null_mut(), 0, core::ptr::null())
        };

        if cch != 0 {
            let buffer = malloc(cch as usize * 2) as *mut u16;
            if !buffer.is_null() {
                cch = if is_time_format {
                    __acrt_GetTimeFormatEx((*lc_time).w_ww_locale_name, 0, &system_time, format, buffer, cch)
                } else {
                    __acrt_GetDateFormatEx((*lc_time).w_ww_locale_name, 0, &system_time, format, buffer, cch, core::ptr::null())
                };

                let mut buffer_it = buffer;
                cch -= 1;
                while cch > 0 && *count > 0 {
                    **out = *buffer_it;
                    *out = (*out).add(1);
                    buffer_it = buffer_it.add(1);
                    *count -= 1;
                    cch -= 1;
                }

                free(buffer as *mut c_void);
                let _ = &mut system_time;
                return true;
            }
        }
        // fall through to localized Gregorian.
    }

    while *format != 0 && *count != 0 {
        let mut specifier: u16 = 0;
        let mut no_lead_zeros = false;

        let mut repeat: i32 = 0;
        let mut p = format;
        loop {
            let c = *p;
            p = p.add(1);
            if c != *format {
                break;
            }
            repeat += 1;
        }
        p = p.sub(1);

        match *format {
            v if v == b'M' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'm' as u16
                    }
                    2 => b'm' as u16,
                    3 => b'b' as u16,
                    4 => b'B' as u16,
                    _ => 0,
                };
            }
            v if v == b'd' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'd' as u16
                    }
                    2 => b'd' as u16,
                    3 => b'a' as u16,
                    4 => b'A' as u16,
                    _ => 0,
                };
            }
            v if v == b'y' as u16 => {
                specifier = match repeat {
                    2 => b'y' as u16,
                    4 => b'Y' as u16,
                    _ => 0,
                };
            }
            v if v == b'h' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'I' as u16
                    }
                    2 => b'I' as u16,
                    _ => 0,
                };
            }
            v if v == b'H' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'H' as u16
                    }
                    2 => b'H' as u16,
                    _ => 0,
                };
            }
            v if v == b'm' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'M' as u16
                    }
                    2 => b'M' as u16,
                    _ => 0,
                };
            }
            v if v == b's' as u16 => {
                specifier = match repeat {
                    1 => {
                        no_lead_zeros = true;
                        b'S' as u16
                    }
                    2 => b'S' as u16,
                    _ => 0,
                };
            }
            v if v == b'A' as u16 || v == b'a' as u16 => {
                static AMPM: [u16; 6] = [b'a' as u16, b'm' as u16, b'/' as u16, b'p' as u16, b'm' as u16, 0];
                static AP: [u16; 4] = [b'a' as u16, b'/' as u16, b'p' as u16, 0];
                if _wcsicmp(format, AMPM.as_ptr()) == 0 {
                    p = format.add(5);
                } else if _wcsicmp(format, AP.as_ptr()) == 0 {
                    p = format.add(3);
                }
                specifier = b'p' as u16;
            }
            v if v == b't' as u16 => {
                let ampmstr: *const u16 = if (*tmptr).tm_hour <= 11 { (*lc_time).w_ampm[0] } else { (*lc_time).w_ampm[1] };

                if repeat == 1 && *count > 0 {
                    **out = *ampmstr;
                    *out = (*out).add(1);
                    *count -= 1;
                } else {
                    let mut ampmstr = ampmstr;
                    while *ampmstr != 0 && *count > 0 {
                        **out = *ampmstr;
                        *out = (*out).add(1);
                        ampmstr = ampmstr.add(1);
                        *count -= 1;
                    }
                }
                format = p;
                continue;
            }
            v if v == b'\'' as u16 => {
                if repeat % 2 == 0 {
                    format = format.add(repeat as usize);
                } else {
                    format = format.add(repeat as usize);
                    while *format != 0 && *count != 0 {
                        if *format == b'\'' as u16 {
                            format = format.add(1);
                            break;
                        }
                        **out = *format;
                        *out = (*out).add(1);
                        format = format.add(1);
                        *count -= 1;
                    }
                }
                continue;
            }
            _ => {}
        }

        if specifier != 0 {
            if !validate_noexc(expand_time(locale, specifier, tmptr, out, count, lc_time, no_lead_zeros), EINVAL) {
                return false;
            }
            format = p;
        } else {
            **out = *format;
            *out = (*out).add(1);
            format = format.add(1);
            *count -= 1;
        }
    }

    true
}

#[allow(clippy::too_many_arguments)]
unsafe fn expand_time(
    locale: LocaleT,
    specifier: u16,
    timeptr: *const Tm,
    string: *mut *mut u16,
    left: *mut usize,
    lc_time: *const LcTimeData,
    alternate_form: bool,
) -> bool {
    let tm = &*timeptr;
    match specifier as u32 {
        // 'a': abbreviated weekday name
        0x61 => {
            if !validate(tm.tm_wday >= 0 && tm.tm_wday <= 6, EINVAL) {
                return false;
            }
            store_string((*lc_time).w_wday_abbr[tm.tm_wday as usize], string, left);
            true
        }
        // 'A': full weekday name
        0x41 => {
            if !validate(tm.tm_wday >= 0 && tm.tm_wday <= 6, EINVAL) {
                return false;
            }
            store_string((*lc_time).w_wday[tm.tm_wday as usize], string, left);
            true
        }
        // 'b': abbreviated month name
        0x62 => {
            if !validate(tm.tm_mon >= 0 && tm.tm_mon <= 11, EINVAL) {
                return false;
            }
            store_string((*lc_time).w_month_abbr[tm.tm_mon as usize], string, left);
            true
        }
        // 'B': full month name
        0x42 => {
            if !validate(tm.tm_mon >= 0 && tm.tm_mon <= 11, EINVAL) {
                return false;
            }
            store_string((*lc_time).w_month[tm.tm_mon as usize], string, left);
            true
        }
        // 'c': date and time representation
        0x63 => {
            if core::ptr::eq(lc_time, &__lc_time_c as *const LcTimeData) && !alternate_form {
                if !validate_noexc(expand_time(locale, b'a' as u16, timeptr, string, left, lc_time, false), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(expand_time(locale, b'b' as u16, timeptr, string, left, lc_time, false), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(expand_time(locale, b'e' as u16, timeptr, string, left, lc_time, false), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(expand_time(locale, b'T' as u16, timeptr, string, left, lc_time, false), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(expand_time(locale, b'Y' as u16, timeptr, string, left, lc_time, false), EINVAL) {
                    return false;
                }
            } else {
                let field_code = if alternate_form { WW_LDATEFMT } else { WW_SDATEFMT };
                if !validate_noexc(store_winword(locale, field_code, timeptr, string, left, lc_time), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(store_winword(locale, WW_TIMEFMT, timeptr, string, left, lc_time), EINVAL) {
                    return false;
                }
            }
            true
        }
        // 'C': century (00-99)
        0x43 => {
            if !validate(tm.tm_year >= -1900 && tm.tm_year <= 8099, EINVAL) {
                return false;
            }
            store_number(super::get_century(tm.tm_year), 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'd': day of month (01-31)
        0x64 => {
            if !validate(tm.tm_mday >= 1 && tm.tm_mday <= 31, EINVAL) {
                return false;
            }
            store_number(tm.tm_mday, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'D': "%m/%d/%y"
        0x44 => {
            if !validate_noexc(expand_time(locale, b'm' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!("/"), string, left);
            if !validate_noexc(expand_time(locale, b'd' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!("/"), string, left);
            if !validate_noexc(expand_time(locale, b'y' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            true
        }
        // 'e': day of month, space-padded
        0x65 => {
            if !validate(tm.tm_mday >= 1 && tm.tm_mday <= 31, EINVAL) {
                return false;
            }
            store_number(tm.tm_mday, 2, string, left, if alternate_form { 0 } else { b' ' as u16 });
            true
        }
        // 'F': "%Y-%m-%d"
        0x46 => {
            if !validate_noexc(expand_time(locale, b'Y' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!("-"), string, left);
            if !validate_noexc(expand_time(locale, b'm' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!("-"), string, left);
            if !validate_noexc(expand_time(locale, b'd' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            true
        }
        // 'g': last two digits of week-based year
        0x67 => {
            if !validate(tm.tm_year >= -1900 && tm.tm_year <= 8099, EINVAL) {
                return false;
            }
            let iso_year = compute_iso_year(tm.tm_year, tm.tm_wday, tm.tm_yday) + 1900;
            store_number(iso_year % 100, 2, string, left, b'0' as u16);
            true
        }
        // 'G': week-based year
        0x47 => {
            if !validate(tm.tm_year >= -1900 && tm.tm_year <= 8099, EINVAL) {
                return false;
            }
            let iso_year = compute_iso_year(tm.tm_year, tm.tm_wday, tm.tm_yday) + 1900;
            store_number(iso_year, 4, string, left, b'0' as u16);
            true
        }
        // 'h': "%b"
        0x68 => expand_time(locale, b'b' as u16, timeptr, string, left, lc_time, alternate_form),
        // 'H': 24-hour (00-23)
        0x48 => {
            if !validate(tm.tm_hour >= 0 && tm.tm_hour <= 23, EINVAL) {
                return false;
            }
            store_number(tm.tm_hour, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'I': 12-hour (01-12)
        0x49 => {
            if !validate(tm.tm_hour >= 0 && tm.tm_hour <= 23, EINVAL) {
                return false;
            }
            let mut hour = tm.tm_hour % 12;
            if hour == 0 {
                hour = 12;
            }
            store_number(hour, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'j': day of year (001-366)
        0x6a => {
            if !validate(tm.tm_yday >= 0 && tm.tm_yday <= 365, EINVAL) {
                return false;
            }
            store_number(tm.tm_yday + 1, 3, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'm': month (01-12)
        0x6d => {
            if !validate(tm.tm_mon >= 0 && tm.tm_mon <= 11, EINVAL) {
                return false;
            }
            store_number(tm.tm_mon + 1, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'M': minute (00-59)
        0x4d => {
            if !validate(tm.tm_min >= 0 && tm.tm_min <= 59, EINVAL) {
                return false;
            }
            store_number(tm.tm_min, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'n': newline
        0x6e => {
            store_string(w!("\n"), string, left);
            true
        }
        // 'p': AM/PM
        0x70 => {
            if !validate(tm.tm_hour >= 0 && tm.tm_hour <= 23, EINVAL) {
                return false;
            }
            let ampm = if tm.tm_hour <= 11 { (*lc_time).w_ampm[0] } else { (*lc_time).w_ampm[1] };
            store_string(ampm, string, left);
            true
        }
        // 'r': locale 12-hour clock
        0x72 => {
            if core::ptr::eq(lc_time, &__lc_time_c as *const LcTimeData) {
                if !validate_noexc(expand_time(locale, b'I' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                    return false;
                }
                store_string(w!(":"), string, left);
                if !validate_noexc(expand_time(locale, b'M' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                    return false;
                }
                store_string(w!(":"), string, left);
                if !validate_noexc(expand_time(locale, b'S' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                    return false;
                }
                store_string(w!(" "), string, left);
                if !validate_noexc(expand_time(locale, b'p' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                    return false;
                }
            } else if !validate_noexc(expand_time(locale, b'X' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            true
        }
        // 'R': "%H:%M"
        0x52 => {
            if !validate_noexc(expand_time(locale, b'H' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!(":"), string, left);
            if !validate_noexc(expand_time(locale, b'M' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            true
        }
        // 'S': seconds (00-60)
        0x53 => {
            if !validate(tm.tm_sec >= 0 && tm.tm_sec <= 60, EINVAL) {
                return false;
            }
            store_number(tm.tm_sec, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 't': tab
        0x74 => {
            store_string(w!("\t"), string, left);
            true
        }
        // 'T': "%H:%M:%S"
        0x54 => {
            if !validate_noexc(expand_time(locale, b'H' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!(":"), string, left);
            if !validate_noexc(expand_time(locale, b'M' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            store_string(w!(":"), string, left);
            if !validate_noexc(expand_time(locale, b'S' as u16, timeptr, string, left, lc_time, alternate_form), EINVAL) {
                return false;
            }
            true
        }
        // 'u': weekday (1-7); 'w': weekday (0-6)
        0x75 | 0x77 => {
            if !validate(tm.tm_wday >= 0 && tm.tm_wday <= 6, EINVAL) {
                return false;
            }
            let weekday_number = if tm.tm_wday == 0 && specifier == b'u' as u16 { 7 } else { tm.tm_wday };
            store_number(weekday_number, 1, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'U': Sunday week number; 'W': Monday week number
        0x55 | 0x57 => {
            if !validate(tm.tm_wday >= 0 && tm.tm_wday <= 6, EINVAL) {
                return false;
            }
            let mut wdaytemp = tm.tm_wday;
            if specifier == b'W' as u16 {
                wdaytemp = if tm.tm_wday == 0 { 6 } else { tm.tm_wday - 1 };
            }

            if !validate(tm.tm_yday >= 0 && tm.tm_yday <= 365, EINVAL) {
                return false;
            }
            let mut week_number: i32 = 0;
            if tm.tm_yday >= wdaytemp {
                week_number = tm.tm_yday / 7;
                if tm.tm_yday % 7 >= wdaytemp {
                    week_number += 1;
                }
            }
            store_number(week_number, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'V': ISO 8601 week number (01-53)
        0x56 => {
            let iso_week = compute_iso_week(tm.tm_year, tm.tm_wday, tm.tm_yday);
            store_number(iso_week, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'x': date display
        0x78 => {
            let field_code = if alternate_form { WW_LDATEFMT } else { WW_SDATEFMT };
            validate_noexc(store_winword(locale, field_code, timeptr, string, left, lc_time), EINVAL)
        }
        // 'X': time display
        0x58 => validate_noexc(store_winword(locale, WW_TIMEFMT, timeptr, string, left, lc_time), EINVAL),
        // 'y': year without century (00-99)
        0x79 => {
            if !validate(tm.tm_year >= -1900 && tm.tm_year <= 8099, EINVAL) {
                return false;
            }
            let two_digit_year = super::get_2digit_year(tm.tm_year);
            store_number(two_digit_year, 2, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'Y': year with century
        0x59 => {
            if !validate(tm.tm_year >= -1900 && tm.tm_year <= 8099, EINVAL) {
                return false;
            }
            let full_year = tm.tm_year + 1900;
            store_number(full_year, 4, string, left, if alternate_form { 0 } else { b'0' as u16 });
            true
        }
        // 'z': ISO 8601 timezone offset
        0x7a => {
            __tzset();
            let mut offset: c_long = 0;
            if !validate(_get_timezone(&mut offset) == 0, EINVAL) {
                return false;
            }
            if tm.tm_isdst != 0 {
                let mut dst_bias: c_long = 0;
                if !validate(_get_dstbias(&mut dst_bias) == 0, EINVAL) {
                    return false;
                }
                offset += dst_bias;
            }
            let positive_offset = if offset < 0 { -offset } else { offset };
            let hours_offset = (positive_offset / 60) / 60;
            let minutes_offset = (positive_offset / 60) % 60;
            let sign_string: *const u16 = if offset <= 0 { w!("+") } else { w!("-") };
            store_string(sign_string, string, left);
            store_number(hours_offset as i32, 2, string, left, b'0' as u16);
            store_number(minutes_offset as i32, 2, string, left, b'0' as u16);
            true
        }
        // 'Z': time zone name
        0x5a => {
            __tzset();
            let names = __wide_tzname();
            store_string(*names.add(if tm.tm_isdst != 0 { 1 } else { 0 }), string, left);
            true
        }
        // '%'
        0x25 => {
            store_string(w!("%"), string, left);
            true
        }
        _ => false,
    }
}

// ── `_Wcsftime_l` and friends ───────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _Wcsftime_l(
    string: *mut u16,
    max_size: usize,
    format: *const u16,
    timeptr: *const Tm,
    lc_time_arg: *mut c_void,
    locale: LocaleT,
) -> usize {
    if !validate(!string.is_null(), EINVAL) {
        return 0;
    }
    if !validate(max_size != 0, EINVAL) {
        return 0;
    }
    *string = 0;

    if !validate(!format.is_null(), EINVAL) {
        return 0;
    }

    let mut lu = LocaleUpdate::new(locale);
    let lc_time: *const LcTimeData = if lc_time_arg.is_null() {
        (*lu.locinfo()).lc_time_curr as *const LcTimeData
    } else {
        lc_time_arg as *const LcTimeData
    };

    let mut string_it = string;
    let mut format_it = format;

    let mut failed = false;
    let mut remaining = max_size;

    loop {
        if remaining == 0 {
            break;
        }
        match *format_it {
            0 => break,
            v if v == b'%' as u16 => {
                if !validate(!timeptr.is_null(), EINVAL) {
                    return 0;
                }

                format_it = format_it.add(1);

                let mut alternate_form = false;
                if *format_it == b'#' as u16 {
                    alternate_form = true;
                    format_it = format_it.add(1);
                }

                if *format_it == b'E' as u16 || *format_it == b'O' as u16 {
                    format_it = format_it.add(1);
                }

                if !expand_time(lu.locale_t(), *format_it, timeptr, &mut string_it, &mut remaining, lc_time, alternate_form) {
                    if remaining > 0 {
                        failed = true;
                    }
                    break;
                }

                format_it = format_it.add(1);
            }
            _ => {
                *string_it = *format_it;
                string_it = string_it.add(1);
                format_it = format_it.add(1);
                remaining -= 1;
            }
        }
    }

    if !failed && remaining > 0 {
        *string_it = 0;
        max_size - remaining
    } else {
        *string = 0;
        if !failed && remaining == 0 {
            *_errno() = ERANGE;
        } else {
            validate(false, EINVAL);
        }
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn _Wcsftime(buffer: *mut u16, max_size: usize, format: *const u16, timeptr: *const Tm, lc_time_arg: *mut c_void) -> usize {
    _Wcsftime_l(buffer, max_size, format, timeptr, lc_time_arg, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wcsftime_l(buffer: *mut u16, max_size: usize, format: *const u16, timeptr: *const Tm, locale: LocaleT) -> usize {
    _Wcsftime_l(buffer, max_size, format, timeptr, core::ptr::null_mut(), locale)
}

#[no_mangle]
pub unsafe extern "C" fn wcsftime(buffer: *mut u16, max_size: usize, format: *const u16, timeptr: *const Tm) -> usize {
    _Wcsftime_l(buffer, max_size, format, timeptr, core::ptr::null_mut(), core::ptr::null_mut())
}
