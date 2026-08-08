//! Grades time.c — reference TU #11. All 52 exports, over timeimpl.c's frozen clock and
//! its four time-zone presets.
//!
//! WHY THIS FILE COULD NOT EXIST WITHOUT THE KNOBS. The subject and the reference are two
//! bodies of code in one process, called one after the other. If both asked the host what
//! time it is they would get two different answers, and every row over `_time64`, `clock`,
//! `_ftime64`, `asctime` or `_strdate` would be a coin flip that usually agrees — which is
//! the worst possible oracle, green in CI and red at midnight. So the instant is set by the
//! test (`ref_time_set_now`) and read by both sides, and that buys something a real clock
//! cannot: a row can sit ON a DST transition, on a leap day, on 2038-01-19T03:14:07Z, or on
//! `_MAX__TIME64_T`.
//!
//! The ZONE is the other half, and it is where time.c's actual arithmetic lives. A shim
//! that always answered "UTC, no DST" would make `is_dst`, `mktime_helper`'s bias
//! correction and half of `_tzset` unreachable, and the harness would report a confident
//! green over them. timeimpl.c's four presets exist to stop that: UTC, US Pacific
//! (post-2007 rules), India (a HALF-HOUR bias, which catches whole-hour arithmetic), and
//! Sydney (southern hemisphere, so the DST window WRAPS the year end and a rule engine that
//! assumes start < end gets it backwards all year). Preset -1 makes
//! `GetTimeZoneInformation` FAIL, which is the only way `_tzset`'s early-out arm is
//! reachable. The `TZ` environment variable is a fifth: it takes `_tzset`'s OTHER branch
//! entirely, the hand-rolled `tzn[+|-]hh[:mm[:ss]][dzn]` parser.
//!
//! WHAT IS SHARED AND WHAT IS NOT. The clock, the zone and the C locale's calendar strings
//! (`cloc_time_data`, lifted from locale.c by build.rs) are ONE copy read by both sides —
//! they are what the environment IS, and a differential test cannot grade them anyway
//! (§9.73). Everything time.c OWNS is per-side: `MSVCRT___timezone`, `MSVCRT___daylight`,
//! `MSVCRT__dstbias`, `MSVCRT__tzname` and the three thread-data result buffers. `_tzset`
//! WRITES the first four, so one shared copy would let each side read back what the other
//! stored and every row would compare a value against itself.
//!
//! POINTERS ARE COMPARED AS CONTENT PLUS IDENTITY. `asctime`, `_localtime64` and friends
//! return an interior pointer into the caller's own thread data, so the addresses are
//! meaningless across sides — but the ALIASING is part of the contract (the classic bug is
//! printing `localtime()` and `gmtime()` in one `printf` and getting the same string
//! twice). Each row compares the bytes across sides, and compares the POINTER to the same
//! side's previous call.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_double, c_int, c_uint, c_void};

/// msvcrt's `__msvcrt_long`, which is Windows `long` — 32 bits on every arch. NOT
/// `core::ffi::c_long`, which on this LP64 host is 64: `__p__timezone` and `__p__dstbias`
/// hand back a pointer to one of these, and reading eight bytes where the DLL stores four
/// pulls in the next global as the high half. It does not crash, it just reports garbage,
/// and the garbage differs between the two sides because their globals are laid out
/// independently — which is exactly what the first `__p__timezone` row said.
#[allow(non_camel_case_types)]
type MsvcrtLong = i32;

// ── the shared type model ────────────────────────────────────────────────────

/// msvcrt's `struct tm` is nine ints. The HOST's — which the reference TU compiles
/// against, since it takes glibc's `<time.h>` — is those same nine in the same order plus
/// `tm_gmtoff`/`tm_zone` at the end. Every field time.c touches therefore sits at the same
/// offset on both sides and the reference simply allocates a larger box than the DLL would.
///
/// The tail is spelled here rather than left off because a `struct tm` handed IN (to
/// `_mktime64`, `asctime`, `strftime`) is allocated by THIS file: nine ints would be a
/// 20-byte object the reference's declared type says is 56, and while time.c never writes
/// past `tm_isdst`, "never" is a property of today's source, not of the ABI. `_pad` is dead
/// storage on purpose — no row reads it, and neither side should write it.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct CTm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
    _pad: [u64; 2],
}

impl CTm {
    /// The nine fields that mean anything, for comparison and for messages.
    fn nine(&self) -> [c_int; 9] {
        [
            self.tm_sec,
            self.tm_min,
            self.tm_hour,
            self.tm_mday,
            self.tm_mon,
            self.tm_year,
            self.tm_wday,
            self.tm_yday,
            self.tm_isdst,
        ]
    }
}

/// `<sys/timeb.h>`'s two structs. NOT a narrowing view of one another — `_ftime32` copies
/// field by field for exactly that reason, which is a thing a row can catch.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Timeb32 {
    time: i32,
    millitm: u16,
    timezone: i16,
    dstflag: i16,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Timeb64 {
    time: i64,
    millitm: u16,
    timezone: i16,
    dstflag: i16,
}

