//! `_makepath`/`_wmakepath` and their `_s` forms. Replaces
//! `sdk/lib/ucrt/filesystem/makepath.cpp` in full.
//!
//! One generic body (`common_makepath_s`) over the character width, exactly
//! as the C's `common_makepath_s<Character>` template; only "the character
//! just before here" (`PathChar::previous`) differs between narrow (MBCS
//! -aware, via `_mbsdec`) and wide (`current - 1`).
//!
//! ## Details that are silent if wrong
//!
//! * **The non-`_s` forms pass `_CRT_UNBOUNDED_BUFFER_SIZE` (`usize::MAX`) as
//!   the count**, which the body treats as "no bound": every too-small check,
//!   including the final NUL write's, is skipped. A caller with an undersized
//!   fixed buffer overflows exactly as the real UCRT does.
//! * **`_makepath_s`/`_wmakepath_s` validate `(buffer, count)` FIRST**
//!   (`_VALIDATE_STRING`): a NULL buffer or a zero count is EINVAL,
//!   unconditionally, before any of `drive`/`directory`/`file_name`/
//!   `extension` are even looked at.
//! * **On overflow the buffer is reset to `""`** (first char only -- the
//!   C's `_RESET_STRING` fill-the-rest behaviour is debug-build only) **and
//!   the return is ERANGE THROUGH THE INVALID-PARAMETER HANDLER** (the
//!   `_RETURN_BUFFER_TOO_SMALL` macro), not a raw, unhandled ERANGE --
//!   contrast `splitpath.rs`, whose overflow sets `errno` directly with no
//!   handler call.
//! * **The directory gets exactly one trailing separator added**, only if it
//!   doesn't already end in `/` or `\`. The narrow form decides "does it end
//!   in a separator" by walking back a WHOLE multibyte character with
//!   `_mbsdec`, so a DBCS trail byte that happens to look like `\` is never
//!   misread as one.
//! * **The extension gets a `.` prepended** unless it is empty or already
//!   starts with one.
//! * **A NULL `drive`/`directory` is treated the same as an empty one**
//!   (nothing copied); a NULL `file_name`/`extension` skips that whole
//!   section (including, for extension, the `.` insertion), while an EMPTY
//!   (but non-NULL) one still runs its loop and copies nothing -- observably
//!   identical, since the loop condition is "until NUL".

use core::ffi::c_int;

use super::helpers::{invalid_parameter, EINVAL, ERANGE};

/// `_CRT_UNBOUNDED_BUFFER_SIZE`.
const UNBOUNDED: usize = usize::MAX;

extern "C" {
    // Matches `sdk/lib/rust/ucrt-rust/src/mbstring/nav.rs`'s `_mbsdec`
    // declaration exactly.
    fn _mbsdec(string: *const u8, current: *const u8) -> *mut u8;
}

/// The one difference between the narrow and wide `common_makepath_s`
/// instantiations: how to step back one CHARACTER (not necessarily one code
/// unit) from `current`, and the separator/punctuation code points.
trait PathChar: Copy + PartialEq {
    const ZERO: Self;
    const COLON: Self;
    const DOT: Self;
    const BACKSLASH: Self;
    const SLASH: Self;
    /// `previous_character`.
    unsafe fn previous(first: *const Self, current: *const Self) -> *const Self;
}

impl PathChar for u8 {
    const ZERO: u8 = 0;
    const COLON: u8 = b':';
    const DOT: u8 = b'.';
    const BACKSLASH: u8 = b'\\';
    const SLASH: u8 = b'/';
    unsafe fn previous(first: *const u8, current: *const u8) -> *const u8 {
        _mbsdec(first, current) as *const u8
    }
}

impl PathChar for u16 {
    const ZERO: u16 = 0;
    const COLON: u16 = b':' as u16;
    const DOT: u16 = b'.' as u16;
    const BACKSLASH: u16 = b'\\' as u16;
    const SLASH: u16 = b'/' as u16;
    unsafe fn previous(_first: *const u16, current: *const u16) -> *const u16 {
        current.offset(-1)
    }
}

