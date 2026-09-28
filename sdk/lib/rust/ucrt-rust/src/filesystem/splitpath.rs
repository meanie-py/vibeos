//! `_splitpath`/`_wsplitpath` and their `_s` forms. Replaces
//! `sdk/lib/ucrt/filesystem/splitpath.cpp` in full.
//!
//! One generic body (`common_splitpath_internal`) over the character width,
//! exactly as the C's own template; the difference between narrow and wide
//! is `needs_trail_byte` (MBCS lead-byte awareness) via the `SplitChar`
//! trait. The `_s` forms and the legacy fixed-size forms differ only in
//! which counts are checked (the caller's own vs. `_MAX_DRIVE`/`_MAX_DIR`/
//! `_MAX_FNAME`/`_MAX_EXT`) -- both bound-check against the REAL count before
//! ever writing, so this port skips `strncpy_s`/`wcsncpy_s` entirely and
//! copies+NUL-terminates directly once a fit is already proven.
//!
//! ## Details that are silent if wrong
//!
//! * **Two different "buffer too small" behaviours, kept distinct**: the
//!   whole-call NULL/shape checks below go through `_VALIDATE_RETURN_ERRCODE`
//!   (handler invoked, every requested component buffer reset to `""`
//!   first); a per-component ERANGE (one buffer too small for what it found)
//!   is `return errno = ERANGE;` DIRECTLY -- no handler call at all -- after
//!   the same full reset. Contrast `makepath.rs`, whose overflow always goes
//!   through the handler.
//! * **The shape check**: for each of drive/directory/file_name/extension,
//!   `(pointer == NULL) != (count == 0)` is itself EINVAL (through the
//!   handler) -- a non-NULL buffer with a zero count, or vice versa, is
//!   rejected before any parsing happens.
//! * **Drive detection looks at exactly one position**: it steps forward
//!   `_MAX_DRIVE - 2` (= 1) characters (or to the first NUL, if sooner), and
//!   the path has a drive iff THAT position holds `:`. This is "does
//!   `path[1] == ':'`", not a general scan.
//! * **The last `/` or `\` wins for the directory boundary, and the last `.`
//!   AT OR AFTER it wins for the extension boundary** -- both are found in
//!   one forward scan of the whole (post-drive) path.
//! * **The narrow form skips DBCS trail bytes while scanning** for those
//!   separators via `_ismbblead`, so a trail byte that happens to look like
//!   `\` or `.` is never misread as one. The wide form has no such concern.
//! * **A dot at or after the last separator, but with nothing following it
//!   that ISN'T itself the terminator, still counts** -- an extension can be
//!   just `.` (empty extension, dot still copied) if that's genuinely the
//!   last character.
//! * **Requesting a component the caller didn't ask for (a NULL pointer with
//!   0 count) is simply skipped**, both for writing AND for resetting --
//!   `reset_buffer`/the `_s` reset lambda are no-ops on a NULL pointer.

use core::ffi::c_int;

use super::helpers::{invalid_parameter, EINVAL, ERANGE, _errno};

const MAX_DRIVE: usize = 3;
const MAX_DIR: usize = 256;
const MAX_FNAME: usize = 256;
const MAX_EXT: usize = 256;

extern "C" {
    // Matches `sdk/lib/rust/ucrt-rust/src/ismbbyte.rs`'s `_ismbblead`
    // declaration exactly.
    fn _ismbblead(c: u32) -> c_int;
}

trait SplitChar: Copy + PartialEq {
    const ZERO: Self;
    const DOT: Self;
    const SLASH: Self;
    const BACKSLASH: Self;
    const COLON: Self;
    /// `needs_trail_byte`: true only for a narrow DBCS lead byte.
    unsafe fn needs_trail_byte(c: Self) -> bool;
}