extern "C" {
    // ── the subject: msvcrt-crt-rust's time.rs, under the plain names ──
    fn _time64(buf: *mut i64) -> i64;
    fn _time32(buf: *mut i32) -> i32;
    fn clock() -> c_int;
    fn msvcrt_init_clock();
    fn _difftime64(t1: i64, t2: i64) -> c_double;
    fn _difftime32(t1: i32, t2: i32) -> c_double;
    fn _gmtime64(secs: *const i64) -> *mut CTm;
    fn _gmtime64_s(res: *mut CTm, secs: *const i64) -> c_int;
    fn _gmtime32(secs: *const i32) -> *mut CTm;
    fn _gmtime32_s(res: *mut CTm, secs: *const i32) -> c_int;
    fn _localtime64(secs: *const i64) -> *mut CTm;
    fn _localtime64_s(res: *mut CTm, secs: *const i64) -> c_int;
    fn _localtime32(secs: *const i32) -> *mut CTm;
    fn _localtime32_s(res: *mut CTm, secs: *const i32) -> c_int;
    fn _mktime64(t: *mut CTm) -> i64;
    fn _mktime32(t: *mut CTm) -> i32;
    fn _mkgmtime64(t: *mut CTm) -> i64;
    fn _mkgmtime32(t: *mut CTm) -> i32;
    fn asctime(t: *const CTm) -> *mut c_char;
    fn asctime_s(buf: *mut c_char, size: usize, t: *const CTm) -> c_int;
    fn _wasctime(t: *const CTm) -> *mut u16;
    fn _wasctime_s(buf: *mut u16, size: usize, t: *const CTm) -> c_int;
    fn _ctime64(t: *const i64) -> *mut c_char;
    fn _ctime64_s(buf: *mut c_char, len: usize, t: *const i64) -> c_int;
    fn _ctime32(t: *const i32) -> *mut c_char;
    fn _ctime32_s(buf: *mut c_char, len: usize, t: *const i32) -> c_int;
    fn _wctime64(t: *const i64) -> *mut u16;
    fn _wctime64_s(buf: *mut u16, size: usize, t: *const i64) -> c_int;
    fn _wctime32(t: *const i32) -> *mut u16;
    fn _wctime32_s(buf: *mut u16, size: usize, t: *const i32) -> c_int;
    fn strftime(s: *mut c_char, max: usize, fmt: *const c_char, t: *const CTm) -> usize;
    fn wcsftime(s: *mut u16, max: usize, fmt: *const u16, t: *const CTm) -> usize;
    fn _wcsftime_l(
        s: *mut u16, max: usize, fmt: *const u16, t: *const CTm, loc: *mut c_void,
    ) -> usize;
    fn _Strftime(
        s: *mut c_char, max: usize, fmt: *const c_char, t: *const CTm, td: *mut c_void,
    ) -> usize;
    fn _strdate(buf: *mut c_char) -> *mut c_char;
    fn _strdate_s(buf: *mut c_char, size: usize) -> c_int;
    fn _strtime(buf: *mut c_char) -> *mut c_char;
    fn _strtime_s(buf: *mut c_char, size: usize) -> c_int;
    fn _wstrdate(buf: *mut u16) -> *mut u16;
    fn _wstrdate_s(buf: *mut u16, size: usize) -> c_int;
    fn _wstrtime(buf: *mut u16) -> *mut u16;
    fn _wstrtime_s(buf: *mut u16, size: usize) -> c_int;
    fn _ftime64(buf: *mut Timeb64);
    fn _ftime64_s(buf: *mut Timeb64) -> c_int;
    fn _ftime32(buf: *mut Timeb32);
    fn _ftime32_s(buf: *mut Timeb32) -> c_int;
    fn _tzset();
    fn _get_tzname(ret: *mut usize, buf: *mut c_char, bufsize: usize, index: c_int) -> c_int;
    fn __p__daylight() -> *mut c_int;
    fn __p__dstbias() -> *mut MsvcrtLong;
    fn __p__timezone() -> *mut MsvcrtLong;
    fn __p__tzname() -> *mut *mut c_char;

    // ── the reference: the genuine dll/win32/msvcrt/time.c, renamed ──
    fn ref__time64(buf: *mut i64) -> i64;
    fn ref__time32(buf: *mut i32) -> i32;
    fn ref_clock() -> c_int;
    fn ref_msvcrt_init_clock();
    fn ref__difftime64(t1: i64, t2: i64) -> c_double;
    fn ref__difftime32(t1: i32, t2: i32) -> c_double;
    fn ref__gmtime64(secs: *const i64) -> *mut CTm;
    fn ref__gmtime64_s(res: *mut CTm, secs: *const i64) -> c_int;
    fn ref__gmtime32(secs: *const i32) -> *mut CTm;
    fn ref__gmtime32_s(res: *mut CTm, secs: *const i32) -> c_int;
    fn ref__localtime64(secs: *const i64) -> *mut CTm;
    fn ref__localtime64_s(res: *mut CTm, secs: *const i64) -> c_int;
    fn ref__localtime32(secs: *const i32) -> *mut CTm;
    fn ref__localtime32_s(res: *mut CTm, secs: *const i32) -> c_int;
    fn ref__mktime64(t: *mut CTm) -> i64;
    fn ref__mktime32(t: *mut CTm) -> i32;
    fn ref__mkgmtime64(t: *mut CTm) -> i64;
    fn ref__mkgmtime32(t: *mut CTm) -> i32;
    fn ref_asctime(t: *const CTm) -> *mut c_char;
    fn ref_asctime_s(buf: *mut c_char, size: usize, t: *const CTm) -> c_int;
    fn ref__wasctime(t: *const CTm) -> *mut u16;
    fn ref__wasctime_s(buf: *mut u16, size: usize, t: *const CTm) -> c_int;
    fn ref__ctime64(t: *const i64) -> *mut c_char;
    fn ref__ctime64_s(buf: *mut c_char, len: usize, t: *const i64) -> c_int;
    fn ref__ctime32(t: *const i32) -> *mut c_char;
    fn ref__ctime32_s(buf: *mut c_char, len: usize, t: *const i32) -> c_int;
    fn ref__wctime64(t: *const i64) -> *mut u16;
    fn ref__wctime64_s(buf: *mut u16, size: usize, t: *const i64) -> c_int;
    fn ref__wctime32(t: *const i32) -> *mut u16;
    fn ref__wctime32_s(buf: *mut u16, size: usize, t: *const i32) -> c_int;
    fn ref_strftime(s: *mut c_char, max: usize, fmt: *const c_char, t: *const CTm) -> usize;
    fn ref_wcsftime(s: *mut u16, max: usize, fmt: *const u16, t: *const CTm) -> usize;
    fn ref__wcsftime_l(
        s: *mut u16, max: usize, fmt: *const u16, t: *const CTm, loc: *mut c_void,
    ) -> usize;
    fn ref__Strftime(
        s: *mut c_char, max: usize, fmt: *const c_char, t: *const CTm, td: *mut c_void,
    ) -> usize;
    fn ref__strdate(buf: *mut c_char) -> *mut c_char;
    fn ref__strdate_s(buf: *mut c_char, size: usize) -> c_int;
    fn ref__strtime(buf: *mut c_char) -> *mut c_char;
    fn ref__strtime_s(buf: *mut c_char, size: usize) -> c_int;
    fn ref__wstrdate(buf: *mut u16) -> *mut u16;
    fn ref__wstrdate_s(buf: *mut u16, size: usize) -> c_int;
    fn ref__wstrtime(buf: *mut u16) -> *mut u16;
    fn ref__wstrtime_s(buf: *mut u16, size: usize) -> c_int;
    fn ref__ftime64(buf: *mut Timeb64);
    fn ref__ftime64_s(buf: *mut Timeb64) -> c_int;
    fn ref__ftime32(buf: *mut Timeb32);
    fn ref__ftime32_s(buf: *mut Timeb32) -> c_int;
    fn ref__tzset();
    fn ref__get_tzname(
        ret: *mut usize, buf: *mut c_char, bufsize: usize, index: c_int,
    ) -> c_int;
    fn ref___p__daylight() -> *mut c_int;
    fn ref___p__dstbias() -> *mut MsvcrtLong;
    fn ref___p__timezone() -> *mut MsvcrtLong;
    fn ref___p__tzname() -> *mut *mut c_char;

    // ── the harness's knobs and per-side accessors ──
    fn ref_time_set_now(ticks: u64);
    fn ref_time_ticks_from_unix(unix_secs: i64, millis: c_uint) -> u64;
    fn ref_time_set_zone(preset: c_int);
    fn ref_time_zone_count() -> c_int;
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
}

/// A value no msvcrt path ever stores, so "errno untouched" is distinguishable from "errno
/// set to 0". No EINVAL/ERANGE constants here on purpose: every errno observation is
/// `subject == reference`, so naming the expected value would put the harness's opinion in
/// between two implementations that can simply be asked.
const ERRNO_SEED: c_int = -31337;

// ── the calendar, computed rather than tabulated ─────────────────────────────

/// Days from 1970-01-01 to y-m-d, proleptic Gregorian (Howard Hinnant's algorithm — the
/// same one timeimpl.c uses, and deliberately so: the fixture's instants have to be
/// expressible without trusting the code under test to convert them).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn unix_at(y: i64, m: i64, d: i64, h: i64, mi: i64, s: i64) -> i64 {
    days_from_civil(y, m, d) * 86400 + h * 3600 + mi * 60 + s
}

/// The instants every family is driven at.
///
/// Chosen so each is a BOUNDARY rather than a sample. In order: the bottom of the FILETIME
/// epoch (1601; `ref_time_set_now(0)` means "unfrozen", so the lowest usable instant is one
/// second in), either side of the Unix epoch, a leap day in a leap century and one in an
/// ordinary leap year, the last day of a NON-leap century year, both US Pacific transitions
/// to the second, both Sydney transitions (which wrap the year end), the 32-bit `time_t`
/// overflow to the second, and `_MAX__TIME64_T` — the last instant msvcrt's 64-bit time_t
/// admits, which six of time.c's range checks compare against.
fn instants() -> Vec<(&'static str, i64)> {
    vec![
        ("1601-01-01T00:00:01Z (FILETIME epoch)", unix_at(1601, 1, 1, 0, 0, 1)),
        ("1969-12-31T23:59:59Z", -1),
        ("1970-01-01T00:00:00Z (Unix epoch)", 0),
        ("1970-01-01T00:00:01Z", 1),
        ("2000-02-29T12:00:00Z (leap century)", unix_at(2000, 2, 29, 12, 0, 0)),
        ("2024-02-29T23:59:59Z (leap day)", unix_at(2024, 2, 29, 23, 59, 59)),
        ("2100-02-28T23:59:59Z (non-leap century)", unix_at(2100, 2, 28, 23, 59, 59)),
        ("2024-03-10T09:59:59Z (US DST start -1s)", unix_at(2024, 3, 10, 9, 59, 59)),
        ("2024-03-10T10:00:00Z (US DST start)", unix_at(2024, 3, 10, 10, 0, 0)),
        ("2024-11-03T08:59:59Z (US DST end -1s)", unix_at(2024, 11, 3, 8, 59, 59)),
        ("2024-11-03T09:00:00Z (US DST end)", unix_at(2024, 11, 3, 9, 0, 0)),
        ("2024-10-05T15:59:59Z (AU DST start -1s)", unix_at(2024, 10, 5, 15, 59, 59)),
        ("2024-10-05T16:00:00Z (AU DST start)", unix_at(2024, 10, 5, 16, 0, 0)),
        ("2024-04-06T15:59:59Z (AU DST end -1s)", unix_at(2024, 4, 6, 15, 59, 59)),
        ("2024-04-06T16:00:00Z (AU DST end)", unix_at(2024, 4, 6, 16, 0, 0)),
        ("2038-01-19T03:14:07Z (INT32_MAX)", 2_147_483_647),
        ("2038-01-19T03:14:08Z (INT32_MAX+1)", 2_147_483_648),
        ("3000-12-31T23:59:59Z (_MAX__TIME64_T)", 32_535_215_999),
    ]
}

