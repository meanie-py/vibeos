//! `lowio/mktemp.cpp`: `_mktemp`, `_wmktemp`, `_mktemp_s`, `_wmktemp_s`.
//!
//! Details that are silent if wrong:
//! * The LAST five `X`s become the current thread id's low five decimal
//!   digits (least significant last), and the `X` before them becomes a
//!   letter tried from `a` to `z`. A template needs six trailing `X`s; fewer
//!   is EINVAL with the handler.
//! * A name counts as "used" only when `_access_s(name, 0)` returns 0; any
//!   failure (ENOENT, but also EACCES...) means "free".
//! * All 26 letters used: the template is reset (`*template = 0`), errno is
//!   EEXIST, no handler, and EEXIST is returned. On success errno is restored
//!   to what it was on entry (the `_access_s` probes' errno is discarded).
//! * The narrow scan stops at a DBCS trail byte (`_ismbstrail`), so an `X`
//!   that is the second byte of a double-byte character is not replaced.
//! * An unterminated buffer (`strnlen == size`) resets the string and is
//!   EINVAL with the handler; so is a too-short template.
//! * `_mktemp` sizes the buffer as `strlen + 1`, so it can never hit the
//!   unterminated case.

use super::{invalid, set_errno, EINVAL, _errno};
use super::common::CRT_EEXIST;
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn strnlen(s: *const c_char, n: usize) -> usize;
    fn wcsnlen(s: *const u16, n: usize) -> usize;
    fn _ismbstrail(string: *const u8, current: *const u8) -> c_int;
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
}

extern "system" {
    fn GetCurrentThreadId() -> u32;
}

/// `__crt_char_traits<Character>` plus the two `common_mktemp_s_continue` overloads.
trait MkChar: Copy + PartialEq {
    fn from_ascii(b: u8) -> Self;
    unsafe fn tcslen(s: *const Self) -> usize;
    unsafe fn tcsnlen(s: *const Self, n: usize) -> usize;
    unsafe fn taccess_s(s: *const Self, mode: c_int) -> c_int;
    unsafe fn scan_continue(string: *const Self, current: *const Self) -> bool;
}

impl MkChar for u8 {
    fn from_ascii(b: u8) -> Self {
        b
    }
    unsafe fn tcslen(s: *const Self) -> usize {
        strlen(s.cast())
    }
    unsafe fn tcsnlen(s: *const Self, n: usize) -> usize {
        strnlen(s.cast(), n)
    }
    unsafe fn taccess_s(s: *const Self, mode: c_int) -> c_int {
        _access_s(s, mode)
    }
    unsafe fn scan_continue(string: *const Self, current: *const Self) -> bool {
        _ismbstrail(string, current) == 0
    }
}

impl MkChar for u16 {
    fn from_ascii(b: u8) -> Self {
        b as u16
    }
    unsafe fn tcslen(s: *const Self) -> usize {
        wcslen(s)
    }
    unsafe fn tcsnlen(s: *const Self, n: usize) -> usize {
        wcsnlen(s, n)
    }
    unsafe fn taccess_s(s: *const Self, mode: c_int) -> c_int {
        _waccess_s(s, mode)
    }
    unsafe fn scan_continue(_: *const Self, _: *const Self) -> bool {
        true
    }
}

unsafe fn common_mktemp_s<C: MkChar>(template_string: *mut C, buffer_size_in_chars: usize) -> c_int {
    if template_string.is_null() || buffer_size_in_chars == 0 {
        invalid(EINVAL);
        return EINVAL;
    }

    let template_string_length = C::tcsnlen(template_string, buffer_size_in_chars);
    if template_string_length >= buffer_size_in_chars {
        // _RESET_STRING, then _RETURN_DEST_NOT_NULL_TERMINATED.
        *template_string = C::from_ascii(0);
        invalid(EINVAL);
        return EINVAL;
    }

    // (_FILL_STRING is a no-op outside _DEBUG.)

    if template_string_length < 6 || buffer_size_in_chars <= template_string_length {
        *template_string = C::from_ascii(0);
        invalid(EINVAL);
        return EINVAL;
    }

    let mut number: u32 = GetCurrentThreadId();

    // `string` starts at the terminator.
    let mut string = template_string.wrapping_add(template_string_length);
    let mut template_length: usize = 0;

    // Replace the last five 'X' characters with the number.
    loop {
        string = string.wrapping_sub(1);
        if !(string >= template_string
            && C::scan_continue(template_string, string)
            && *string == C::from_ascii(b'X')
            && template_length < 5)
        {
            break;
        }
        template_length += 1;
        *string = C::from_ascii(((number % 10) as u8).wrapping_add(b'0'));
        number /= 10;
    }

    // Too few X's?
    if *string != C::from_ascii(b'X') || template_length < 5 {
        *template_string = C::from_ascii(0);
        invalid(EINVAL);
        return EINVAL;
    }

    // Finally, add the letter.
    let mut letter: u8 = b'a';
    *string = C::from_ascii(letter);
    letter += 1;

    let saved_errno = *_errno();
    set_errno(0);

    // Test each letter, from a-z, until we find a name that is not yet used.
    while C::taccess_s(template_string, 0) == 0 {
        if letter == b'z' + 1 {
            *template_string = C::from_ascii(0);
            set_errno(CRT_EEXIST);
            return *_errno();
        }
        *string = C::from_ascii(letter);
        letter += 1;
        set_errno(0);
    }

    // Restore the old value of errno and return success.
    set_errno(saved_errno);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mktemp_s(template_string: *mut c_char, buffer_size_in_chars: usize) -> c_int {
    common_mktemp_s(template_string.cast::<u8>(), buffer_size_in_chars)
}

#[no_mangle]
pub unsafe extern "C" fn _wmktemp_s(template_string: *mut u16, buffer_size_in_chars: usize) -> c_int {
    common_mktemp_s(template_string, buffer_size_in_chars)
}

unsafe fn common_mktemp<C: MkChar>(template_string: *mut C) -> *mut C {
    if template_string.is_null() {
        invalid(EINVAL);
        return null_mut();
    }
    let result = common_mktemp_s(template_string, C::tcslen(template_string) + 1);
    if result == 0 {
        template_string
    } else {
        null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mktemp(template_string: *mut c_char) -> *mut c_char {
    common_mktemp(template_string.cast::<u8>()).cast()
}

#[no_mangle]
pub unsafe extern "C" fn _wmktemp(template_string: *mut u16) -> *mut u16 {
    common_mktemp(template_string)
}
