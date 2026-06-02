//! Pure-decision leaf kernels of `sdk/lib/rtl/path.c`.
//!
//! These are the pure-byte-walk pieces that fall out of the `path.c`
//! translation. They live here (no `unsafe`, no FFI) so they are
//! `cargo test`-able directly. The cabi shims that wrap them with
//! `UNICODE_STRING` reconstruction are TBD (Wave PATH-1; stashed
//! pending the x86 boot fix).
//!
//! Faithful to ReactOS `sdk/lib/rtl/path.c` (Wine + Alex Ionescu +
//! Thomas Weidenmueller + Gunnar Dalsnes + Pierre Schweitzer).

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// --- NTSTATUS values used here.
pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_INVALID_PARAMETER: i32 = 0xC000_000Du32 as i32;

// --- `RTL_PATH_TYPE` (rtltypes.h:466).
pub const RTL_PATH_TYPE_UNKNOWN: u32 = 0;
pub const RTL_PATH_TYPE_UNC_ABSOLUTE: u32 = 1;
pub const RTL_PATH_TYPE_DRIVE_ABSOLUTE: u32 = 2;
pub const RTL_PATH_TYPE_DRIVE_RELATIVE: u32 = 3;
pub const RTL_PATH_TYPE_ROOTED: u32 = 4;
pub const RTL_PATH_TYPE_RELATIVE: u32 = 5;
pub const RTL_PATH_TYPE_LOCAL_DEVICE: u32 = 6;
pub const RTL_PATH_TYPE_ROOT_LOCAL_DEVICE: u32 = 7;

/// `MAX_PATH` from windef.h.
pub const MAX_PATH: u32 = 260;

/// `RtlGetLongestNtPathLength` constant: `MAX_PATH +
/// strlen(L"\\??\\UNC\\") + 1` = `260 + 8 + 1 = 269`.
pub const LONGEST_NT_PATH_LENGTH: u32 = MAX_PATH + 8 + 1;

/// `IS_PATH_SEPARATOR(x)` from path.c.
#[inline]
pub fn is_path_separator(c: u16) -> bool {
    c == b'\\' as u16 || c == b'/' as u16
}

/// Pure `RtlDetermineDosPathNameType_Ustr` — takes the wide-char
/// slice (length-bounded), returns the `RTL_PATH_TYPE` enum value.
/// Faithful to lines 60-91 of `sdk/lib/rtl/path.c`.
pub fn determine_dos_path_name_type_ustr(buf: &[u16]) -> u32 {
    let chars = buf.len();
    if chars == 0 {
        return RTL_PATH_TYPE_RELATIVE;
    }
    if is_path_separator(buf[0]) {
        if chars < 2 || !is_path_separator(buf[1]) {
            return RTL_PATH_TYPE_ROOTED;
        }
        if chars < 3 || (buf[2] != b'.' as u16 && buf[2] != b'?' as u16) {
            return RTL_PATH_TYPE_UNC_ABSOLUTE;
        }
        if chars >= 4 && is_path_separator(buf[3]) {
            return RTL_PATH_TYPE_LOCAL_DEVICE;
        }
        if chars != 3 {
            return RTL_PATH_TYPE_UNC_ABSOLUTE;
        }
        RTL_PATH_TYPE_ROOT_LOCAL_DEVICE
    } else {
        if chars < 2 || buf[1] != b':' as u16 {
            return RTL_PATH_TYPE_RELATIVE;
        }
        if chars < 3 || !is_path_separator(buf[2]) {
            return RTL_PATH_TYPE_DRIVE_RELATIVE;
        }
        RTL_PATH_TYPE_DRIVE_ABSOLUTE
    }
}