/// The zone settings. `-1` is `GetTimeZoneInformation` FAILING, which is a real Windows
/// outcome and the only way `_tzset`'s early-out arm is reachable.
const ZONES: [(&str, c_int); 5] = [
    ("UTC", 0),
    ("US Pacific", 1),
    ("India (+5:30)", 2),
    ("Sydney (southern)", 3),
    ("GetTimeZoneInformation FAILS", -1),
];

/// `TZ` values for `_tzset`'s OTHER branch — the hand-rolled parser, which the zone presets
/// can never reach because it runs INSTEAD of `GetTimeZoneInformation`.
///
/// Every entry is at least three characters, and that is a deliberate limit rather than an
/// oversight: `_tzset` does `lstrcpynA(tzname[0], tz, 3); tz += 3;`, so a two-character `TZ`
/// steps the cursor one past the string's NUL and parses whatever the environment block
/// happens to hold there. Both sides would read the same byte and the row would pass, but it
/// would be passing on undefined behaviour, and a row that depends on the allocator is not a
/// row. What IS covered: the sign arms, the `hh`/`hh:mm`/`hh:mm:ss` shapes, a daylight
/// abbreviation present and absent (which is what `__daylight` is set from — it is `*tz`,
/// i.e. truthiness of the FIRST BYTE, not a parse), and a bare zone with no offset at all.
const TZ_VALUES: [&str; 8] = [
    "UTC0",
    "GMT0BST",
    "PST8PDT",
    "EST-5EDT",
    "IST-5:30",
    "XYZ+11:22:33ABC",
    "ABC",
    "ABC0:0:0",
];

// ── row plumbing ─────────────────────────────────────────────────────────────

static mut ROWS: u64 = 0;

fn row() {
    unsafe { ROWS += 1 };
}

fn rows() -> u64 {
    unsafe { *core::ptr::addr_of!(ROWS) }
}

/// Compare one observation across the two sides. Every row goes through here so the count
/// is honest — a suite that reports "all green" without saying how much it looked at is
/// indistinguishable from one that looked at nothing.
fn both<T: PartialEq + core::fmt::Debug>(what: &str, cfg: &str, subj: T, refr: T) {
    row();
    assert_eq!(subj, refr, "{what} [{cfg}]: subject != reference");
}

/// The two side-effect channels every msvcrt entry point can touch, captured immediately
/// after the call so a later call cannot overwrite them.
#[derive(PartialEq, Eq, Debug)]
struct Side {
    errno: c_int,
    pmt: c_int,
}

unsafe fn arm_subj() {
    subj_errno_set(ERRNO_SEED);
    subj_invalid_pmt_reset();
}

unsafe fn arm_ref() {
    ref_errno_set(ERRNO_SEED);
    ref_invalid_pmt_reset();
}

unsafe fn take_subj() -> Side {
    Side { errno: subj_errno_get(), pmt: subj_invalid_pmt_count() }
}

unsafe fn take_ref() -> Side {
    Side { errno: ref_errno_get(), pmt: ref_invalid_pmt_count() }
}

/// A NUL-terminated C string read back as bytes, with a hard cap so a missing terminator is
/// a bounded failure rather than a walk off the end of the process.
unsafe fn cstr(p: *const c_char, cap: usize) -> Option<Vec<u8>> {
    if p.is_null() {
        return None;
    }
    let mut out = Vec::new();
    for i in 0..cap {
        let b = *p.add(i) as u8;
        if b == 0 {
            return Some(out);
        }
        out.push(b);
    }
    panic!("string of more than {cap} bytes with no terminator");
}

unsafe fn wstr(p: *const u16, cap: usize) -> Option<Vec<u16>> {
    if p.is_null() {
        return None;
    }
    let mut out = Vec::new();
    for i in 0..cap {
        let w = *p.add(i);
        if w == 0 {
            return Some(out);
        }
        out.push(w);
    }
    panic!("wide string of more than {cap} units with no terminator");
}

fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

/// A buffer poisoned with a byte no conversion produces, so "wrote nothing" and "wrote a
/// terminator" are distinguishable and so an overrun past the declared size is visible.
///
/// 0xAA and 0xAAAA, the same values printf_shells.rs and the scanf suite use. Not a
/// preference: `scan_const_contradictions` flags a same-named constant that disagrees
/// across files, and it is right to — a reader who learns what POISON means in one harness
/// must not have to re-learn it in the next.
const POISON: u8 = 0xAA;
const POISON_W: u16 = 0xAAAA;

fn poisoned(n: usize) -> Vec<u8> {
    vec![POISON; n]
}

fn wpoisoned(n: usize) -> Vec<u16> {
    vec![POISON_W; n]
}

/// Point both sides at one instant and one zone, and re-derive each side's cached zone
/// state from it.
///
/// The explicit `_tzset()` pair is load-bearing. time.c caches the
/// `TIME_ZONE_INFORMATION` in a file static and `_tzset_init()` fills it exactly once per
/// side, so changing the knob without this leaves both sides answering from the PREVIOUS
/// zone — identically, so every row would still pass while grading the wrong preset.
unsafe fn configure(zone: c_int, secs: i64, millis: c_uint) {
    ref_time_set_zone(zone);
    ref_time_set_now(ref_time_ticks_from_unix(secs, millis));
    _tzset();
    ref__tzset();
}

// ── the families ─────────────────────────────────────────────────────────────

/// `_time64` / `_time32`, both arms of the optional out-parameter.
///
/// `_time32` is the one with something to say: it is `(__time32_t)_time64(...)`, so at
/// 2038-01-19T03:14:08Z it TRUNCATES, and the row at that instant is what proves the
/// truncation survived translation rather than being widened into correctness.
unsafe fn grade_time(cfg: &str) {
    arm_subj();
    let s_ret = _time64(core::ptr::null_mut());
    let s_side = take_subj();
    arm_ref();
    let r_ret = ref__time64(core::ptr::null_mut());
    let r_side = take_ref();
    both("_time64(NULL)", cfg, s_ret, r_ret);
    both("_time64(NULL) side", cfg, s_side, r_side);

    // ONE sentinel value on both sides, not two. Seeding them differently would make
    // "neither side wrote the out-parameter" look like a divergence — which is a real
    // outcome here (`_get_tzname` below leaves `*ret` alone on every rejection arm) and
    // must compare EQUAL. i64::MIN is a value no instant in the fixture produces, so an
    // untouched slot is still distinguishable from a written one.
    let (mut s_out, mut r_out) = (i64::MIN, i64::MIN);
    both("_time64(&t)", cfg, _time64(&mut s_out), ref__time64(&mut r_out));
    both("_time64 out", cfg, s_out, r_out);

    both("_time32(NULL)", cfg, _time32(core::ptr::null_mut()), ref__time32(core::ptr::null_mut()));
    let (mut s32, mut r32) = (i32::MIN, i32::MIN);
    both("_time32(&t)", cfg, _time32(&mut s32), ref__time32(&mut r32));
    both("_time32 out", cfg, s32, r32);
}

/// `clock` — elapsed 100ns ticks since `msvcrt_init_clock`, divided down to
/// `CLOCKS_PER_SEC` units.
///
/// Two things are being watched, and neither is the arithmetic. The UNIT: msvcrt's
/// `CLOCKS_PER_SEC` is 1000 and glibc's is 1000000, so an unpinned reference would answer in
/// microseconds. And the WIDTH: `clock_t` is 32 bits on Windows and 64 on the host, so the
/// long-elapsed rows below are the ones that prove the reference truncates where msvcrt
/// does. Both are pinned in the reference TU's prelude; these rows are what would notice if
/// a future change unpinned them.
unsafe fn grade_clock(zone: c_int) {
    let base = unix_at(2024, 6, 1, 0, 0, 0);
    ref_time_set_zone(zone);
    ref_time_set_now(ref_time_ticks_from_unix(base, 0));
    msvcrt_init_clock();
    ref_msvcrt_init_clock();

    for (label, secs, ms) in [
        ("+0", 0i64, 0u32),
        ("+1ms", 0, 1),
        ("+999ms", 0, 999),
        ("+1s", 1, 0),
        ("+1h", 3600, 0),
        // Past 2^31 milliseconds — 24.86 days — which is where a 32-bit clock_t wraps and
        // a 64-bit one does not.
        ("+24d (under the wrap)", 24 * 86400, 0),
        ("+25d (past the wrap)", 25 * 86400, 0),
        ("+60d (well past)", 60 * 86400, 0),
    ] {
        ref_time_set_now(ref_time_ticks_from_unix(base + secs, ms));
        both(&format!("clock() {label}"), "clock", clock(), ref_clock());
    }
}

