//! `startup/argv_winmain.cpp`: `_get_narrow_winmain_command_line` and
//! `_get_wide_winmain_command_line` -- the command-line tail `[w]WinMain` gets.
//!
//! "This code was originally implemented by the NT group and has remained
//! pretty much unchanged since 12-91": skip the program name (every character
//! `> ' '`, or anything but NUL while inside quotes), then everything `<= ' '`.
//! Both comparisons are UNSIGNED (`unsigned char` in the narrow form), and the
//! result points INTO `_acmdln`/`_wcmdln`, or at a static empty string when
//! that is NULL.

use super::{ArgChar, _ismbblead};
use crate::argv_data::{_acmdln, _wcmdln};
use core::ptr::addr_of_mut;

trait WinmainChar: ArgChar {
    unsafe fn do_locale_initialization();
    unsafe fn get_command_line() -> *mut Self;
    unsafe fn empty_string() -> *mut Self;
    unsafe fn should_copy_another_character(self) -> bool;
}

static mut NARROW_EMPTY_STRING: [u8; 1] = [0];
static mut WIDE_EMPTY_STRING: [u16; 1] = [0];

impl WinmainChar for u8 {
    unsafe fn do_locale_initialization() {
        crate::locale_abi::__acrt_initialize_multibyte();
    }
    unsafe fn get_command_line() -> *mut u8 {
        _acmdln
    }
    unsafe fn empty_string() -> *mut u8 {
        addr_of_mut!(NARROW_EMPTY_STRING).cast()
    }
    unsafe fn should_copy_another_character(self) -> bool {
        // `unsigned char` here (zero-extended), unlike argv_parsing's `char`.
        _ismbblead(self as u32) != 0
    }
}

impl WinmainChar for u16 {
    unsafe fn do_locale_initialization() {}
    unsafe fn get_command_line() -> *mut u16 {
        _wcmdln
    }
    unsafe fn empty_string() -> *mut u16 {
        addr_of_mut!(WIDE_EMPTY_STRING).cast()
    }
    unsafe fn should_copy_another_character(self) -> bool {
        false
    }
}

/// `common_wincmdln<Character>`
unsafe fn common_wincmdln<C: WinmainChar>() -> *mut C {
    C::do_locale_initialization();

    let mut command_line = if C::get_command_line().is_null() {
        C::empty_string()
    } else {
        C::get_command_line()
    };

    // Skip past the program name (the first token in the command line) and
    // check for and handle a quoted program name:
    let mut in_double_quotes = false;
    while (*command_line).value() > b' ' as u32 || (!(*command_line).is_nul() && in_double_quotes) {
        // Toggle the in_double_quotes flag if the current character is '"'
        if (*command_line).is(b'"') {
            in_double_quotes = !in_double_quotes;
        }

        // DIVERGENCE: the C skips the byte after a lead byte unconditionally,
        // so a lead byte right before the NUL steps over the terminator and
        // keeps scanning past the end. A NUL is never a trail byte here.
        if (*command_line).should_copy_another_character() && !(*command_line.add(1)).is_nul() {
            command_line = command_line.add(1);
        }

        command_line = command_line.add(1);
    }

    // Skip past any whitespace preceding the next token:
    while !(*command_line).is_nul() && (*command_line).value() <= b' ' as u32 {
        command_line = command_line.add(1);
    }

    command_line
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _get_narrow_winmain_command_line() -> *mut u8 {
    // unsigned char, so that bytes above 127 compare correctly against ' '.
    common_wincmdln::<u8>()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _get_wide_winmain_command_line() -> *mut u16 {
    common_wincmdln::<u16>()
}