/// `common_makepath_s<Character>`.
unsafe fn common_makepath_s<C: PathChar>(
    result_buffer: *mut C,
    result_count: usize,
    drive: *const C,
    directory: *const C,
    file_name: *const C,
    extension: *const C,
) -> c_int {
    // _VALIDATE_STRING(result_buffer, result_count)
    if result_buffer.is_null() || result_count == 0 {
        invalid_parameter(EINVAL);
        return EINVAL;
    }

    let bound: Option<usize> = if result_count == UNBOUNDED { None } else { Some(result_count) };
    let mut idx: usize = 0;

    // `cleanup_after_error`: reset the string (release build: just the first
    // char) and return ERANGE through the invalid-parameter handler.
    macro_rules! too_small {
        () => {{
            *result_buffer = C::ZERO;
            invalid_parameter(ERANGE);
            return ERANGE;
        }};
    }
    macro_rules! room_for {
        ($n:expr) => {
            if let Some(b) = bound {
                if b - idx < $n {
                    too_small!();
                }
            }
        };
    }

    // Copy the drive:
    if !drive.is_null() && *drive != C::ZERO {
        room_for!(2);
        *result_buffer.add(idx) = *drive;
        idx += 1;
        *result_buffer.add(idx) = C::COLON;
        idx += 1;
    }

    // Copy the directory:
    if !directory.is_null() && *directory != C::ZERO {
        let mut source_it = directory;
        while *source_it != C::ZERO {
            room_for!(1);
            *result_buffer.add(idx) = *source_it;
            idx += 1;
            source_it = source_it.add(1);
        }

        // Write a trailing backslash if there isn't one:
        let prev = C::previous(directory, source_it);
        if *prev != C::SLASH && *prev != C::BACKSLASH {
            room_for!(1);
            *result_buffer.add(idx) = C::BACKSLASH;
            idx += 1;
        }
    }

    // Copy the file name:
    if !file_name.is_null() {
        let mut source_it = file_name;
        while *source_it != C::ZERO {
            room_for!(1);
            *result_buffer.add(idx) = *source_it;
            idx += 1;
            source_it = source_it.add(1);
        }
    }

    // Copy the extension:
    if !extension.is_null() {
        // Add a '.' if one is required:
        if *extension != C::ZERO && *extension != C::DOT {
            room_for!(1);
            *result_buffer.add(idx) = C::DOT;
            idx += 1;
        }

        let mut source_it = extension;
        while *source_it != C::ZERO {
            room_for!(1);
            *result_buffer.add(idx) = *source_it;
            idx += 1;
            source_it = source_it.add(1);
        }
    }

    // Copy the null terminator:
    room_for!(1);
    *result_buffer.add(idx) = C::ZERO;

    0
}

/// `_makepath_s`.
#[no_mangle]
pub unsafe extern "C" fn _makepath_s(
    result_buffer: *mut u8,
    result_count: usize,
    drive: *const u8,
    directory: *const u8,
    file_name: *const u8,
    extension: *const u8,
) -> c_int {
    common_makepath_s(result_buffer, result_count, drive, directory, file_name, extension)
}

/// `_wmakepath_s`.
#[no_mangle]
pub unsafe extern "C" fn _wmakepath_s(
    result_buffer: *mut u16,
    result_count: usize,
    drive: *const u16,
    directory: *const u16,
    file_name: *const u16,
    extension: *const u16,
) -> c_int {
    common_makepath_s(result_buffer, result_count, drive, directory, file_name, extension)
}

/// `_makepath`.
#[no_mangle]
pub unsafe extern "C" fn _makepath(
    result_buffer: *mut u8,
    drive: *const u8,
    directory: *const u8,
    file_name: *const u8,
    extension: *const u8,
) {
    _makepath_s(result_buffer, UNBOUNDED, drive, directory, file_name, extension);
}

/// `_wmakepath`.
#[no_mangle]
pub unsafe extern "C" fn _wmakepath(
    result_buffer: *mut u16,
    drive: *const u16,
    directory: *const u16,
    file_name: *const u16,
    extension: *const u16,
) {
    _wmakepath_s(result_buffer, UNBOUNDED, drive, directory, file_name, extension);
}
