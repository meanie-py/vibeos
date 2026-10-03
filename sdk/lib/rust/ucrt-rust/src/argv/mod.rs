//! The last Microsoft C in ucrtbase: command-line parsing, wildcard expansion,
//! the WinMain command-line tail, the runtime-error reporter and app type, the
//! Windows app-model policy getters, the two PEB probes and the CRT message box.
//! Replaces, in full:
//!
//! | C file                                  | module                   |
//! |-----------------------------------------|--------------------------|
//! | `startup/argv_parsing.cpp`              | [`parsing`]              |
//! | `startup/argv_wildcards.cpp`            | [`wildcards`]            |
//! | `startup/argv_winmain.cpp`              | [`winmain`]              |
//! | `internal/report_runtime_error.cpp`     | [`report_runtime_error`] |
//! | `internal/win_policies.cpp`             | [`win_policies`]         |
//! | `internal/peb_access.cpp`               | [`peb_access`]           |
//! | `misc/crtmbox.cpp`                      | [`crtmbox`]              |
//!
//! The argument globals (`__argc`, `__argv`, `__wargv`, `_pgmptr`, `_wpgmptr`,
//! `_acmdln`, `_wcmdln`) and `__acrt_initialize_command_line` are
//! `crate::argv_data`'s; this module reads and writes them there.
//!
//! Every internal the C defined `extern "C"` keeps its exact name here, because
//! other ucrt objects still call it: `__acrt_report_runtime_error` (abort),
//! `__acrt_get_process_end_policy` / `__acrt_app_verifier_enabled` (exit),
//! `__acrt_get_begin_thread_init_policy` (thread), `__acrt_get_windowing_model_policy`
//! (winapi_thunks), `__acrt_show_wide_message_box` (dbgrpt, assert through the
//! traits), `_query_app_type` (assert, lowio).
//!
//! ## Details that are silent if wrong
//!
//! * **The parse is NOT `CommandLineToArgvW`'s.** The program name ends at the
//!   first unquoted space/tab and its quotes are simply dropped (no backslash
//!   rules at all); after it, `2N` backslashes + `"` are N backslashes and a
//!   quote toggle, `2N+1` + `"` are N backslashes and a literal quote, and a
//!   `""` INSIDE quotes is one literal `"` that does NOT end the quoted run.
//! * **argv is ONE allocation**: the pointer array (argc+1 slots, the last NULL)
//!   followed immediately by every argument's characters, `calloc`ed once
//!   (`__acrt_allocate_buffer_for_argv`). Callers `free(__argv)` once.
//! * **An empty or NULL `_acmdln` parses the module file name instead**, and
//!   `_pgmptr` always points at a static `MAX_PATH + 1` buffer filled by
//!   `GetModuleFileName` -- not into argv.
//! * **Wildcard "sorting" compares the ADDRESSES of the array slots**, not the
//!   strings (Microsoft's lambda casts nothing). So the matches come out in
//!   whatever order `qsort`'s partitioning leaves slots whose keys are their own
//!   positions: enumeration order for up to 8 matches, a permutation beyond.
//!   The crate's own `qsort` (a statement-for-statement port) is called, so the
//!   permutation is the same one.
//! * **A pattern with no match is kept verbatim**, as is one whose directory
//!   part contains a colon anywhere but index 1 (`a:b*`); `.` and `..` are never
//!   returned; the directory prefix of the PATTERN (up to its last `/`, `\` or
//!   `:` before the first wildcard) is glued onto each match.
//! * **Expansion failing to allocate the repacked block returns -1**, not ENOMEM.
//! * **`_get_*_winmain_command_line` skips the program name by `> ' '`**, with
//!   quotes toggling, then every character `<= ' '` -- UNSIGNED, so 0x80..0xFF
//!   are not whitespace in the narrow form.
//! * **`__acrt_report_runtime_error`'s program name keeps the FRONT**: a path
//!   longer than 59 characters gets `...` written over it at `len + 1 - 60`,
//!   which cuts the tail off, not the head.
//! * **`__acrt_get_process_end_policy` is never cached**; the other three
//!   getters cache their first answer in a static (0 = not yet known).
//!
//! Divergences from the C, each also commented at its site:
//!
//! * A DBCS lead byte immediately before the terminating NUL does NOT swallow
//!   the NUL as its trail byte (the C reads past the end of the command line).
//! * A found file name that cannot be converted back to the narrow codepage
//!   ends the expansion with the conversion's errno (the C dereferences NULL).