/// C-string flavor of the type decision — `RtlDetermineDosPathNameType_U`
/// (lines 1613-1632 of path.c) peeks `Path[0..=3]` unconditionally.
/// The cabi shim reads those 4 wide chars; this is the pure decision.
pub fn determine_dos_path_name_type_u_prefix(p0: u16, p1: u16, p2: u16, p3: u16) -> u32 {
    if is_path_separator(p0) {
        if !is_path_separator(p1) {
            return RTL_PATH_TYPE_ROOTED;
        }
        if p2 != b'.' as u16 && p2 != b'?' as u16 {
            return RTL_PATH_TYPE_UNC_ABSOLUTE;
        }
        if is_path_separator(p3) {
            return RTL_PATH_TYPE_LOCAL_DEVICE;
        }
        if p3 != 0 {
            return RTL_PATH_TYPE_UNC_ABSOLUTE;
        }
        RTL_PATH_TYPE_ROOT_LOCAL_DEVICE
    } else {
        if p0 == 0 || p1 != b':' as u16 {
            return RTL_PATH_TYPE_RELATIVE;
        }
        if is_path_separator(p2) {
            return RTL_PATH_TYPE_DRIVE_ABSOLUTE;
        }
        RTL_PATH_TYPE_DRIVE_RELATIVE
    }
}

/// `RtlGetLengthWithoutTrailingPathSeparators` body. Lines 1573-1610.
/// Counts the chars from the end that are NOT path separators.
pub fn length_without_trailing_separators(buf: &[u16]) -> u32 {
    let mut n = buf.len();
    while n > 0 && is_path_separator(buf[n - 1]) {
        n -= 1;
    }
    n as u32
}

/// ASCII downcase. The C code uses `RtlpDowncaseUnicodeChar` (a full
/// Unicode case-fold), but in the DOS-device path context the only
/// chars compared are ASCII letters `LPT`/`COM`/`PRN`/`AUX`/`CON`/`NUL`
/// (the path.c `RtlIsDosDeviceName_Ustr` comparisons go through
/// `RtlEqualUnicodeString(..., TRUE)` which is also ASCII-folded
/// in practice).
#[inline]
pub fn ascii_lower(c: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&c) {
        c + (b'a' as u16 - b'A' as u16)
    } else {
        c
    }
}