impl SplitChar for u8 {
    const ZERO: u8 = 0;
    const DOT: u8 = b'.';
    const SLASH: u8 = b'/';
    const BACKSLASH: u8 = b'\\';
    const COLON: u8 = b':';
    unsafe fn needs_trail_byte(c: u8) -> bool {
        _ismbblead(c as u32) != 0
    }
}

impl SplitChar for u16 {
    const ZERO: u16 = 0;
    const DOT: u16 = b'.' as u16;
    const SLASH: u16 = b'/' as u16;
    const BACKSLASH: u16 = b'\\' as u16;
    const COLON: u16 = b':' as u16;
    unsafe fn needs_trail_byte(_c: u16) -> bool {
        false
    }
}

struct Components<C> {
    drive: *mut C,
    drive_count: usize,
    directory: *mut C,
    directory_count: usize,
    file_name: *mut C,
    file_name_count: usize,
    extension: *mut C,
    extension_count: usize,
}

/// `_RESET_STRING(buffer, count)` in a release build (the fill is debug-only)
/// gated the same way both the `_s` and legacy reset policies gate it: only
/// on the pointer being non-NULL (the shape check above already guarantees a
/// non-NULL pointer has count > 0).
unsafe fn reset_component<C: SplitChar>(buffer: *mut C, _count: usize) {
    if !buffer.is_null() {
        *buffer = C::ZERO;
    }
}

unsafe fn reset_all<C: SplitChar>(c: &Components<C>) {
    reset_component(c.drive, c.drive_count);
    reset_component(c.directory, c.directory_count);
    reset_component(c.file_name, c.file_name_count);
    reset_component(c.extension, c.extension_count);
}

/// Copy exactly `n` characters from `src`, then NUL-terminate at `dst[n]`.
/// Every call site has already proven `n < the destination's real count`.
unsafe fn copy_and_terminate<C: SplitChar>(dst: *mut C, src: *const C, n: usize) {
    core::ptr::copy_nonoverlapping(src, dst, n);
    *dst.add(n) = C::ZERO;
}

/// `common_splitpath_internal<Character>`.
unsafe fn common_splitpath_internal<C: SplitChar>(path: *const C, components: &Components<C>) -> c_int {
    if path.is_null() {
        reset_all(components);
        invalid_parameter(EINVAL);
        return EINVAL;
    }

    if (components.drive.is_null()) != (components.drive_count == 0)
        || (components.directory.is_null()) != (components.directory_count == 0)
        || (components.file_name.is_null()) != (components.file_name_count == 0)
        || (components.extension.is_null()) != (components.extension_count == 0)
    {
        reset_all(components);
        invalid_parameter(EINVAL);
        return EINVAL;
    }

    macro_rules! too_small_range {
        () => {{
            reset_all(components);
            *_errno() = ERANGE;
            return ERANGE;
        }};
    }

    let mut path_it = path;

    // Extract drive letter and ':', if any:
    {
        let mut skip: usize = MAX_DRIVE - 2;
        let mut p = path_it;
        while skip > 0 && *p != C::ZERO {
            skip -= 1;
            p = p.add(1);
        }

        if *p == C::COLON {
            if !components.drive.is_null() {
                if components.drive_count < MAX_DRIVE {
                    too_small_range!();
                }
                copy_and_terminate(components.drive, path_it, MAX_DRIVE - 1);
            }
            path_it = p.add(1);
        } else {
            reset_component(components.drive, components.drive_count);
        }
    }

    // Scan for the last '/' or '\' and the last '.' at or after it:
    let mut p = path_it;
    let mut last_slash: *const C = core::ptr::null();
    let mut last_dot: *const C = core::ptr::null();
    while *p != C::ZERO {
        if C::needs_trail_byte(*p) {
            p = p.add(1);
            if *p == C::ZERO {
                break;
            }
        } else if *p == C::SLASH || *p == C::BACKSLASH {
            last_slash = p.add(1);
        } else if *p == C::DOT {
            last_dot = p;
        }
        p = p.add(1);
    }

    if !last_slash.is_null() {
        if !components.directory.is_null() {
            let length = last_slash.offset_from(path_it) as usize;
            if components.directory_count <= length {
                too_small_range!();
            }
            copy_and_terminate(components.directory, path_it, length);
        }
        path_it = last_slash;
    } else {
        reset_component(components.directory, components.directory_count);
    }

    if !last_dot.is_null() && last_dot >= path_it {
        if !components.file_name.is_null() {
            let length = last_dot.offset_from(path_it) as usize;
            if components.file_name_count <= length {
                too_small_range!();
            }
            copy_and_terminate(components.file_name, path_it, length);
        }
        if !components.extension.is_null() {
            let length = p.offset_from(last_dot) as usize;
            if components.extension_count <= length {
                too_small_range!();
            }
            copy_and_terminate(components.extension, last_dot, length);
        }
    } else {
        if !components.file_name.is_null() {
            let length = p.offset_from(path_it) as usize;
            if components.file_name_count <= length {
                too_small_range!();
            }
            copy_and_terminate(components.file_name, path_it, length);
        }
        reset_component(components.extension, components.extension_count);
    }

    0
}