/// `_difftime64` / `_difftime32` — a subtraction promoted to double. The interesting rows
/// are the ones whose difference does not fit the operand type.
unsafe fn grade_difftime() {
    let pairs64: [(i64, i64); 8] = [
        (0, 0),
        (1, 0),
        (0, 1),
        (32_535_215_999, 0),
        (0, 32_535_215_999),
        (-1, 1),
        (i64::MAX / 2, -(i64::MAX / 2)),
        (2_147_483_647, -2_147_483_648),
    ];
    for (a, b) in pairs64 {
        both(
            &format!("_difftime64({a},{b})"),
            "difftime",
            _difftime64(a, b).to_bits(),
            ref__difftime64(a, b).to_bits(),
        );
    }
    let pairs32: [(i32, i32); 6] = [
        (0, 0),
        (1, 0),
        (0, 1),
        (i32::MAX, 0),
        (i32::MAX, i32::MIN),
        (i32::MIN, i32::MAX),
    ];
    for (a, b) in pairs32 {
        both(
            &format!("_difftime32({a},{b})"),
            "difftime",
            _difftime32(a, b).to_bits(),
            ref__difftime32(a, b).to_bits(),
        );
    }
}

/// The broken-down-time family: `_gmtime*` and `_localtime*`, both the pointer-returning
/// and the `_s` forms.
///
/// The pointer forms return an interior pointer into the caller's own thread data, so the
/// ADDRESS is meaningless across sides — but the aliasing is not, so each row checks that
/// the same side hands back the same slot twice and that `_gmtime64` and `_localtime64`
/// share it (they do: one `time_buffer` per thread, which is why the classic
/// print-both-in-one-printf bug prints one string twice).
unsafe fn grade_broken_down(cfg: &str, secs: i64) {
    // ── the 64-bit pointer forms ──
    arm_subj();
    let s_g = _gmtime64(&secs);
    let s_g_val = if s_g.is_null() { None } else { Some((*s_g).nine()) };
    let s_side = take_subj();
    arm_ref();
    let r_g = ref__gmtime64(&secs);
    let r_g_val = if r_g.is_null() { None } else { Some((*r_g).nine()) };
    let r_side = take_ref();
    both("_gmtime64", cfg, s_g_val, r_g_val);
    both("_gmtime64 side", cfg, s_side, r_side);
    both("_gmtime64 null-ness", cfg, s_g.is_null(), r_g.is_null());

    arm_subj();
    let s_l = _localtime64(&secs);
    let s_l_val = if s_l.is_null() { None } else { Some((*s_l).nine()) };
    let s_side = take_subj();
    arm_ref();
    let r_l = ref__localtime64(&secs);
    let r_l_val = if r_l.is_null() { None } else { Some((*r_l).nine()) };
    let r_side = take_ref();
    both("_localtime64", cfg, s_l_val, r_l_val);
    both("_localtime64 side", cfg, s_side, r_side);
    both("_localtime64 null-ness", cfg, s_l.is_null(), r_l.is_null());

    // The aliasing contract, per side. Only meaningful when both calls produced a buffer.
    if !s_g.is_null() && !s_l.is_null() {
        both("gmtime/localtime share one thread buffer", cfg, s_g == s_l, r_g == r_l);
    }

    // ── the 64-bit `_s` forms ──
    let (mut s_out, mut r_out) = (CTm::default(), CTm::default());
    arm_subj();
    let s_rc = _gmtime64_s(&mut s_out, &secs);
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__gmtime64_s(&mut r_out, &secs);
    let r_side = take_ref();
    both("_gmtime64_s rc", cfg, s_rc, r_rc);
    both("_gmtime64_s out", cfg, s_out.nine(), r_out.nine());
    both("_gmtime64_s side", cfg, s_side, r_side);

    let (mut s_out, mut r_out) = (CTm::default(), CTm::default());
    arm_subj();
    let s_rc = _localtime64_s(&mut s_out, &secs);
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__localtime64_s(&mut r_out, &secs);
    let r_side = take_ref();
    both("_localtime64_s rc", cfg, s_rc, r_rc);
    both("_localtime64_s out", cfg, s_out.nine(), r_out.nine());
    both("_localtime64_s side", cfg, s_side, r_side);

    // The NULL-argument arms, which are pure `MSVCRT_CHECK_PMT` and so are graded by the
    // invalid-parameter COUNT as much as by the return value.
    arm_subj();
    let s_rc = _gmtime64_s(core::ptr::null_mut(), &secs);
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__gmtime64_s(core::ptr::null_mut(), &secs);
    let r_side = take_ref();
    both("_gmtime64_s(NULL,t) rc", cfg, s_rc, r_rc);
    both("_gmtime64_s(NULL,t) side", cfg, s_side, r_side);

    let (mut s_out, mut r_out) = (CTm::default(), CTm::default());
    arm_subj();
    let s_rc = _localtime64_s(&mut s_out, core::ptr::null());
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__localtime64_s(&mut r_out, core::ptr::null());
    let r_side = take_ref();
    both("_localtime64_s(buf,NULL) rc", cfg, s_rc, r_rc);
    both("_localtime64_s(buf,NULL) out", cfg, s_out.nine(), r_out.nine());
    both("_localtime64_s(buf,NULL) side", cfg, s_side, r_side);

    // ── the 32-bit family, at the truncated instant ──
    let secs32 = secs as i32;
    arm_subj();
    let s_g = _gmtime32(&secs32);
    let s_g_val = if s_g.is_null() { None } else { Some((*s_g).nine()) };
    let s_side = take_subj();
    arm_ref();
    let r_g = ref__gmtime32(&secs32);
    let r_g_val = if r_g.is_null() { None } else { Some((*r_g).nine()) };
    let r_side = take_ref();
    both("_gmtime32", cfg, s_g_val, r_g_val);
    both("_gmtime32 side", cfg, s_side, r_side);

    arm_subj();
    let s_l = _localtime32(&secs32);
    let s_l_val = if s_l.is_null() { None } else { Some((*s_l).nine()) };
    let s_side = take_subj();
    arm_ref();
    let r_l = ref__localtime32(&secs32);
    let r_l_val = if r_l.is_null() { None } else { Some((*r_l).nine()) };
    let r_side = take_ref();
    both("_localtime32", cfg, s_l_val, r_l_val);
    both("_localtime32 side", cfg, s_side, r_side);

    let (mut s_out, mut r_out) = (CTm::default(), CTm::default());
    arm_subj();
    let s_rc = _gmtime32_s(&mut s_out, &secs32);
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__gmtime32_s(&mut r_out, &secs32);
    let r_side = take_ref();
    both("_gmtime32_s rc", cfg, s_rc, r_rc);
    both("_gmtime32_s out", cfg, s_out.nine(), r_out.nine());
    both("_gmtime32_s side", cfg, s_side, r_side);

    let (mut s_out, mut r_out) = (CTm::default(), CTm::default());
    arm_subj();
    let s_rc = _localtime32_s(&mut s_out, &secs32);
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__localtime32_s(&mut r_out, &secs32);
    let r_side = take_ref();
    both("_localtime32_s rc", cfg, s_rc, r_rc);
    both("_localtime32_s out", cfg, s_out.nine(), r_out.nine());
    both("_localtime32_s side", cfg, s_side, r_side);
}

