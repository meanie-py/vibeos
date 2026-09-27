//! time/timeset.cpp: the default-time-zone globals `_daylight`, `_dstbias`,
//! `_timezone`, `_tzname` and their `_get_*`/accessor functions.
//!
//! State isolation is compiled out in this tree
//! (`sdk/lib/ucrt_reactos/inc/corecrt_internal_state_isolation.h`:
//! `state_index_count == 1`), so every `__crt_state_management::dual_state_global<T>`
//! here is exactly one `T` -- see `mbctype.rs`/`locale_data.rs` for the same
//! reduction elsewhere. The `#ifdef _CRT_GLOBAL_STATE_ISOLATION` "os" row of
//! `tzname_states`/`w_tzname_states` and the second state slot never exist in
//! this build, so they are not ported.
//!
//! `_daylight`, `_dstbias`, `_timezone` and `_tzname` are NOT referenced by
//! plain C name from any other UCRT file: `time.h` makes each one a macro over
//! its accessor (`#define _timezone (*__timezone())`, `#define _tzname (__tzname())`,
//! etc.), so `tzset.cpp` and everyone else goes through `__timezone()`/`__tzname()`
//! and friends, never the raw symbol. `_timezone`/`_daylight`/`_dstbias` are
//! nonetheless given here with the exact C names (verified with
//! `tools/compare_coff_data.py` against `timeset.cpp.obj`'s own `_timezone`/
//! `_daylight`/`_dstbias`, which are NOT name-mangled: they are plain
//! namespace-scope C++ variables, and Itanium-family mangling does not touch
//! those) purely for byte-level fidelity; nothing links against them by name.
//!
//! `__dnames`/`__mnames` ARE depended on by exact name: `time/asctime.cpp`
//! (still C, defines `asctime`/`_wasctime`, called through by `ctime.rs`) reads
//! `__dnames + tm_wday*3` / `__mnames + tm_mon*3` directly.
//!
//! ## Details that are silent if wrong
//!
//! * **`_TZ_STRINGS_SIZE` is 64**, both narrow and wide (i.e. 128 bytes wide) --
//!   "NT POSIX's TZNAME_MAX is only 10", per the C's own comment, so this is
//!   generous padding, not a tight buffer.
//! * **The statics are zero until `__acrt_initialize_timeset` runs**, exactly
//!   as in the C: `dual_state_global<T>` is a class type with a non-trivial
//!   default constructor that zero-initialises `_value`, so `_timezone`/
//!   `_daylight`/`_dstbias` are genuinely `0` in `timeset.cpp.obj`'s `.bss` --
//!   the Pacific/DST-on/-3600s defaults are set only when the `.CRT$XIC`
//!   initializer (`initializers/timeset_initializer.cpp`, unchanged, still
//!   calling `extern "C" int __cdecl __acrt_initialize_timeset();`) runs at
//!   process startup. Baking the target values into the `static`s' own
//!   initializers instead (as a first pass here did) matches the post-init
//!   runtime value but NOT the byte image `compare_coff_data.py` checks, and
//!   would diverge from the C during the (believed empty, but unverified)
//!   window before `.CRT$XIC` runs -- so these start at `0`, like the C.
//! * **`_get_tzname` NUL-terminates `buffer[0]` on the FIRST validation success,
//!   before the `length`/`index` checks**, so a caller with a bad `index` still
//!   gets an empty (not untouched) buffer.
//! * **`_get_tzname` with `buffer == nullptr` is a size query**: it validates,
//!   fills `*length`, and returns 0 without ever consulting `size_in_bytes`
//!   (which must be 0 in that case) or calling `strcpy_s`.

#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_long};
use core::ptr::addr_of_mut;

// timeset.cpp carried `_CRT_LINKER_FORCE_INCLUDE(__acrt_timeset_initializer)`; see force_include.rs.
crate::force_include!(__acrt_timeset_initializer);

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

const _TZ_STRINGS_SIZE: usize = 64;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn strlen(s: *const u8) -> usize;
    fn strcpy_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
}

/// `_VALIDATE_RETURN_ERRCODE(expr, err)`.
unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

// ── the state (exact C names; see the module doc for why) ─────────────────────

#[no_mangle]
pub static mut _timezone: c_long = 0; // Pacific Time Zone (set by __acrt_initialize_timeset)
#[no_mangle]
pub static mut _daylight: c_int = 0; // Daylight Saving Time (DST) in timezone (ditto)
#[no_mangle]
pub static mut _dstbias: c_long = 0; // DST offset in seconds (ditto)

