//! Differential harness against the **real compiled msvcrt C reference**.
//!
//! `build.rs` compiles the genuine `dll/win32/msvcrt/string.c` for the host
//! with each of its 106 public symbols renamed to `ref_<name>`, so this
//! process holds *both* the C reference and cnt-rust's `#[no_mangle]` shims
//! (`strlen`, `_itoa`, …) and can fuzz them against each other.
//!
//! Naming: a C name that already starts with `_` gets the prefix glued on,
//! so `_stricmp` → [`ref__stricmp`].
//!
//! # Semantic notes on the reference (read before trusting a diff)
//!
//! * **Locale = the default "C" locale.** `get_locinfo()` (refimpl.c) hands
//!   back a locinfo with every `lc_handle[]` zeroed, exactly as msvcrt does
//!   before `setlocale()`. string.c *branches* on that and takes its
//!   plain-ASCII paths, so those paths are the genuine ones; `CompareStringA`
//!   / `LCMapStringA` are unreachable. A test that plants a non-zero
//!   `lc_handle` leaves the modelled world.
//! * **`long` is 32 bits** (`__msvcrt_long == int`), as on every Windows
//!   target. `LONG_MAX`/`LONG_MIN`/`ULONG_MAX` are clamped to match, so the
//!   `_atol_l` / `_strtol_l` / `_strtoul_l` saturation and the `_ltoa`
//!   negative-radix reinterpret behave as they do on the real thing.
//! * **`size_t` is 64 bits** — inherent to a host harness; the i386 msvcrt's
//!   is 32. Only reachable by fuzzing a length above `u32::MAX`.
//! * **`ref_strncmp` returns the raw byte difference**, not `-1/0/+1`.
//!   string.c has a `#if defined(_WIN64) || …` fork here and we compile the
//!   32-bit arm (the arm the symbol inventory was taken from). The 64-bit
//!   msvcrt returns the sign. Flip by defining `_WIN64` in build.rs.
//! * **`ref_memmove` does not exist**: msvcrt's memmove lives outside
//!   string.c (it is an *undefined* symbol in the real object), so
//!   `ref_memcpy` tail-calls the host `memmove`. Semantically identical.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

pub type SizeT = usize;