/// The composition family: `_mktime*` (local time in) and `_mkgmtime*` (UTC in).
///
/// These take a caller-owned `struct tm` and REWRITE IT IN PLACE — normalising out-of-range
/// fields and filling `tm_wday`/`tm_yday` — so the row has to compare the struct afterwards
/// as well as the return value. An implementation that returned the right instant without
/// normalising would score MATCH on the return alone.
///
/// The inputs deliberately include denormalised fields (month 13, day 0, hour 25, second
/// 61), which is the whole point of `mktime`: it is specified to accept them.
unsafe fn grade_mktime(cfg: &str) {
    let cases: [(&str, [c_int; 6]); 12] = [
        ("epoch", [0, 0, 0, 1, 0, 70]),
        ("2024-02-29 12:00", [0, 0, 12, 29, 1, 124]),
        ("2024-03-10 02:30 (US spring-forward gap)", [0, 30, 2, 10, 2, 124]),
        ("2024-11-03 01:30 (US fall-back overlap)", [0, 30, 1, 3, 10, 124]),
        ("2024-10-06 02:30 (AU spring-forward gap)", [0, 30, 2, 6, 9, 124]),
        ("month 13 (normalises to next year)", [0, 0, 0, 1, 12, 124]),
        ("month -1", [0, 0, 0, 1, -1, 124]),
        ("day 0", [0, 0, 0, 0, 0, 124]),
        ("day 32", [0, 0, 0, 32, 0, 124]),
        ("hour 25", [0, 0, 25, 1, 0, 124]),
        ("second 61", [61, 0, 0, 1, 0, 124]),
        ("1969 (before the epoch)", [0, 0, 0, 1, 0, 69]),
    ];
    for (label, [sec, min, hour, mday, mon, year]) in cases {
        for isdst in [-1, 0, 1] {
            let mk = || CTm {
                tm_sec: sec,
                tm_min: min,
                tm_hour: hour,
                tm_mday: mday,
                tm_mon: mon,
                tm_year: year,
                tm_wday: 0,
                tm_yday: 0,
                tm_isdst: isdst,
                _pad: [0; 2],
            };
            let what = format!("{label} isdst={isdst}");

            let (mut s, mut r) = (mk(), mk());
            arm_subj();
            let s_ret = _mktime64(&mut s);
            let s_side = take_subj();
            arm_ref();
            let r_ret = ref__mktime64(&mut r);
            let r_side = take_ref();
            both(&format!("_mktime64 {what}"), cfg, s_ret, r_ret);
            both(&format!("_mktime64 {what} normalised"), cfg, s.nine(), r.nine());
            both(&format!("_mktime64 {what} side"), cfg, s_side, r_side);

            let (mut s, mut r) = (mk(), mk());
            arm_subj();
            let s_ret = _mkgmtime64(&mut s);
            let s_side = take_subj();
            arm_ref();
            let r_ret = ref__mkgmtime64(&mut r);
            let r_side = take_ref();
            both(&format!("_mkgmtime64 {what}"), cfg, s_ret, r_ret);
            both(&format!("_mkgmtime64 {what} normalised"), cfg, s.nine(), r.nine());
            both(&format!("_mkgmtime64 {what} side"), cfg, s_side, r_side);

            let (mut s, mut r) = (mk(), mk());
            both(&format!("_mktime32 {what}"), cfg, _mktime32(&mut s), ref__mktime32(&mut r));
            both(&format!("_mktime32 {what} normalised"), cfg, s.nine(), r.nine());

            let (mut s, mut r) = (mk(), mk());
            both(
                &format!("_mkgmtime32 {what}"),
                cfg,
                _mkgmtime32(&mut s),
                ref__mkgmtime32(&mut r),
            );
            both(&format!("_mkgmtime32 {what} normalised"), cfg, s.nine(), r.nine());
        }
    }

    // NO `_mktime64(NULL)` ROW, and that is a finding rather than an omission. Unlike
    // `_gmtime64_s`, `_ftime64_s` and `strftime` — all of which reject a null argument —
    // `mktime_helper` opens with `ret = mstm->tm_year + mstm->tm_mon/12;` and has no guard
    // at all, so msvcrt itself segfaults on a null `struct tm`. The Rust reproduces that
    // exactly, which is correct for a faithful port and useless as a row: what the row
    // would compare is two crashes. Same shape as math.c's `_fcvt_s` infinite loop.
}

/// `asctime` / `_wasctime` and the four `_ctime*` shells that wrap them.
///
/// The fixed 26-byte result is the point: `asctime_s` is specified to demand exactly that
/// much room, so the sizes below walk 0, 1, 25, 26, 27 — the arm either side of the
/// boundary and the boundary itself. The `_s` forms also CLEAR the buffer on rejection,
/// which the poison byte makes visible.
unsafe fn grade_asctime(cfg: &str, secs: i64) {
    // Both sides format the SAME broken-down time. Taken from the reference, which the
    // `_gmtime64_s` rows above have already compared against the subject's — so this is a
    // known-equal input rather than a second, unrelated conversion.
    let mut tm_r = CTm::default();
    if ref__gmtime64_s(&mut tm_r, &secs) != 0 {
        return; // out of gmtime's domain at this instant; the rc was graded above.
    }
    let tm_s = tm_r;

    arm_subj();
    let s_p = asctime(&tm_s);
    let s_txt = cstr(s_p, 64);
    let s_side = take_subj();
    arm_ref();
    let r_p = ref_asctime(&tm_r);
    let r_txt = cstr(r_p, 64);
    let r_side = take_ref();
    both("asctime", cfg, s_txt, r_txt);
    both("asctime side", cfg, s_side, r_side);

    // The buffer is per-thread and REUSED: a second call must hand back the same slot.
    if !s_p.is_null() && !r_p.is_null() {
        both("asctime buffer is stable", cfg, asctime(&tm_s) == s_p, ref_asctime(&tm_r) == r_p);
    }

    let s_wp = _wasctime(&tm_s);
    let s_w = wstr(s_wp, 64);
    let r_wp = ref__wasctime(&tm_r);
    let r_w = wstr(r_wp, 64);
    both("_wasctime", cfg, s_w, r_w);
    if !s_wp.is_null() && !r_wp.is_null() {
        both(
            "_wasctime buffer is stable",
            cfg,
            _wasctime(&tm_s) == s_wp,
            ref__wasctime(&tm_r) == r_wp,
        );
    }

    for size in [0usize, 1, 25, 26, 27, 64] {
        let (mut sb, mut rb) = (poisoned(80), poisoned(80));
        arm_subj();
        let s_rc = asctime_s(sb.as_mut_ptr() as *mut c_char, size, &tm_s);
        let s_side = take_subj();
        arm_ref();
        let r_rc = ref_asctime_s(rb.as_mut_ptr() as *mut c_char, size, &tm_r);
        let r_side = take_ref();
        both(&format!("asctime_s(size={size}) rc"), cfg, s_rc, r_rc);
        both(&format!("asctime_s(size={size}) buf"), cfg, sb, rb);
        both(&format!("asctime_s(size={size}) side"), cfg, s_side, r_side);

        let (mut sb, mut rb) = (wpoisoned(80), wpoisoned(80));
        arm_subj();
        let s_rc = _wasctime_s(sb.as_mut_ptr(), size, &tm_s);
        let s_side = take_subj();
        arm_ref();
        let r_rc = ref__wasctime_s(rb.as_mut_ptr(), size, &tm_r);
        let r_side = take_ref();
        both(&format!("_wasctime_s(size={size}) rc"), cfg, s_rc, r_rc);
        both(&format!("_wasctime_s(size={size}) buf"), cfg, sb, rb);
        both(&format!("_wasctime_s(size={size}) side"), cfg, s_side, r_side);
    }

    // asctime(NULL) — the CHECK_PMT arm.
    arm_subj();
    let s_p = asctime(core::ptr::null());
    let s_side = take_subj();
    arm_ref();
    let r_p = ref_asctime(core::ptr::null());
    let r_side = take_ref();
    both("asctime(NULL) null-ness", cfg, s_p.is_null(), r_p.is_null());
    both("asctime(NULL) side", cfg, s_side, r_side);

    // ── the _ctime shells: a time_t straight to a string, i.e. localtime + asctime ──
    let secs32 = secs as i32;
    arm_subj();
    let s_txt = cstr(_ctime64(&secs), 64);
    let s_side = take_subj();
    arm_ref();
    let r_txt = cstr(ref__ctime64(&secs), 64);
    let r_side = take_ref();
    both("_ctime64", cfg, s_txt, r_txt);
    both("_ctime64 side", cfg, s_side, r_side);

    both("_ctime32", cfg, cstr(_ctime32(&secs32), 64), cstr(ref__ctime32(&secs32), 64));
    both("_wctime64", cfg, wstr(_wctime64(&secs), 64), wstr(ref__wctime64(&secs), 64));
    both("_wctime32", cfg, wstr(_wctime32(&secs32), 64), wstr(ref__wctime32(&secs32), 64));

    for size in [0usize, 1, 25, 26, 27, 64] {
        let (mut sb, mut rb) = (poisoned(80), poisoned(80));
        arm_subj();
        let s_rc = _ctime64_s(sb.as_mut_ptr() as *mut c_char, size, &secs);
        let s_side = take_subj();
        arm_ref();
        let r_rc = ref__ctime64_s(rb.as_mut_ptr() as *mut c_char, size, &secs);
        let r_side = take_ref();
        both(&format!("_ctime64_s(size={size}) rc"), cfg, s_rc, r_rc);
        both(&format!("_ctime64_s(size={size}) buf"), cfg, sb, rb);
        both(&format!("_ctime64_s(size={size}) side"), cfg, s_side, r_side);

        let (mut sb, mut rb) = (poisoned(80), poisoned(80));
        both(
            &format!("_ctime32_s(size={size}) rc"),
            cfg,
            _ctime32_s(sb.as_mut_ptr() as *mut c_char, size, &secs32),
            ref__ctime32_s(rb.as_mut_ptr() as *mut c_char, size, &secs32),
        );
        both(&format!("_ctime32_s(size={size}) buf"), cfg, sb, rb);

        let (mut sb, mut rb) = (wpoisoned(80), wpoisoned(80));
        both(
            &format!("_wctime64_s(size={size}) rc"),
            cfg,
            _wctime64_s(sb.as_mut_ptr(), size, &secs),
            ref__wctime64_s(rb.as_mut_ptr(), size, &secs),
        );
        both(&format!("_wctime64_s(size={size}) buf"), cfg, sb, rb);

        let (mut sb, mut rb) = (wpoisoned(80), wpoisoned(80));
        both(
            &format!("_wctime32_s(size={size}) rc"),
            cfg,
            _wctime32_s(sb.as_mut_ptr(), size, &secs32),
            ref__wctime32_s(rb.as_mut_ptr(), size, &secs32),
        );
        both(&format!("_wctime32_s(size={size}) buf"), cfg, sb, rb);
    }
}