/// ASCII digit `0..=9` — what `iswdigit` resolves to for the
/// `LPT`/`COM` number-suffix peek in `RtlIsDosDeviceName_Ustr`.
#[inline]
pub fn is_digit(c: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    // tiny helper: wide a literal.
    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn empty_is_relative() {
        assert_eq!(determine_dos_path_name_type_ustr(&[]), RTL_PATH_TYPE_RELATIVE);
    }

    #[test]
    fn single_letter_is_relative() {
        assert_eq!(determine_dos_path_name_type_ustr(&w("x")), RTL_PATH_TYPE_RELATIVE);
    }

    #[test]
    fn drive_letter_only_is_drive_relative() {
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("C:")),
            RTL_PATH_TYPE_DRIVE_RELATIVE
        );
    }

    #[test]
    fn drive_with_path_is_absolute() {
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("C:\\")),
            RTL_PATH_TYPE_DRIVE_ABSOLUTE
        );
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("C:/")),
            RTL_PATH_TYPE_DRIVE_ABSOLUTE
        );
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("C:\\foo")),
            RTL_PATH_TYPE_DRIVE_ABSOLUTE
        );
    }

    #[test]
    fn single_separator_is_rooted() {
        assert_eq!(determine_dos_path_name_type_ustr(&w("\\")), RTL_PATH_TYPE_ROOTED);
        assert_eq!(determine_dos_path_name_type_ustr(&w("\\foo")), RTL_PATH_TYPE_ROOTED);
        assert_eq!(determine_dos_path_name_type_ustr(&w("/")), RTL_PATH_TYPE_ROOTED);
    }

    #[test]
    fn double_separator_unc() {
        assert_eq!(determine_dos_path_name_type_ustr(&w("\\\\x")), RTL_PATH_TYPE_UNC_ABSOLUTE);
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("\\\\server\\share")),
            RTL_PATH_TYPE_UNC_ABSOLUTE
        );
    }

    #[test]
    fn local_device_paths() {
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("\\\\.\\x")),
            RTL_PATH_TYPE_LOCAL_DEVICE
        );
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("\\\\?\\C:\\foo")),
            RTL_PATH_TYPE_LOCAL_DEVICE
        );
    }

    #[test]
    fn root_local_device_paths() {
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("\\\\.")),
            RTL_PATH_TYPE_ROOT_LOCAL_DEVICE
        );
        assert_eq!(
            determine_dos_path_name_type_ustr(&w("\\\\?")),
            RTL_PATH_TYPE_ROOT_LOCAL_DEVICE
        );
    }

    // \\.x and \\?x (no trailing slash on the device-marker — the
    // wikipedia-corner cases) classify as UNC, not local device.
    #[test]
    fn two_slash_dot_followed_by_non_slash_is_unc() {
        assert_eq!(determine_dos_path_name_type_ustr(&w("\\\\.x")), RTL_PATH_TYPE_UNC_ABSOLUTE);
        assert_eq!(determine_dos_path_name_type_ustr(&w("\\\\?x")), RTL_PATH_TYPE_UNC_ABSOLUTE);
    }

    // ------ C-string flavor parity with the slice flavor ------

    #[test]
    fn cstring_flavor_drive_absolute() {
        // "C:\0" — but the C-string flavor reads p0..=p3 unconditionally,
        // including past the NUL. Mirror that.
        assert_eq!(
            determine_dos_path_name_type_u_prefix(b'C' as u16, b':' as u16, b'\\' as u16, 0),
            RTL_PATH_TYPE_DRIVE_ABSOLUTE
        );
    }

    #[test]
    fn cstring_flavor_relative_on_empty() {
        // Single NUL at front.
        assert_eq!(
            determine_dos_path_name_type_u_prefix(0, 0, 0, 0),
            RTL_PATH_TYPE_RELATIVE
        );
    }

    // ------ length_without_trailing_separators ------

    #[test]
    fn length_no_separators_returns_full() {
        assert_eq!(length_without_trailing_separators(&w("foo")), 3);
    }

    #[test]
    fn length_one_trailing_drops_one() {
        assert_eq!(length_without_trailing_separators(&w("foo\\")), 3);
        assert_eq!(length_without_trailing_separators(&w("foo/")), 3);
    }

    #[test]
    fn length_many_trailing_drops_all() {
        assert_eq!(length_without_trailing_separators(&w("foo\\\\//")), 3);
    }

    #[test]
    fn length_only_separators_zero() {
        assert_eq!(length_without_trailing_separators(&w("\\\\\\")), 0);
        assert_eq!(length_without_trailing_separators(&w("///")), 0);
    }

    #[test]
    fn length_empty_zero() {
        assert_eq!(length_without_trailing_separators(&[]), 0);
    }

    // ------ ascii_lower / is_digit ------

    #[test]
    fn ascii_lower_caps_to_lower() {
        assert_eq!(ascii_lower(b'A' as u16), b'a' as u16);
        assert_eq!(ascii_lower(b'Z' as u16), b'z' as u16);
    }

    #[test]
    fn ascii_lower_lower_passthrough() {
        assert_eq!(ascii_lower(b'a' as u16), b'a' as u16);
        assert_eq!(ascii_lower(b'z' as u16), b'z' as u16);
    }

    #[test]
    fn ascii_lower_digit_passthrough() {
        assert_eq!(ascii_lower(b'5' as u16), b'5' as u16);
    }

    #[test]
    fn ascii_lower_unicode_passthrough() {
        // Outside ASCII — pass through untouched.
        assert_eq!(ascii_lower(0x00C0), 0x00C0); // À
    }

    #[test]
    fn is_digit_boundary() {
        assert!(is_digit(b'0' as u16));
        assert!(is_digit(b'9' as u16));
        assert!(!is_digit(b'/' as u16));
        assert!(!is_digit(b':' as u16));
        assert!(!is_digit(b'a' as u16));
    }

    // ------ longest_nt_path is the documented 269 ------

    #[test]
    fn longest_nt_path_is_269() {
        assert_eq!(LONGEST_NT_PATH_LENGTH, 269);
    }
}