extern "C" {
    // ── search / length ───────────────────────────────────────────────
    pub fn ref_strlen(s: *const c_char) -> SizeT;
    pub fn ref_strnlen(s: *const c_char, maxlen: SizeT) -> SizeT;
    pub fn ref_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn ref_strncmp(s1: *const c_char, s2: *const c_char, len: SizeT) -> c_int;
    pub fn ref_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn ref_strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn ref_strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    pub fn ref_strspn(s: *const c_char, accept: *const c_char) -> SizeT;
    pub fn ref_strcspn(s: *const c_char, reject: *const c_char) -> SizeT;
    pub fn ref_strpbrk(s: *const c_char, accept: *const c_char) -> *mut c_char;

    // ── copy / concat ─────────────────────────────────────────────────
    pub fn ref_strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn ref_strncpy(dst: *mut c_char, src: *const c_char, len: SizeT) -> *mut c_char;
    pub fn ref_strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn ref_strncat(dst: *mut c_char, src: *const c_char, len: SizeT) -> *mut c_char;
    pub fn ref__strdup(s: *const c_char) -> *mut c_char;

    // ── secure-CRT variants (errno_t) ─────────────────────────────────
    pub fn ref_strcpy_s(dst: *mut c_char, size: SizeT, src: *const c_char) -> c_int;
    pub fn ref_strncpy_s(
        dst: *mut c_char,
        size: SizeT,
        src: *const c_char,
        count: SizeT,
    ) -> c_int;
    pub fn ref_strcat_s(dst: *mut c_char, size: SizeT, src: *const c_char) -> c_int;
    pub fn ref_strncat_s(
        dst: *mut c_char,
        size: SizeT,
        src: *const c_char,
        count: SizeT,
    ) -> c_int;

    // ── case / in-place mutation ──────────────────────────────────────
    pub fn ref__stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn ref__strnicmp(s1: *const c_char, s2: *const c_char, count: SizeT) -> c_int;
    pub fn ref__strlwr(s: *mut c_char) -> *mut c_char;
    pub fn ref__strupr(s: *mut c_char) -> *mut c_char;
    pub fn ref__strlwr_s(s: *mut c_char, len: SizeT) -> c_int;
    pub fn ref__strupr_s(s: *mut c_char, len: SizeT) -> c_int;
    pub fn ref__strrev(s: *mut c_char) -> *mut c_char;
    pub fn ref__strset(s: *mut c_char, c: c_int) -> *mut c_char;
    pub fn ref__strnset(s: *mut c_char, c: c_int, len: SizeT) -> *mut c_char;
    pub fn ref__swab(src: *mut c_char, dst: *mut c_char, len: c_int);

    // ── collation (C locale → plain ASCII paths) ──────────────────────
    pub fn ref_strcoll(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn ref__stricoll(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn ref__strncoll(s1: *const c_char, s2: *const c_char, count: SizeT) -> c_int;
    pub fn ref__strnicoll(s1: *const c_char, s2: *const c_char, count: SizeT) -> c_int;
    pub fn ref_strxfrm(dst: *mut c_char, src: *const c_char, len: SizeT) -> SizeT;

    // ── memory ────────────────────────────────────────────────────────
    pub fn ref_memchr(buf: *const c_void, c: c_int, n: SizeT) -> *mut c_void;
    pub fn ref_memcmp(p1: *const c_void, p2: *const c_void, n: SizeT) -> c_int;
    pub fn ref_memcpy(dst: *mut c_void, src: *const c_void, n: SizeT) -> *mut c_void;
    pub fn ref_memset(dst: *mut c_void, c: c_int, n: SizeT) -> *mut c_void;
    pub fn ref__memicmp(p1: *const c_void, p2: *const c_void, n: SizeT) -> c_int;
    pub fn ref__memccpy(
        dst: *mut c_void,
        src: *const c_void,
        c: c_int,
        n: SizeT,
    ) -> *mut c_void;

    // ── tokenise (per-thread cursor, see refimpl.c) ───────────────────
    pub fn ref_strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
    pub fn ref_strtok_s(
        s: *mut c_char,
        delim: *const c_char,
        ctx: *mut *mut c_char,
    ) -> *mut c_char;

    // ── integer parse ─────────────────────────────────────────────────
    // NOTE: `long` here is the *Windows* 32-bit long (__msvcrt_long == int),
    // NOT the host's `c_long`. See the module docs.
    pub fn ref_atoi(s: *const c_char) -> c_int;
    pub fn ref_atol(s: *const c_char) -> i32;
    pub fn ref_strtol(s: *const c_char, end: *mut *mut c_char, base: c_int) -> i32;
    pub fn ref_strtoul(s: *const c_char, end: *mut *mut c_char, base: c_int) -> u32;
    pub fn ref__atoi64(s: *const c_char) -> i64;
    pub fn ref__strtoi64(s: *const c_char, end: *mut *mut c_char, base: c_int) -> i64;
    pub fn ref__strtoui64(s: *const c_char, end: *mut *mut c_char, base: c_int) -> u64;

    // ── integer format ────────────────────────────────────────────────
    pub fn ref__itoa(value: c_int, s: *mut c_char, radix: c_int) -> *mut c_char;
    pub fn ref__ltoa(value: i32, s: *mut c_char, radix: c_int) -> *mut c_char;
    pub fn ref__ultoa(value: u32, s: *mut c_char, radix: c_int) -> *mut c_char;
    pub fn ref__i64toa(value: i64, s: *mut c_char, radix: c_int) -> *mut c_char;
    pub fn ref__ui64toa(value: u64, s: *mut c_char, radix: c_int) -> *mut c_char;
    pub fn ref__itoa_s(value: c_int, s: *mut c_char, size: SizeT, radix: c_int) -> c_int;
    pub fn ref__ltoa_s(value: i32, s: *mut c_char, size: SizeT, radix: c_int) -> c_int;
    pub fn ref__ultoa_s(value: u32, s: *mut c_char, size: SizeT, radix: c_int) -> c_int;
    pub fn ref__i64toa_s(value: i64, s: *mut c_char, size: SizeT, radix: c_int) -> c_int;
    pub fn ref__ui64toa_s(value: u64, s: *mut c_char, size: SizeT, radix: c_int) -> c_int;

    // ── wide format twins (Windows wchar_t == u16, forced in refstub) ──
    pub fn ref__itow(value: c_int, s: *mut u16, radix: c_int) -> *mut u16;
    pub fn ref__ltow(value: i32, s: *mut u16, radix: c_int) -> *mut u16;
    pub fn ref__ultow(value: u32, s: *mut u16, radix: c_int) -> *mut u16;
    pub fn ref__i64tow(value: i64, s: *mut u16, radix: c_int) -> *mut u16;
    pub fn ref__ui64tow(value: u64, s: *mut u16, radix: c_int) -> *mut u16;

    // ── float parse ───────────────────────────────────────────────────
    pub fn ref_atof(s: *const c_char) -> f64;
    pub fn ref_strtod(s: *const c_char, end: *mut *mut c_char) -> f64;

    // ── the harness's own errno window (refimpl.c) ────────────────────
    /// msvcrt's `errno` as the reference sees it (EINVAL / ERANGE / EILSEQ
    /// are part of several contracts). Distinct from the host's `errno` and
    /// from whatever cnt-rust reports.
    pub fn ref_errno_get() -> c_int;
    pub fn ref_errno_set(value: c_int);
}

/// Reference-side errno codes, as msvcrt spells them.
pub mod errno {
    pub const EINVAL: i32 = 22;
    pub const ERANGE: i32 = 34;
    pub const EILSEQ: i32 = 42;
    /// `strncpy_s`/`strcat_s` truncation result (`STRUNCATE`).
    pub const STRUNCATE: i32 = 80;
}

/// Keep the unused host-typed imports honest: these are the widths the
/// reference does *not* use, kept here so a future signature that really
/// does take a host `long` is spelled correctly.
#[doc(hidden)]
pub type HostLong = c_long;
#[doc(hidden)]
pub type HostULong = c_ulong;
#[doc(hidden)]
pub type HostUInt = c_uint;