/// `_strdate` / `_strtime` and their wide and `_s` variants — the four that go through
/// timeimpl.c's `GetDateFormatW`/`GetTimeFormatW` picture parser.
///
/// The narrow pair are interesting for a reason the wide pair are not: they format WIDE and
/// then convert down, so the row also exercises `_wcstombs_l`. The declared buffer is 9
/// bytes and the `_s` forms check it, so the sizes walk both sides of that.
unsafe fn grade_strdate(cfg: &str) {
    let (mut sb, mut rb) = (poisoned(32), poisoned(32));
    arm_subj();
    let s_p = _strdate(sb.as_mut_ptr() as *mut c_char);
    let s_side = take_subj();
    arm_ref();
    let r_p = ref__strdate(rb.as_mut_ptr() as *mut c_char);
    let r_side = take_ref();
    both("_strdate buf", cfg, sb.clone(), rb.clone());
    both("_strdate side", cfg, s_side, r_side);
    // It returns its own argument; comparing the raw pointers across sides is meaningless,
    // comparing each to the buffer it was handed is the contract.
    both(
        "_strdate returns its argument",
        cfg,
        s_p == sb.as_mut_ptr() as *mut c_char,
        r_p == rb.as_mut_ptr() as *mut c_char,
    );

    let (mut sb, mut rb) = (poisoned(32), poisoned(32));
    both(
        "_strtime buf",
        cfg,
        {
            _strtime(sb.as_mut_ptr() as *mut c_char);
            sb
        },
        {
            ref__strtime(rb.as_mut_ptr() as *mut c_char);
            rb
        },
    );

    let (mut sb, mut rb) = (wpoisoned(32), wpoisoned(32));
    both(
        "_wstrdate buf",
        cfg,
        {
            _wstrdate(sb.as_mut_ptr());
            sb
        },
        {
            ref__wstrdate(rb.as_mut_ptr());
            rb
        },
    );

    let (mut sb, mut rb) = (wpoisoned(32), wpoisoned(32));
    both(
        "_wstrtime buf",
        cfg,
        {
            _wstrtime(sb.as_mut_ptr());
            sb
        },
        {
            ref__wstrtime(rb.as_mut_ptr());
            rb
        },
    );

    for size in [0usize, 1, 8, 9, 10, 32] {
        let (mut sb, mut rb) = (poisoned(40), poisoned(40));
        arm_subj();
        let s_rc = _strdate_s(sb.as_mut_ptr() as *mut c_char, size);
        let s_side = take_subj();
        arm_ref();
        let r_rc = ref__strdate_s(rb.as_mut_ptr() as *mut c_char, size);
        let r_side = take_ref();
        both(&format!("_strdate_s(size={size}) rc"), cfg, s_rc, r_rc);
        both(&format!("_strdate_s(size={size}) buf"), cfg, sb, rb);
        both(&format!("_strdate_s(size={size}) side"), cfg, s_side, r_side);

        let (mut sb, mut rb) = (poisoned(40), poisoned(40));
        both(
            &format!("_strtime_s(size={size}) rc"),
            cfg,
            _strtime_s(sb.as_mut_ptr() as *mut c_char, size),
            ref__strtime_s(rb.as_mut_ptr() as *mut c_char, size),
        );
        both(&format!("_strtime_s(size={size}) buf"), cfg, sb, rb);

        let (mut sb, mut rb) = (wpoisoned(40), wpoisoned(40));
        both(
            &format!("_wstrdate_s(size={size}) rc"),
            cfg,
            _wstrdate_s(sb.as_mut_ptr(), size),
            ref__wstrdate_s(rb.as_mut_ptr(), size),
        );
        both(&format!("_wstrdate_s(size={size}) buf"), cfg, sb, rb);

        let (mut sb, mut rb) = (wpoisoned(40), wpoisoned(40));
        both(
            &format!("_wstrtime_s(size={size}) rc"),
            cfg,
            _wstrtime_s(sb.as_mut_ptr(), size),
            ref__wstrtime_s(rb.as_mut_ptr(), size),
        );
        both(&format!("_wstrtime_s(size={size}) buf"), cfg, sb, rb);
    }
}

/// `_ftime*` — the whole zone state condensed into four fields, which makes it the sharpest
/// single check that the two sides agree about what the zone IS.
unsafe fn grade_ftime(cfg: &str) {
    let (mut s, mut r) = (Timeb64::default(), Timeb64::default());
    arm_subj();
    _ftime64(&mut s);
    let s_side = take_subj();
    arm_ref();
    ref__ftime64(&mut r);
    let r_side = take_ref();
    both("_ftime64", cfg, s, r);
    both("_ftime64 side", cfg, s_side, r_side);

    let (mut s, mut r) = (Timeb64::default(), Timeb64::default());
    both("_ftime64_s rc", cfg, _ftime64_s(&mut s), ref__ftime64_s(&mut r));
    both("_ftime64_s out", cfg, s, r);

    let (mut s, mut r) = (Timeb32::default(), Timeb32::default());
    _ftime32(&mut s);
    ref__ftime32(&mut r);
    both("_ftime32", cfg, s, r);

    let (mut s, mut r) = (Timeb32::default(), Timeb32::default());
    both("_ftime32_s rc", cfg, _ftime32_s(&mut s), ref__ftime32_s(&mut r));
    both("_ftime32_s out", cfg, s, r);

    // The NULL arm.
    arm_subj();
    let s_rc = _ftime64_s(core::ptr::null_mut());
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__ftime64_s(core::ptr::null_mut());
    let r_side = take_ref();
    both("_ftime64_s(NULL) rc", cfg, s_rc, r_rc);
    both("_ftime64_s(NULL) side", cfg, s_side, r_side);
}