pub mod crtmbox;
pub mod parsing;
pub mod peb_access;
pub mod report_runtime_error;
pub mod win_policies;
pub mod wildcards;
pub mod winmain;

use core::ffi::{c_char, c_int, c_void};

pub(crate) const EINVAL: c_int = 22;
pub(crate) const ENOMEM: c_int = 12;
pub(crate) const MAX_PATH: usize = 260;

extern "C" {
    pub(crate) fn _errno() -> *mut c_int;
    pub(crate) fn _invalid_parameter_noinfo();
    pub(crate) fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    pub(crate) fn calloc(count: usize, size: usize) -> *mut c_void;
    pub(crate) fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void;
    pub(crate) fn free(p: *mut c_void);
    pub(crate) fn strlen(s: *const c_char) -> usize;
    pub(crate) fn wcslen(s: *const u16) -> usize;
    pub(crate) fn strncpy_s(dst: *mut c_char, dst_size: usize, src: *const c_char, count: usize) -> c_int;
    pub(crate) fn wcsncpy_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int;
    pub(crate) fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    pub(crate) fn wcscat_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    /// `_ismbblead(unsigned int)` -- truncates its argument to a byte itself.
    pub(crate) fn _ismbblead(c: u32) -> c_int;
}

/// `_ERRCHECK(e)`: Watson on any nonzero result.
#[inline]
pub(crate) unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// Wide string literal, NUL-terminated, from an ASCII `&str`.
macro_rules! w {
    ($s:literal) => {{
        const S: &str = $s;
        const N: usize = S.len() + 1;
        const W: [u16; N] = {
            let b = S.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() {
                w[i] = b[i] as u16;
                i += 1;
            }
            w
        };
        &W
    }};
}
pub(crate) use w;

/// The two character types every template here is instantiated for.
pub(crate) trait ArgChar: Copy + Eq + 'static {
    const NUL: Self;
    fn ascii(b: u8) -> Self;
    /// Unsigned value, as `unsigned char`/`wchar_t` compare.
    fn value(self) -> u32;
    #[inline(always)]
    fn is(self, b: u8) -> bool {
        self == Self::ascii(b)
    }
    #[inline(always)]
    fn is_nul(self) -> bool {
        self == Self::NUL
    }
    /// `traits::tcslen`
    unsafe fn tcslen(s: *const Self) -> usize;
    /// `traits::tcsncpy_s`
    unsafe fn tcsncpy_s(dst: *mut Self, size: usize, src: *const Self, count: usize) -> c_int;
}

impl ArgChar for u8 {
    const NUL: u8 = 0;
    #[inline(always)]
    fn ascii(b: u8) -> u8 {
        b
    }
    #[inline(always)]
    fn value(self) -> u32 {
        self as u32
    }
    unsafe fn tcslen(s: *const u8) -> usize {
        strlen(s.cast())
    }
    unsafe fn tcsncpy_s(dst: *mut u8, size: usize, src: *const u8, count: usize) -> c_int {
        strncpy_s(dst.cast(), size, src.cast(), count)
    }
}

impl ArgChar for u16 {
    const NUL: u16 = 0;
    #[inline(always)]
    fn ascii(b: u8) -> u16 {
        b as u16
    }
    #[inline(always)]
    fn value(self) -> u32 {
        self as u32
    }
    unsafe fn tcslen(s: *const u16) -> usize {
        wcslen(s)
    }
    unsafe fn tcsncpy_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int {
        wcsncpy_s(dst, size, src, count)
    }
}