/// `_splitpath_s`.
#[no_mangle]
pub unsafe extern "C" fn _splitpath_s(
    path: *const u8,
    drive: *mut u8,
    drive_count: usize,
    directory: *mut u8,
    directory_count: usize,
    file_name: *mut u8,
    file_name_count: usize,
    extension: *mut u8,
    extension_count: usize,
) -> c_int {
    let components = Components {
        drive,
        drive_count,
        directory,
        directory_count,
        file_name,
        file_name_count,
        extension,
        extension_count,
    };
    common_splitpath_internal(path, &components)
}

/// `_wsplitpath_s`.
#[no_mangle]
pub unsafe extern "C" fn _wsplitpath_s(
    path: *const u16,
    drive: *mut u16,
    drive_count: usize,
    directory: *mut u16,
    directory_count: usize,
    file_name: *mut u16,
    file_name_count: usize,
    extension: *mut u16,
    extension_count: usize,
) -> c_int {
    let components = Components {
        drive,
        drive_count,
        directory,
        directory_count,
        file_name,
        file_name_count,
        extension,
        extension_count,
    };
    common_splitpath_internal(path, &components)
}

/// `common_splitpath<Character>`: the legacy, unchecked-size form, which
/// always requests every one of `_MAX_DRIVE`/`_MAX_DIR`/`_MAX_FNAME`/
/// `_MAX_EXT` for a non-NULL destination (0 for a NULL one).
unsafe fn common_splitpath<C: SplitChar>(
    path: *const C,
    drive: *mut C,
    directory: *mut C,
    file_name: *mut C,
    extension: *mut C,
) {
    let components = Components {
        drive,
        drive_count: if drive.is_null() { 0 } else { MAX_DRIVE },
        directory,
        directory_count: if directory.is_null() { 0 } else { MAX_DIR },
        file_name,
        file_name_count: if file_name.is_null() { 0 } else { MAX_FNAME },
        extension,
        extension_count: if extension.is_null() { 0 } else { MAX_EXT },
    };
    common_splitpath_internal(path, &components);
}

/// `_splitpath`.
#[no_mangle]
pub unsafe extern "C" fn _splitpath(
    path: *const u8,
    drive: *mut u8,
    directory: *mut u8,
    file_name: *mut u8,
    extension: *mut u8,
) {
    common_splitpath(path, drive, directory, file_name, extension);
}

/// `_wsplitpath`.
#[no_mangle]
pub unsafe extern "C" fn _wsplitpath(
    path: *const u16,
    drive: *mut u16,
    directory: *mut u16,
    file_name: *mut u16,
    extension: *mut u16,
) {
    common_splitpath(path, drive, directory, file_name, extension);
}