/// The four exported zone globals and the two accessors over them.
///
/// These are what `_tzset` WRITES, so they are per-side storage and the comparison is
/// meaningful. `__p__tzname` returns a `char*[2]` whose entries point into two more
/// statics, so the row reads through them rather than comparing the pointers.
unsafe fn grade_zone_state(cfg: &str) {
    both("__p__daylight", cfg, *__p__daylight(), *ref___p__daylight());
    both("__p__timezone", cfg, *__p__timezone(), *ref___p__timezone());
    both("__p__dstbias", cfg, *__p__dstbias(), *ref___p__dstbias());

    let (s_tz, r_tz) = (__p__tzname(), ref___p__tzname());
    for i in 0..2 {
        both(
            &format!("__p__tzname[{i}]"),
            cfg,
            cstr(*s_tz.add(i), 256),
            cstr(*r_tz.add(i), 256),
        );
    }

    // `_get_tzname` copies one of those two into a caller buffer and reports the length it
    // WANTED, so the sizes walk 0 (the query form), one short, exact, and long. Index 2 is
    // out of range and takes the CHECK_PMT arm.
    for index in [0, 1, 2, -1] {
        for bufsize in [0usize, 1, 2, 3, 4, 32] {
            let (mut sb, mut rb) = (poisoned(64), poisoned(64));
            // One sentinel, both sides — see the note in grade_time. `_get_tzname`
            // leaves `*ret` untouched on all four of its rejection arms.
            let (mut s_ret, mut r_ret) = (usize::MAX, usize::MAX);
            arm_subj();
            let s_rc = _get_tzname(&mut s_ret, sb.as_mut_ptr() as *mut c_char, bufsize, index);
            let s_side = take_subj();
            arm_ref();
            let r_rc =
                ref__get_tzname(&mut r_ret, rb.as_mut_ptr() as *mut c_char, bufsize, index);
            let r_side = take_ref();
            let what = format!("_get_tzname(index={index}, bufsize={bufsize})");
            both(&format!("{what} rc"), cfg, s_rc, r_rc);
            both(&format!("{what} ret"), cfg, s_ret, r_ret);
            both(&format!("{what} buf"), cfg, sb, rb);
            both(&format!("{what} side"), cfg, s_side, r_side);
        }
    }

    // The NULL-out-parameter arm, which is a different CHECK_PMT from the index one.
    let (mut sb, mut rb) = (poisoned(64), poisoned(64));
    arm_subj();
    let s_rc = _get_tzname(
        core::ptr::null_mut(),
        sb.as_mut_ptr() as *mut c_char,
        64,
        0,
    );
    let s_side = take_subj();
    arm_ref();
    let r_rc = ref__get_tzname(
        core::ptr::null_mut(),
        rb.as_mut_ptr() as *mut c_char,
        64,
        0,
    );
    let r_side = take_ref();
    both("_get_tzname(NULL ret) rc", cfg, s_rc, r_rc);
    both("_get_tzname(NULL ret) buf", cfg, sb, rb);
    both("_get_tzname(NULL ret) side", cfg, s_side, r_side);
}

/// The formats `strftime` is driven with.
///
/// Every conversion `strftime_impl` has a `case` for, plus the shapes that are not
/// conversions: the `#` alternate flag (which changes six of them), a literal run, `%%`, an
/// unknown conversion (`%q`, which is the EINVAL arm), and a trailing `%` with nothing after
/// it. `%c`, `%x` and `%X` are the ones that reach `cloc_time_data`'s picture strings and so
/// run the SECOND formatter, `strftime_format`, over `dddd, MMMM dd, yyyy` and `HH:mm:ss`.
const FORMATS: [&str; 46] = [
    "", "%a", "%A", "%b", "%B", "%c", "%C", "%d", "%D", "%e", "%F", "%g", "%G", "%h", "%H",
    "%I", "%j", "%m", "%M", "%n", "%p", "%r", "%R", "%S", "%t", "%T", "%u", "%U", "%V", "%w",
    "%W", "%x", "%X", "%y", "%Y", "%z", "%Z", "%%", "%q", "%",
    "plain text with no conversions",
    "%Y-%m-%dT%H:%M:%SZ",
    "%#c", "%#x", "%#d", "%#j",
];

/// `strftime` / `wcsftime` / `_wcsftime_l` / `_Strftime`.
///
/// The buffer sizes are the second axis and they matter as much as the formats: every
/// conversion writes through `strftime_nstr`, which stops at `max` and sets ERANGE, so a
/// size that lands mid-conversion is the arm most likely to differ. 0 is its own case —
/// `strftime` with `max == 0` writes nothing at all, not even a terminator.
unsafe fn grade_strftime(cfg: &str, secs: i64) {
    let mut tm = CTm::default();
    if ref__localtime64_s(&mut tm, &secs) != 0 {
        return; // outside localtime's domain here; the rc was graded above.
    }

    for fmt in FORMATS {
        let cfmt = std::ffi::CString::new(fmt).unwrap();
        let wfmt = wide(fmt);
        for max in [0usize, 1, 2, 3, 5, 12, 64] {
            let what = format!("strftime({fmt:?}, max={max})");

            let (mut sb, mut rb) = (poisoned(96), poisoned(96));
            arm_subj();
            let s_ret = strftime(sb.as_mut_ptr() as *mut c_char, max, cfmt.as_ptr(), &tm);
            let s_side = take_subj();
            arm_ref();
            let r_ret = ref_strftime(rb.as_mut_ptr() as *mut c_char, max, cfmt.as_ptr(), &tm);
            let r_side = take_ref();
            both(&format!("{what} ret"), cfg, s_ret, r_ret);
            both(&format!("{what} buf"), cfg, sb, rb);
            both(&format!("{what} side"), cfg, s_side, r_side);

            let (mut sb, mut rb) = (wpoisoned(96), wpoisoned(96));
            arm_subj();
            let s_ret = wcsftime(sb.as_mut_ptr(), max, wfmt.as_ptr(), &tm);
            let s_side = take_subj();
            arm_ref();
            let r_ret = ref_wcsftime(rb.as_mut_ptr(), max, wfmt.as_ptr(), &tm);
            let r_side = take_ref();
            both(&format!("wcs{what} ret"), cfg, s_ret, r_ret);
            both(&format!("wcs{what} buf"), cfg, sb, rb);
            both(&format!("wcs{what} side"), cfg, s_side, r_side);

            let (mut sb, mut rb) = (wpoisoned(96), wpoisoned(96));
            both(
                &format!("_wcsftime_l {what} ret"),
                cfg,
                _wcsftime_l(sb.as_mut_ptr(), max, wfmt.as_ptr(), &tm, core::ptr::null_mut()),
                ref__wcsftime_l(
                    rb.as_mut_ptr(),
                    max,
                    wfmt.as_ptr(),
                    &tm,
                    core::ptr::null_mut(),
                ),
            );
            both(&format!("_wcsftime_l {what} buf"), cfg, sb, rb);

            // `_Strftime` with a NULL time_data resolves it from get_locinfo() — i.e. from
            // the SHARED cloc_time_data both locinfos point at, which is what makes the
            // month and weekday names comparable at all.
            let (mut sb, mut rb) = (poisoned(96), poisoned(96));
            both(
                &format!("_Strftime {what} ret"),
                cfg,
                _Strftime(
                    sb.as_mut_ptr() as *mut c_char,
                    max,
                    cfmt.as_ptr(),
                    &tm,
                    core::ptr::null_mut(),
                ),
                ref__Strftime(
                    rb.as_mut_ptr() as *mut c_char,
                    max,
                    cfmt.as_ptr(),
                    &tm,
                    core::ptr::null_mut(),
                ),
            );
            both(&format!("_Strftime {what} buf"), cfg, sb, rb);
        }
    }

    // The two NULL arms, which return 0 with EINVAL rather than formatting.
    let cfmt = std::ffi::CString::new("%Y").unwrap();
    let (mut sb, mut rb) = (poisoned(96), poisoned(96));
    arm_subj();
    let s_ret = strftime(sb.as_mut_ptr() as *mut c_char, 64, core::ptr::null(), &tm);
    let s_side = take_subj();
    arm_ref();
    let r_ret = ref_strftime(rb.as_mut_ptr() as *mut c_char, 64, core::ptr::null(), &tm);
    let r_side = take_ref();
    both("strftime(fmt=NULL) ret", cfg, s_ret, r_ret);
    both("strftime(fmt=NULL) buf", cfg, sb, rb);
    both("strftime(fmt=NULL) side", cfg, s_side, r_side);

    arm_subj();
    let s_ret = strftime(core::ptr::null_mut(), 64, cfmt.as_ptr(), &tm);
    let s_side = take_subj();
    arm_ref();
    let r_ret = ref_strftime(core::ptr::null_mut(), 64, cfmt.as_ptr(), &tm);
    let r_side = take_ref();
    both("strftime(str=NULL) ret", cfg, s_ret, r_ret);
    both("strftime(str=NULL) side", cfg, s_side, r_side);

    let (mut sb, mut rb) = (poisoned(96), poisoned(96));
    arm_subj();
    let s_ret = strftime(sb.as_mut_ptr() as *mut c_char, 64, cfmt.as_ptr(), core::ptr::null());
    let s_side = take_subj();
    arm_ref();
    let r_ret =
        ref_strftime(rb.as_mut_ptr() as *mut c_char, 64, cfmt.as_ptr(), core::ptr::null());
    let r_side = take_ref();
    both("strftime(tm=NULL) ret", cfg, s_ret, r_ret);
    both("strftime(tm=NULL) buf", cfg, sb, rb);
    both("strftime(tm=NULL) side", cfg, s_side, r_side);
}

