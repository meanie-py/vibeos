use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static nameprep_char_type: [u16; 0];
}

// ---- file-scope preamble (translated) ----
pub const BASE: i32 = 36;
pub const TMIN: i32 = 1;
pub const TMAX: i32 = 26;
pub const SKEW: i32 = 38;
pub const DAMP: i32 = 700;
pub const INIT_BIAS: i32 = 72;
pub const INIT_N: i32 = 128;

unsafe extern "C" fn adapt(mut delta: INT, mut numpoints: INT, mut firsttime: BOOL) -> INT {
    let mut k: INT;

    delta /= if firsttime != 0 { DAMP as INT } else { 2 as INT };
    delta += delta / numpoints;

    k = 0 as INT;
    while delta > ((BASE as INT - TMIN as INT) * TMAX as INT) / 2 as INT {
        delta /= BASE as INT - TMIN as INT;
        k += BASE as INT;
    }
    return k + ((BASE as INT - TMIN as INT + 1 as INT) * delta) / (delta + SKEW as INT);
}

unsafe extern "C" fn get_table_entry(table: *const ::core::ffi::c_ushort, ch: WCHAR) -> ::core::ffi::c_ushort {
    return *table.offset(
        (*table.offset(
            (*table.offset(((ch as ::core::ffi::c_int) >> 8) as isize) as ::core::ffi::c_int
                + (((ch as ::core::ffi::c_int) >> 4) & 0x0f)) as isize,
        ) as ::core::ffi::c_int
            + ((ch as ::core::ffi::c_int) & 0xf)) as isize,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IdnToUnicode(
    mut dwFlags: DWORD,
    mut lpASCIICharStr: LPCWSTR,
    mut cchASCIIChar: INT,
    mut lpUnicodeCharStr: LPWSTR,
    mut cchUnicodeChar: INT,
) -> INT {

    let mut i: INT = 0;
    let mut label_start: INT = 0;
    let mut label_end: INT = 0;
    let mut out_label: INT = 0;
    let mut out: INT = 0;
    let mut ch: WCHAR = 0;

    DPRINT!(
        "%x %p %d %p %d\n",
        dwFlags,
        lpASCIICharStr,
        cchASCIIChar,
        lpUnicodeCharStr,
        cchUnicodeChar
    );

    label_start = 0;
    while label_start < cchASCIIChar {
        let mut n: INT = INIT_N;
        let mut pos: INT = 0;
        let mut old_pos: INT = 0;
        let mut w: INT = 0;
        let mut k: INT = 0;
        let mut bias: INT = INIT_BIAS;
        let mut delim: INT = 0;
        let mut digit: INT = 0;
        let mut t: INT = 0;

        out_label = out;
        i = label_start;
        while i < cchASCIIChar {
            ch = *lpASCIICharStr.offset(i as isize);

            if (ch as INT) > 0x7f
                || (i != cchASCIIChar - 1 && ch == 0)
            {
                SetLastError(ERROR_INVALID_NAME as DWORD);
                return 0;
            }

            if ch == 0 || ch == '.' as WCHAR {
                break;
            }
            if ch == '-' as WCHAR {
                delim = i;
            }

            if (dwFlags & IDN_USE_STD3_ASCII_RULES) != 0 {
                if !((ch >= 'a' as WCHAR && ch <= 'z' as WCHAR)
                    || (ch >= 'A' as WCHAR && ch <= 'Z' as WCHAR)
                    || (ch >= '0' as WCHAR && ch <= '9' as WCHAR)
                    || ch == '-' as WCHAR)
                {
                    SetLastError(ERROR_INVALID_NAME as DWORD);
                    return 0;
                }
            }

            i += 1;
        }
        label_end = i;

        if label_start == label_end && ch != 0 {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0;
        }

        // Divergence from the C, which reads `lpASCIICharStr[label_end-1]` for an empty
        // final label too: on an input that is just L"\0" that is index -1, before the
        // caller's buffer (an empty label after a '.' reads the '.', never '-'). An empty
        // label has no last character, so the test needs a non-empty one.
        if (dwFlags & IDN_USE_STD3_ASCII_RULES) != 0
            && (*lpASCIICharStr.offset(label_start as isize) == '-' as WCHAR
                || (label_end > label_start
                    && *lpASCIICharStr.offset((label_end - 1) as isize) == '-' as WCHAR))
        {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0;
        }

        if label_end - label_start > 63 {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0;
        }

        if label_end - label_start < 4
            || tolowerW(*lpASCIICharStr.offset(label_start as isize)) as WCHAR != 'x' as WCHAR
            || tolowerW(*lpASCIICharStr.offset((label_start + 1) as isize))
                as WCHAR
                != 'n' as WCHAR
            || *lpASCIICharStr.offset((label_start + 2) as isize)
                != '-' as WCHAR
            || *lpASCIICharStr.offset((label_start + 3) as isize)
                != '-' as WCHAR
        {
            if label_end < cchASCIIChar {
                label_end += 1;
            }

            if lpUnicodeCharStr.is_null() {
                out += label_end - label_start;
            } else if out + label_end - label_start <= cchUnicodeChar {
                memcpy(
                    lpUnicodeCharStr.offset(out as isize) as *mut core::ffi::c_void,
                    lpASCIICharStr.offset(label_start as isize) as *const core::ffi::c_void,
                    ((label_end - label_start) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                out += label_end - label_start;
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }

            label_start = label_end;
            continue;
        }

        if delim == label_start + 3 {
            delim += 1;
        }
        if lpUnicodeCharStr.is_null() {
            out += delim - label_start - 4;
        } else if out + delim - label_start - 4 <= cchUnicodeChar {
            memcpy(
                lpUnicodeCharStr.offset(out as isize) as *mut core::ffi::c_void,
                lpASCIICharStr.offset((label_start + 4) as isize)
                    as *const core::ffi::c_void,
                ((delim - label_start - 4) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            out += delim - label_start - 4;
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return 0;
        }
        if out != out_label {
            delim += 1;
        }

        i = delim;
        while i < label_end {
            old_pos = pos;
            w = 1;
            k = BASE;
            loop {
                ch = if i < label_end {
                    let fresh = i;
                    i += 1;
                    tolowerW(*lpASCIICharStr.offset(fresh as isize)) as WCHAR
                } else {
                    0 as WCHAR
                };

                if (ch < 'a' as WCHAR || ch > 'z' as WCHAR)
                    && (ch < '0' as WCHAR || ch > '9' as WCHAR)
                {
                    SetLastError(ERROR_INVALID_NAME as DWORD);
                    return 0;
                }

                digit = if ch <= '9' as WCHAR {
                    (ch as INT) - ('0' as INT) + ('z' as INT) - ('a' as INT)
                        + 1
                } else {
                    (ch as INT) - ('a' as INT)
                };
                pos += digit * w;

                t = if k <= bias {
                    TMIN
                } else if k >= bias + TMAX {
                    TMAX
                } else {
                    k - bias
                };
                if digit < t {
                    break;
                }
                w *= BASE - t;
                k += BASE;
            }

            bias = adapt(
                pos - old_pos,
                out - out_label + 1,
                (old_pos == 0) as INT,
            );
            n += pos / (out - out_label + 1);
            pos %= out - out_label + 1;

            if (dwFlags & IDN_ALLOW_UNASSIGNED) == 0
                && get_table_entry(
                    core::ptr::addr_of!(nameprep_char_type) as *const u16,
                    n as WCHAR,
                ) as INT
                    == 1
            {
                SetLastError(ERROR_INVALID_NAME as DWORD);
                return 0;
            }

            // Divergence from the C, which bounds this insert by `cchASCIIChar` (the INPUT
            // length) where every other write in the function checks `cchUnicodeChar`: a
            // caller whose output buffer is shorter than its input got a write past its end.
            // Later Wine checks `cchUnicodeChar` here too.
            if lpUnicodeCharStr.is_null() {
                out += 1;
            } else if out + 1 <= cchUnicodeChar {
                memmove(
                    lpUnicodeCharStr.offset((out_label + pos + 1) as isize)
                        as *mut core::ffi::c_void,
                    lpUnicodeCharStr.offset((out_label + pos) as isize)
                        as *const core::ffi::c_void,
                    ((out - out_label - pos) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                *lpUnicodeCharStr.offset((out_label + pos) as isize) = n as WCHAR;
                out += 1;
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }
            pos += 1;
        }

        if out - out_label > 63 {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0;
        }

        if label_end < cchASCIIChar {
            if lpUnicodeCharStr.is_null() {
                out += 1;
            } else if out + 1 <= cchUnicodeChar {
                *lpUnicodeCharStr.offset(out as isize) =
                    *lpASCIICharStr.offset(label_end as isize);
                out += 1;
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }
        }
        label_start = label_end + 1;
    }

    return out;
}
