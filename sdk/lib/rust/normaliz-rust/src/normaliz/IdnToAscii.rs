use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static nameprep_char_type: [::core::ffi::c_ushort; 0];
    static nameprep_mapping: [WCHAR; 0];
}

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS (valueless flag, no Rust constant emitted)
// #define NDEBUG (valueless flag, no Rust constant emitted)

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

unsafe extern "C" fn get_table_entry(table: *const u16, ch: WCHAR) -> u16 {
    let a: usize = *table.add((ch >> 8) as usize) as usize;
    let b: usize = *table.add(a + ((ch >> 4) & 0x0f) as usize) as usize;
    return *table.add(b + (ch & 0xf) as usize);
}

#[no_mangle]
pub unsafe extern "system" fn IdnToNameprepUnicode(dwFlags: DWORD, lpUnicodeCharStr: LPCWSTR, mut cchUnicodeChar: INT, lpNameprepCharStr: LPWSTR, cchNameprepChar: INT) -> INT {
    const UNASSIGNED: i32 = 0x1;
    const PROHIBITED: i32 = 0x2;
    const BIDI_RAL: i32 = 0x4;
    const BIDI_L: i32 = 0x8;

    let mut ptr: *const WCHAR = core::ptr::null();
    let mut flags: WORD = 0;
    let mut buf: [WCHAR; 64] = [0; 64];
    let mut map_str: *mut WCHAR = core::ptr::null_mut();
    let mut norm_str: [WCHAR; 64] = [0; 64];
    let mut ch: WCHAR = 0;
    let mut i: DWORD = 0;
    let mut map_len: DWORD = 0;
    let mut norm_len: DWORD = 0;
    let mut mask: DWORD = 0;
    let mut label_start: DWORD = 0;
    let mut label_end: DWORD = 0;
    let mut out: DWORD = 0 as DWORD;
    let mut have_bidi_ral: BOOL = 0;
    let mut prohibit_bidi_ral: BOOL = 0;
    let mut ascii_only: BOOL = 0;

    DPRINT!("%x %p %d %p %d\n", dwFlags, lpUnicodeCharStr, cchUnicodeChar, lpNameprepCharStr, cchNameprepChar);

    if dwFlags & !((IDN_ALLOW_UNASSIGNED | IDN_USE_STD3_ASCII_RULES) as DWORD) != 0 {
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        return 0 as INT;
    }

    if lpUnicodeCharStr.is_null() || cchUnicodeChar < -(1 as INT) {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0 as INT;
    }

    if cchUnicodeChar == -(1 as INT) {
        cchUnicodeChar = (strlenW(lpUnicodeCharStr) as UINT).wrapping_add(1 as UINT) as INT;
    }
    if cchUnicodeChar == 0
        || (cchUnicodeChar == 1 as INT
            && *lpUnicodeCharStr.offset(0 as isize) == 0 as WCHAR)
    {
        SetLastError(ERROR_INVALID_NAME as DWORD);
        return 0 as INT;
    }

    label_start = 0 as DWORD;
    while label_start < (cchUnicodeChar as UINT) {
        ascii_only = TRUE as BOOL;
        i = label_start;
        while i < (cchUnicodeChar as UINT) {
            ch = *lpUnicodeCharStr.offset(i as isize);

            if i != ((cchUnicodeChar - (1 as INT)) as UINT) && ch == 0 {
                SetLastError(ERROR_INVALID_NAME as DWORD);
                return 0 as INT;
            }
            if ch == 0
                || (ch as INT) == ('.' as INT)
                || (ch as INT) == (0x3002 as INT)
                || (ch as INT) == (0xff0e as INT)
                || (ch as INT) == (0xff61 as INT)
            {
                break;
            }

            if (ch as INT) > (0x7f as INT) {
                ascii_only = FALSE as BOOL;
                i = i.wrapping_add(1);
                continue;
            }

            if (dwFlags & (IDN_USE_STD3_ASCII_RULES as DWORD)) == (0 as DWORD) {
                i = i.wrapping_add(1);
                continue;
            }
            if (ch >= ('a' as WCHAR) && ch <= ('z' as WCHAR))
                || (ch >= ('A' as WCHAR) && ch <= ('Z' as WCHAR))
                || (ch >= ('0' as WCHAR) && ch <= ('9' as WCHAR))
                || ch == ('-' as WCHAR)
            {
                i = i.wrapping_add(1);
                continue;
            }

            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0 as INT;
            #[allow(unreachable_code)]
            {
                i = i.wrapping_add(1);
            }
        }
        label_end = i;
        /* last label may be empty */
        if label_start == label_end && ch != 0 {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0 as INT;
        }

        if (dwFlags & (IDN_USE_STD3_ASCII_RULES as DWORD)) != 0
            && (*lpUnicodeCharStr.offset(label_start as isize) == ('-' as WCHAR)
                || *lpUnicodeCharStr.offset(label_end.wrapping_sub(1 as DWORD) as isize) == ('-' as WCHAR))
        {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0 as INT;
        }

        if ascii_only != 0 {
            /* maximal label length is 63 characters */
            if label_end.wrapping_sub(label_start) > (63 as DWORD) {
                SetLastError(ERROR_INVALID_NAME as DWORD);
                return 0 as INT;
            }
            if label_end < (cchUnicodeChar as UINT) {
                label_end = label_end.wrapping_add(1);
            }

            if lpNameprepCharStr.is_null() {
                out = out.wrapping_add(label_end.wrapping_sub(label_start));
            } else if out.wrapping_add(label_end).wrapping_sub(label_start) <= (cchNameprepChar as UINT) {
                memcpy(
                    lpNameprepCharStr.offset(out as isize) as *mut core::ffi::c_void,
                    lpUnicodeCharStr.offset(label_start as isize) as *const core::ffi::c_void,
                    (label_end.wrapping_sub(label_start) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                if (*lpUnicodeCharStr.offset(label_end.wrapping_sub(1 as DWORD) as isize)) as INT
                    > (0x7f as INT)
                {
                    *lpNameprepCharStr.offset(
                        out.wrapping_add(label_end)
                            .wrapping_sub(label_start)
                            .wrapping_sub(1 as DWORD) as isize,
                    ) = '.' as WCHAR;
                }
                out = out.wrapping_add(label_end.wrapping_sub(label_start));
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }

            label_start = label_end;
            continue;
        }

        map_len = 0 as DWORD;
        i = label_start;
        while i < label_end {
            ch = *lpUnicodeCharStr.offset(i as isize);
            ptr = (&raw const nameprep_mapping as *const WCHAR).offset(
                *(&raw const nameprep_mapping as *const WCHAR)
                    .offset(((ch as INT) >> (8 as INT)) as isize) as INT as isize,
            );
            ptr = (&raw const nameprep_mapping as *const WCHAR)
                .offset(
                    *ptr.offset((((ch as INT) >> (4 as INT)) & (0x0f as INT)) as isize)
                        as INT as isize,
                )
                .offset(((3 as INT) * ((ch as INT) & (0x0f as INT))) as isize);

            if *ptr.offset(0 as isize) == 0 {
                map_len = map_len.wrapping_add(1);
            } else if *ptr.offset(1 as isize) == 0 {
                map_len = map_len.wrapping_add(1);
            } else if *ptr.offset(2 as isize) == 0 {
                map_len = map_len.wrapping_add(2 as DWORD);
            } else if *ptr.offset(0 as isize) != (0xffff as WCHAR)
                || *ptr.offset(1 as isize) != (0xffff as WCHAR)
                || *ptr.offset(2 as isize) != (0xffff as WCHAR)
            {
                map_len = map_len.wrapping_add(3 as DWORD);
            }
            i = i.wrapping_add(1);
        }
        if (map_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
            > core::mem::size_of::<[WCHAR; 64]>()
        {
            map_str = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                ((map_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T,
            ) as *mut WCHAR;
            if map_str.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                return 0 as INT;
            }
        } else {
            map_str = &raw mut buf as *mut WCHAR;
        }

        map_len = 0 as DWORD;
        i = label_start;
        while i < label_end {
            ch = *lpUnicodeCharStr.offset(i as isize);
            ptr = (&raw const nameprep_mapping as *const WCHAR).offset(
                *(&raw const nameprep_mapping as *const WCHAR)
                    .offset(((ch as INT) >> (8 as INT)) as isize) as INT as isize,
            );
            ptr = (&raw const nameprep_mapping as *const WCHAR)
                .offset(
                    *ptr.offset((((ch as INT) >> (4 as INT)) & (0x0f as INT)) as isize)
                        as INT as isize,
                )
                .offset(((3 as INT) * ((ch as INT) & (0x0f as INT))) as isize);

            if *ptr.offset(0 as isize) == 0 {
                *map_str.offset(map_len as isize) = ch;
                map_len = map_len.wrapping_add(1);
            } else if *ptr.offset(1 as isize) == 0 {
                *map_str.offset(map_len as isize) = *ptr.offset(0 as isize);
                map_len = map_len.wrapping_add(1);
            } else if *ptr.offset(2 as isize) == 0 {
                *map_str.offset(map_len as isize) = *ptr.offset(0 as isize);
                map_len = map_len.wrapping_add(1);
                *map_str.offset(map_len as isize) = *ptr.offset(1 as isize);
                map_len = map_len.wrapping_add(1);
            } else if *ptr.offset(0 as isize) != (0xffff as WCHAR)
                || *ptr.offset(1 as isize) != (0xffff as WCHAR)
                || *ptr.offset(2 as isize) != (0xffff as WCHAR)
            {
                *map_str.offset(map_len as isize) = *ptr.offset(0 as isize);
                map_len = map_len.wrapping_add(1);
                *map_str.offset(map_len as isize) = *ptr.offset(1 as isize);
                map_len = map_len.wrapping_add(1);
                *map_str.offset(map_len as isize) = *ptr.offset(2 as isize);
                map_len = map_len.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        norm_len = FoldStringW(
            MAP_FOLDCZONE as DWORD,
            map_str as LPCWSTR,
            map_len as INT,
            &raw mut norm_str as LPWSTR,
            ((core::mem::size_of::<[WCHAR; 64]>() / core::mem::size_of::<WCHAR>())
                - (1 as usize)) as INT,
        ) as DWORD;
        if map_str != (&raw mut buf as *mut WCHAR) {
            HeapFree(GetProcessHeap(), 0 as DWORD, map_str as *mut core::ffi::c_void);
        }
        if norm_len == 0 {
            if GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD {
                SetLastError(ERROR_INVALID_NAME as DWORD);
            }
            return 0 as INT;
        }

        if label_end < (cchUnicodeChar as UINT) {
            let fresh26 = norm_len;
            norm_len = norm_len.wrapping_add(1);
            norm_str[fresh26 as usize] = (if *lpUnicodeCharStr.offset(label_end as isize) != 0 {
                '.' as INT
            } else {
                0 as INT
            }) as WCHAR;
            label_end = label_end.wrapping_add(1);
        }

        if lpNameprepCharStr.is_null() {
            out = out.wrapping_add(norm_len);
        } else if out.wrapping_add(norm_len) <= (cchNameprepChar as UINT) {
            memcpy(
                lpNameprepCharStr.offset(out as isize) as *mut core::ffi::c_void,
                &raw mut norm_str as *mut WCHAR as *const core::ffi::c_void,
                (norm_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            out = out.wrapping_add(norm_len);
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return 0 as INT;
        }

        prohibit_bidi_ral = FALSE as BOOL;
        have_bidi_ral = prohibit_bidi_ral;
        mask = PROHIBITED as DWORD;
        if (dwFlags & (IDN_ALLOW_UNASSIGNED as DWORD)) == (0 as DWORD) {
            mask |= UNASSIGNED as DWORD;
        }
        i = 0 as DWORD;
        while i < norm_len {
            ch = norm_str[i as usize];
            flags = get_table_entry(&raw const nameprep_char_type as *const u16, ch) as WORD;

            if (flags as DWORD) & mask != 0 {
                SetLastError(
                    (if ((flags as INT) & (PROHIBITED as INT)) != 0 {
                        ERROR_INVALID_NAME as DWORD
                    } else {
                        ERROR_NO_UNICODE_TRANSLATION as DWORD
                    }),
                );
                return 0 as INT;
            }

            if ((flags as INT) & (BIDI_RAL as INT)) != 0 {
                have_bidi_ral = TRUE as BOOL;
            }
            if ((flags as INT) & (BIDI_L as INT)) != 0 {
                prohibit_bidi_ral = TRUE as BOOL;
            }
            i = i.wrapping_add(1);
        }

        if have_bidi_ral != 0 {
            ch = norm_str[0 as usize];
            flags = get_table_entry(&raw const nameprep_char_type as *const u16, ch) as WORD;
            if ((flags as INT) & (BIDI_RAL as INT)) == (0 as INT) {
                prohibit_bidi_ral = TRUE as BOOL;
            }

            ch = norm_str[norm_len.wrapping_sub(1 as DWORD) as usize];
            flags = get_table_entry(&raw const nameprep_char_type as *const u16, ch) as WORD;
            if ((flags as INT) & (BIDI_RAL as INT)) == (0 as INT) {
                prohibit_bidi_ral = TRUE as BOOL;
            }
        }

        if have_bidi_ral != 0 && prohibit_bidi_ral != 0 {
            SetLastError(ERROR_INVALID_NAME as DWORD);
            return 0 as INT;
        }

        label_start = label_end;
    }

    return out as INT;
}

#[no_mangle]
pub unsafe extern "system" fn IdnToAscii(dwFlags: DWORD, lpUnicodeCharStr: LPCWSTR, cchUnicodeChar: INT, lpASCIICharStr: LPWSTR, cchASCIIChar: INT) -> INT {
    static prefixW: [WCHAR; 4] = [0x0078u16, 0x006E, 0x002D, 0x002D];

    let mut norm_str: *mut WCHAR = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut label_start: INT = 0;
    let mut label_end: INT = 0;
    let mut norm_len: INT = 0;
    let mut out_label: INT = 0;
    let mut out: INT = 0;

    DPRINT!("%x %p %d %p %d\n", dwFlags, lpUnicodeCharStr, cchUnicodeChar, lpASCIICharStr, cchASCIIChar);

    norm_len = IdnToNameprepUnicode(dwFlags, lpUnicodeCharStr, cchUnicodeChar, core::ptr::null_mut(), 0) as INT;
    if norm_len == 0 {
        return 0;
    }
    norm_str = HeapAlloc(GetProcessHeap(), 0, ((norm_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T) as *mut WCHAR;
    if norm_str.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    norm_len = IdnToNameprepUnicode(dwFlags, lpUnicodeCharStr, cchUnicodeChar, norm_str, norm_len) as INT;
    if norm_len == 0 {
        HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
        return 0;
    }

    label_start = 0;
    while label_start < norm_len {
        let mut n: INT = INIT_N;
        let mut bias: INT = INIT_BIAS;
        let mut delta: INT = 0;
        let mut b: INT = 0;
        let mut h: INT = 0;

        out_label = out;
        i = label_start;
        while i < norm_len
            && (*norm_str.offset(i as isize) as INT) != '.' as INT
            && (*norm_str.offset(i as isize) as INT) != 0x3002
            && (*norm_str.offset(i as isize) as INT) != '\0' as INT
        {
            if (*norm_str.offset(i as isize) as INT) < 0x80 {
                b += 1;
            }
            i += 1;
        }
        label_end = i;

        if b == label_end - label_start {
            if label_end < norm_len {
                b += 1;
            }
            if lpASCIICharStr.is_null() {
                out += b;
            } else if out + b <= cchASCIIChar {
                memcpy(
                    lpASCIICharStr.offset(out as isize) as *mut core::ffi::c_void,
                    norm_str.offset(label_start as isize) as *const core::ffi::c_void,
                    (b as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                out += b;
            } else {
                HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
            label_start = label_end + 1;
            continue;
        }

        if lpASCIICharStr.is_null() {
            out += 5 + b; /* strlen(xn--...-) */
        } else if out + 5 + b <= cchASCIIChar {
            memcpy(
                lpASCIICharStr.offset(out as isize) as *mut core::ffi::c_void,
                core::ptr::addr_of!(prefixW) as *const core::ffi::c_void,
                core::mem::size_of::<[WCHAR; 4]>(),
            );
            out += 4;
            i = label_start;
            while i < label_end {
                if (*norm_str.offset(i as isize) as INT) < 0x80 {
                    *lpASCIICharStr.offset(out as isize) = *norm_str.offset(i as isize);
                    out += 1;
                }
                i += 1;
            }
            *lpASCIICharStr.offset(out as isize) = '-' as WCHAR;
            out += 1;
        } else {
            HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }
        if b == 0 {
            out -= 1;
        }

        h = b;
        while h < label_end - label_start {
            let mut m: INT = 0xffff;
            let mut q: INT = 0;
            let mut k: INT = 0;

            i = label_start;
            while i < label_end {
                if (*norm_str.offset(i as isize) as INT) >= n && m > (*norm_str.offset(i as isize) as INT) {
                    m = *norm_str.offset(i as isize) as INT;
                }
                i += 1;
            }
            delta += (m - n) * (h + 1);
            n = m;

            i = label_start;
            while i < label_end {
                if (*norm_str.offset(i as isize) as INT) < n {
                    delta += 1;
                } else if (*norm_str.offset(i as isize) as INT) == n {
                    q = delta;
                    k = BASE;
                    loop {
                        let t: INT = if k <= bias {
                            TMIN
                        } else if k >= bias + TMAX {
                            TMAX
                        } else {
                            k - bias
                        };
                        let disp: INT = if q < t {
                            q
                        } else {
                            t + (q - t) % (BASE - t)
                        };
                        if lpASCIICharStr.is_null() {
                            out += 1;
                        } else if out + 1 <= cchASCIIChar {
                            *lpASCIICharStr.offset(out as isize) = (if disp <= 'z' as INT - 'a' as INT {
                                'a' as INT + disp
                            } else {
                                '0' as INT + disp - 'z' as INT + 'a' as INT - 1
                            }) as WCHAR;
                            out += 1;
                        } else {
                            HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
                            SetLastError(ERROR_INSUFFICIENT_BUFFER);
                            return 0;
                        }
                        if q < t {
                            break;
                        }
                        q = (q - t) / (BASE - t);
                        k += BASE;
                    }
                    bias = adapt(delta, h + 1, (h == b) as INT);
                    delta = 0;
                    h += 1;
                }
                i += 1;
            }
            delta += 1;
            n += 1;
        }

        if out - out_label > 63 {
            HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
            SetLastError(ERROR_INVALID_NAME);
            return 0;
        }

        if label_end < norm_len {
            if lpASCIICharStr.is_null() {
                out += 1;
            } else if out + 1 <= cchASCIIChar {
                *lpASCIICharStr.offset(out as isize) = (if *norm_str.offset(label_end as isize) as INT != 0 {
                    '.' as INT
                } else {
                    0
                }) as WCHAR;
                out += 1;
            } else {
                HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
        }
        label_start = label_end + 1;
    }

    HeapFree(GetProcessHeap(), 0, norm_str as *mut core::ffi::c_void);
    return out;
}