/// A dense sweep of one year, at three-hour steps, comparing only `_localtime64` and
/// `_ftime64`.
///
/// The instant fixture above hits the transitions to the second because their dates were
/// worked out by hand — which is exactly the kind of hand-worked constant that stops being
/// true when a rule changes. This sweep is the check that does not depend on knowing where
/// the transitions ARE: if the two sides disagree anywhere in the year, in any zone, it
/// fails and prints the instant. 2920 points per zone, and it costs about nothing.
unsafe fn sweep_year(zone: c_int, zone_name: &str) {
    let start = unix_at(2024, 1, 1, 0, 0, 0);
    let mut disagreements = 0usize;
    let mut t = start;
    while t < start + 366 * 86400 {
        ref_time_set_now(ref_time_ticks_from_unix(t, 0));
        let (mut s, mut r) = (CTm::default(), CTm::default());
        let s_rc = _localtime64_s(&mut s, &t);
        let r_rc = ref__localtime64_s(&mut r, &t);
        if s_rc != r_rc || s.nine() != r.nine() {
            disagreements += 1;
            assert_eq!(
                (s_rc, s.nine()),
                (r_rc, r.nine()),
                "sweep [{zone_name}] diverged at unix {t}"
            );
        }
        let (mut sf, mut rf) = (Timeb64::default(), Timeb64::default());
        _ftime64(&mut sf);
        ref__ftime64(&mut rf);
        assert_eq!(sf, rf, "sweep [{zone_name}] _ftime64 diverged at unix {t}");
        row();
        row();
        t += 3 * 3600;
    }
    assert_eq!(disagreements, 0);
    let _ = zone;
}

/// Proof that the rows above are comparing OUTPUT and not two identical failures.
///
/// Every assertion in this file is `subject == reference`, and that shape has a silent
/// failure mode it cannot see: if both sides return 0 and write nothing — which is exactly
/// what happens when `locinfo->lc_time_curr` is NULL, or when the zone shim answers
/// TIME_ZONE_ID_INVALID for every preset — then every row MATCHES and the suite is green
/// over a harness that ran nothing. That is §9.73 in its purest form, and a differential
/// test cannot detect it by differencing.
///
/// So these are absolute: known text, checked against a value neither side chose. They are
/// deliberately few and deliberately obvious. If one of them fails, the substrate is wrong
/// and the 375k rows above mean nothing.
unsafe fn canaries() {
    let tm_at = |secs: i64| {
        let mut tm = CTm::default();
        assert_eq!(ref__gmtime64_s(&mut tm, &secs), 0, "canary instant is out of domain");
        tm
    };
    // 2024-03-15T13:45:07Z is a Friday in March, 13:45 local in UTC.
    let tm = tm_at(unix_at(2024, 3, 15, 13, 45, 7));
    ref_time_set_zone(0);
    ref_time_set_now(ref_time_ticks_from_unix(unix_at(2024, 3, 15, 13, 45, 7), 0));
    _tzset();
    ref__tzset();

    for (fmt, want) in [
        // The four that read cloc_time_data's string table — a NULL lc_time_curr would
        // crash, and an empty one would render nothing while both sides agreed.
        ("%A", "Friday"),
        ("%a", "Fri"),
        ("%B", "March"),
        ("%p", "PM"),
        // `%c` is the one that runs the SECOND formatter, strftime_format, over the
        // picture strings — and it is the conversion whose run-length loop was hanging
        // until this lane's first row reached it. Pinned to the exact text so a future
        // regression there is a failed canary and not a passing pair of blanks.
        ("%c", "03/15/24 13:45:07"),
        ("%x", "03/15/24"),
        ("%X", "13:45:07"),
        ("%#c", "Friday, March 15, 2024 13:45:07"),
        ("%Y-%m-%dT%H:%M:%S", "2024-03-15T13:45:07"),
    ] {
        let cfmt = std::ffi::CString::new(fmt).unwrap();
        let mut buf = poisoned(96);
        let n = strftime(buf.as_mut_ptr() as *mut c_char, 95, cfmt.as_ptr(), &tm);
        let got = String::from_utf8_lossy(&buf[..n]).to_string();
        assert_eq!(
            got, want,
            "CANARY: strftime({fmt:?}) produced {got:?}. The rows above compare the two \
             sides against each other and would report MATCH even if both produced \
             nothing — this is the check that they produced the right thing."
        );
        row();
    }

    // The zone name really does come from the preset, so the `__p__tzname` and
    // `_get_tzname` rows are comparing a string rather than two empty buffers.
    ref_time_set_zone(1);
    _tzset();
    ref__tzset();
    let names = __p__tzname();
    assert_eq!(
        cstr(*names, 256).as_deref(),
        Some(&b"Pacific Standard Time"[..]),
        "CANARY: preset 1's standard zone name did not reach _tzname. The FULL name is the \
         right answer here and the three-byte truncation is not: `lstrcpynA(tzname[0], \
         tz, 3)` is the TZ-string branch, while the GetTimeZoneInformation branch runs \
         WideCharToMultiByte into a buffer sized by `sizeof(tzname_std)`. Two branches, \
         two lengths — which is itself worth pinning."
    );
    row();
    assert_eq!(*__p__timezone(), 480 * 60, "CANARY: preset 1's bias did not reach __timezone");
    row();
    ref_time_set_zone(0);
    _tzset();
    ref__tzset();
}

fn main() {
    println!("\ntime.c differential\n");

    unsafe {
        // The four presets timeimpl.c actually carries, asserted rather than assumed: a
        // fixture that silently shrank would still pass every row it still ran.
        assert_eq!(
            ref_time_zone_count(),
            4,
            "timeimpl.c's zone preset table changed size — the fixture below names four \
             (UTC, US Pacific, India, Sydney) and each exists to reach a branch the others \
             cannot"
        );

        // No TZ for the preset half of the suite: it is what selects
        // GetTimeZoneInformation over the string parser, and a TZ inherited from the
        // developer's shell would silently take the other branch and grade the wrong half.
        std::env::remove_var("TZ");

        // Absolute checks first: if the substrate is not producing real output, the
        // 375k differential rows below are 375k comparisons of nothing against nothing.
        canaries();

        for (zone_name, zone) in ZONES {
            grade_clock(zone);
            grade_difftime();

            for (label, secs) in instants() {
                let cfg = format!("zone={zone_name} at {label}");
                configure(zone, secs, 250);
                grade_time(&cfg);
                grade_zone_state(&cfg);
                grade_ftime(&cfg);
                grade_broken_down(&cfg, secs);
                grade_asctime(&cfg, secs);
                grade_strdate(&cfg);
                grade_mktime(&cfg);
                grade_strftime(&cfg, secs);
            }
        }

        // ── the TZ parser, which no preset can reach ──
        for tz in TZ_VALUES {
            std::env::set_var("TZ", tz);
            for (label, secs) in instants() {
                let cfg = format!("TZ={tz:?} at {label}");
                // Zone 0 so that if the parser branch is ever NOT taken the fallback is a
                // definite value rather than whatever the previous iteration left.
                configure(0, secs, 250);
                grade_zone_state(&cfg);
                grade_ftime(&cfg);
                grade_broken_down(&cfg, secs);
                grade_strdate(&cfg);
            }
        }
        std::env::remove_var("TZ");

        // ── the transition sweep, which does not depend on knowing the rules ──
        for (zone_name, zone) in ZONES {
            ref_time_set_zone(zone);
            _tzset();
            ref__tzset();
            sweep_year(zone, zone_name);
        }

        println!("time: {} rows", rows());
    }
}