const fn ascii64(s: &[u8]) -> [u8; _TZ_STRINGS_SIZE] {
    let mut buf = [0u8; _TZ_STRINGS_SIZE];
    let mut i = 0;
    while i < s.len() {
        buf[i] = s[i];
        i += 1;
    }
    buf
}

const fn wide64(s: &[u8]) -> [u16; _TZ_STRINGS_SIZE] {
    let mut buf = [0u16; _TZ_STRINGS_SIZE];
    let mut i = 0;
    while i < s.len() {
        buf[i] = s[i] as u16;
        i += 1;
    }
    buf
}

/// `tzstd_program`/`tzdst_program`/`w_tzstd_program`/`w_tzdst_program`: file-local
/// in the C (internal linkage, so not name-matched here), kept for the buffers
/// `_get_tzname`/`__tzname`/`__wide_tzname` hand out.
static mut TZSTD_PROGRAM: [u8; _TZ_STRINGS_SIZE] = ascii64(b"PST");
static mut TZDST_PROGRAM: [u8; _TZ_STRINGS_SIZE] = ascii64(b"PDT");
static mut W_TZSTD_PROGRAM: [u16; _TZ_STRINGS_SIZE] = wide64(b"PST");
static mut W_TZDST_PROGRAM: [u16; _TZ_STRINGS_SIZE] = wide64(b"PDT");

/// `tzname_states[0]` / `w_tzname_states[0]` -- the single (state isolation is
/// off) row `_tzname.value()` / `w_tzname.value()` is a pointer to.
static mut TZNAME_ROW: [*mut u8; 2] = [
    addr_of_mut!(TZSTD_PROGRAM) as *mut u8,
    addr_of_mut!(TZDST_PROGRAM) as *mut u8,
];
static mut W_TZNAME_ROW: [*mut u16; 2] = [
    addr_of_mut!(W_TZSTD_PROGRAM) as *mut u16,
    addr_of_mut!(W_TZDST_PROGRAM) as *mut u16,
];

/// Day names, three-character abbreviations strung together (`SunMonTueWed...`).
#[no_mangle]
pub static __dnames: [u8; 22] = *b"SunMonTueWedThuFriSat\0";

/// Month names, three-character abbreviations strung together (`JanFebMar...`).
#[no_mangle]
pub static __mnames: [u8; 37] = *b"JanFebMarAprMayJunJulAugSepOctNovDec\0";

/// `__acrt_initialize_timeset`. Called from the CRT's `.CRT$XIC` initializer
/// table (`initializers/timeset_initializer.cpp`, unchanged) exactly once per
/// process; see the module doc for why this is a no-op in effect.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_timeset() -> c_int {
    _timezone = 8 * 3600;
    _daylight = 1;
    _dstbias = -3600;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_daylight(result: *mut c_int) -> c_int {
    if !validate(!result.is_null(), EINVAL) {
        return EINVAL;
    }
    *result = _daylight;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_dstbias(result: *mut c_long) -> c_int {
    if !validate(!result.is_null(), EINVAL) {
        return EINVAL;
    }
    *result = _dstbias;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_timezone(result: *mut c_long) -> c_int {
    if !validate(!result.is_null(), EINVAL) {
        return EINVAL;
    }
    *result = _timezone;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_tzname(length: *mut usize, buffer: *mut u8, size_in_bytes: usize, index: c_int) -> c_int {
    let size_ok = (!buffer.is_null() && size_in_bytes > 0) || (buffer.is_null() && size_in_bytes == 0);
    if !validate(size_ok, EINVAL) {
        return EINVAL;
    }

    if !buffer.is_null() {
        *buffer = 0;
    }

    if !validate(!length.is_null(), EINVAL) {
        return EINVAL;
    }
    if !validate(index == 0 || index == 1, EINVAL) {
        return EINVAL;
    }

    let name = TZNAME_ROW[index as usize];
    *length = strlen(name) + 1;

    if buffer.is_null() {
        return 0;
    }
    if *length > size_in_bytes {
        return ERANGE;
    }
    strcpy_s(buffer, size_in_bytes, name)
}

#[no_mangle]
pub unsafe extern "C" fn __daylight() -> *mut c_int {
    addr_of_mut!(_daylight)
}

#[no_mangle]
pub unsafe extern "C" fn __dstbias() -> *mut c_long {
    addr_of_mut!(_dstbias)
}

#[no_mangle]
pub unsafe extern "C" fn __timezone() -> *mut c_long {
    addr_of_mut!(_timezone)
}

#[no_mangle]
pub unsafe extern "C" fn __tzname() -> *mut *mut u8 {
    addr_of_mut!(TZNAME_ROW) as *mut *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn __wide_tzname() -> *mut *mut u16 {
    addr_of_mut!(W_TZNAME_ROW) as *mut *mut u16
}
