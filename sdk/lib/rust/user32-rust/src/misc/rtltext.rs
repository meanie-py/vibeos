// win32ss/user/rtl/text.c — USERMODE (user32.dll) Rust translation.
//
// This is the `#else` (non-`_WIN32K_`) branch of the dual-compiled DrawText layout
// engine. The Wine layout logic (ellipsis / word-break / tab expansion / prefix /
// kinsoku-CJK) is identical between win32k.sys and user32.dll — it was cribbed verbatim
// from the kernel translation (sdk/lib/rust/win32k-rust/src/rtl_text.rs). The ONLY
// differences are the GDI substrate: the kernel Gre*/NtGdi* calls are replaced by the
// gdi32/kernel32 public exports here, and the kernel-only `UserExtTextOutW`
// (KeUserModeCallback shim) is dropped — usermode DrawTextExWorker calls ExtTextOutW
// directly.
use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// CRT cdecl helpers — extern "C".
extern "C" {
    // strlenW is `#define strlenW(s) wcslen(s)` (wine/unicode.h) — not a real symbol.
    #[link_name = "wcslen"]
    fn strlenW(s: LPCWSTR) -> usize;
    // strrchrW is `#define strrchrW(s,c) wcsrchr(s,c)` (wine/unicode.h) — same as wcsrchr.
    fn wcsrchr(s: *const u16, c: i32) -> *mut u16;
    fn wcschr(s: *const u16, c: i32) -> *mut u16;
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

// gdi32.dll / kernel32.dll public exports — stdcall on i386 (`_Name@N`), cdecl on
// amd64/arm64; `extern "system"` resolves to the right thing per arch. These link via
// the gdi32/kernel32 import libs.
extern "system" {
    fn GetTextExtentExPointW(
        hdc: HDC,
        lpszString: LPCWSTR,
        cchString: i32,
        nMaxExtent: i32,
        lpnFit: *mut i32,
        lpnDx: *mut i32,
        lpSize: *mut SIZE,
    ) -> i32;
    fn GetTextExtentPointW(hdc: HDC, lpString: LPCWSTR, cbString: i32, lpSize: *mut SIZE) -> i32;
    fn GetTextMetricsW(hdc: HDC, lptm: *mut TEXTMETRICW) -> i32;
    fn GetTextColor(hdc: HDC) -> u32;
    fn ExtTextOutW(
        hdc: HDC,
        x: i32,
        y: i32,
        fuOptions: u32,
        lprc: *const RECT,
        lpString: LPCWSTR,
        cchCount: u32,
        lpDx: *const i32,
    ) -> i32;
    fn CreatePen(iStyle: i32, cWidth: i32, color: u32) -> HPEN;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn MoveToEx(hdc: HDC, x: i32, y: i32, lppt: *mut core::ffi::c_void) -> i32;
    fn LineTo(hdc: HDC, x: i32, y: i32) -> i32;
    fn DeleteObject(ho: HGDIOBJ) -> i32;
    fn GetGraphicsMode(hdc: HDC) -> i32;
    fn GetWindowExtEx(hdc: HDC, lpsize: *mut SIZE) -> i32;
    fn GetViewportExtEx(hdc: HDC, lpsize: *mut SIZE) -> i32;
    fn CreateRectRgn(x1: i32, y1: i32, x2: i32, y2: i32) -> HRGN;
    fn GetClipRgn(hdc: HDC, hrgn: HRGN) -> i32;
    fn IntersectClipRect(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> i32;
    fn GdiFlush() -> i32;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn ExitProcess(uExitCode: u32) -> !;
}

// C `assert` (live under DBG) is debug_assert! -- the kernel's ASSERT convention:
// checked in a debug build, never evaluated in release. It was an empty fn, which
// never checked yet evaluated its argument on every call.

// ---- file-scope preamble (translated) ----
pub const TAB: i32 = 9;
pub const CR: i32 = 13;
pub const LF: i32 = 10;
pub const SPACE: i32 = 32;
pub const PREFIX: i32 = 38;
pub const ALPHA_PREFIX: i32 = 30;
pub const KANA_PREFIX: i32 = 31;
pub const FORWARD_SLASH: u8 = b'/';
pub const BACK_SLASH: u8 = b'\\';

pub static ELLIPSISW: [u16; 4] = [b'.' as u16, b'.' as u16, b'.' as u16, 0];

#[repr(C)]
pub struct tag_ellipsis_data {
    pub before: i32,
    pub len: i32,
    pub under: i32,
    pub after: i32,
}

pub type ellipsis_data = tag_ellipsis_data;

pub static KinsokuClassA: [u16; 78] = [
    0x2010, 0x2013, 0x2019, 0x201D, 0x203C, 0x2047, 0x2048, 0x2049, 0x3001,
    0x3002, 0x3005, 0x3009, 0x300B, 0x300D, 0x300F, 0x3011, 0x3015, 0x3017,
    0x3019, 0x301C, 0x301F, 0x303B, 0x3041, 0x3043, 0x3045, 0x3047, 0x3049,
    0x3063, 0x3083, 0x3085, 0x3087, 0x308E, 0x3095, 0x3096, 0x30A0, 0x30A1,
    0x30A3, 0x30A5, 0x30A7, 0x30A9, 0x30C3, 0x30E3, 0x30E5, 0x30E7, 0x30EE,
    0x30F5, 0x30F6, 0x30FB, 0x30FC, 0x30FD, 0x30FE, 0x31F0, 0x31F1, 0x31F2,
    0x31F3, 0x31F4, 0x31F5, 0x31F6, 0x31F7, 0x31F8, 0x31F9, 0x31FA, 0x31FB,
    0x31FC, 0x31FD, 0x31FE, 0x31FF, 0xFF01, 0xFF09, 0xFF0C, 0xFF0E, 0xFF1A,
    0xFF1B, 0xFF1F, 0xFF3D, 0xFF5D, 0xFF60, 0
];

pub const MAX_BUFFER: usize = 1024;

// `void _font_assert(...)` — non-static in C, so keep it linkable. The usermode `#else`
// body is `ExitProcess(3); for(;;);` (TRACE/DbgPrint is a no-op here — don't pull it in).
#[no_mangle]
pub unsafe extern "C" fn _font_assert(_msg: *const i8, _file: *const i8, _line: i32) {
    ExitProcess(3);
}

unsafe fn TEXT_Ellipsify(
    hdc: HDC,
    str: LPWSTR,
    max_len: u32,
    len_str: *mut u32,
    width: i32,
    size: *mut SIZE,
    modstr: LPWSTR,
    len_before: *mut i32,
    len_ellip: *mut i32,
) {

    let mut len_ellipsis: u32;
    let mut lo: u32;
    let mut mid: u32;
    let mut hi: u32;

    len_ellipsis = strlenW(ELLIPSISW.as_ptr()) as u32;
    if len_ellipsis > max_len {
        len_ellipsis = max_len;
    }
    if *len_str > max_len - len_ellipsis {
        *len_str = max_len - len_ellipsis;
    }

    if *len_str > 0
        && GetTextExtentExPointW(
            hdc,
            str,
            *len_str as i32,
            width,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            size,
        ) != 0
        && (*size).cx > width
    {
        lo = 0;
        hi = *len_str;
        while lo < hi {
            mid = (lo + hi) / 2;
            if GetTextExtentExPointW(
                hdc,
                str,
                mid as i32,
                width,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                size,
            ) == 0
            {
                break;
            }
            if (*size).cx > width {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        *len_str = hi;
    }

    loop {
        core::ptr::copy_nonoverlapping(
            ELLIPSISW.as_ptr(),
            str.offset(*len_str as isize),
            len_ellipsis as usize,
        );
        if GetTextExtentExPointW(
            hdc,
            str,
            (*len_str + len_ellipsis) as i32,
            width,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            size,
        ) == 0
        {
            break;
        }
        if *len_str == 0 || (*size).cx <= width {
            break;
        }

        *len_str -= 1;
    }

    *len_ellip = len_ellipsis as i32;
    *len_before = *len_str as i32;
    *len_str += len_ellipsis;

    if !modstr.is_null() {
        core::ptr::copy_nonoverlapping(str, modstr, *len_str as usize);
        *modstr.offset(*len_str as isize) = 0;
    }
}

unsafe fn TEXT_PathEllipsify(
    hdc: HDC,
    str: LPWSTR,
    max_len: u32,
    len_str: *mut u32,
    width: i32,
    size: *mut SIZE,
    modstr: LPWSTR,
    pellip: *mut ellipsis_data,
) {
    let mut len_ellipsis: i32;
    let mut len_trailing: i32;
    let mut len_under: i32;
    let lastBkSlash: LPWSTR;
    let lastFwdSlash: LPWSTR;
    let mut lastSlash: LPWSTR;

    len_ellipsis = strlenW(ELLIPSISW.as_ptr()) as i32;
    if max_len == 0 {
        return;
    }
    if len_ellipsis as u32 >= max_len {
        len_ellipsis = max_len as i32 - 1;
    }
    if *len_str + len_ellipsis as u32 >= max_len {
        *len_str = max_len - len_ellipsis as u32 - 1;
    }
    *str.offset(*len_str as isize) = 0;

    lastBkSlash = wcsrchr(str, BACK_SLASH as i32);
    lastFwdSlash = wcsrchr(str, FORWARD_SLASH as i32);

    lastSlash = if lastBkSlash as usize > lastFwdSlash as usize {
        lastBkSlash
    } else {
        lastFwdSlash
    };
    if lastSlash.is_null() {
        lastSlash = str;
    }
    len_trailing = *len_str as i32 - (lastSlash.offset_from(str) as i32);

    memmove(
        lastSlash.offset(len_ellipsis as isize) as *mut core::ffi::c_void,
        lastSlash as *const core::ffi::c_void,
        (len_trailing as usize) * core::mem::size_of::<u16>(),
    );
    memcpy(
        lastSlash as *mut core::ffi::c_void,
        ELLIPSISW.as_ptr() as *const core::ffi::c_void,
        (len_ellipsis as usize) * core::mem::size_of::<u16>(),
    );
    len_trailing += len_ellipsis;

    len_under = 0;
    loop {
        if GetTextExtentExPointW(
            hdc,
            str,
            (*len_str + len_ellipsis as u32) as i32,
            width,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            size,
        ) == 0
        {
            break;
        }
        if lastSlash == str || (*size).cx <= width {
            break;
        }

        memmove(
            lastSlash.offset(-1) as *mut core::ffi::c_void,
            lastSlash as *const core::ffi::c_void,
            (len_trailing as usize) * core::mem::size_of::<u16>(),
        );
        lastSlash = lastSlash.offset(-1);
        len_under += 1;

        debug_assert!(*len_str != 0);
        *len_str -= 1;
    }
    (*pellip).before = lastSlash.offset_from(str) as i32;
    (*pellip).len = len_ellipsis;
    (*pellip).under = len_under;
    (*pellip).after = len_trailing - len_ellipsis;
    *len_str = (*len_str as i32 + len_ellipsis) as u32;

    if !modstr.is_null() {
        memcpy(
            modstr as *mut core::ffi::c_void,
            str as *const core::ffi::c_void,
            (*len_str as usize) * core::mem::size_of::<u16>(),
        );
        *modstr.offset(*len_str as isize) = 0;
    }
}

unsafe fn IsCJKT(wch: u16) -> u8 {
    if 0x0E00 <= wch && wch <= 0x0E7F {
        return 1u8;
    }

    if 0x3000 <= wch && wch <= 0x9FFF {
        return 1u8;
    }

    if 0xAC00 <= wch && wch <= 0xD7FF {
        return 1u8;
    }

    if 0xFF00 <= wch && wch <= 0xFFEF {
        return 1u8;
    }

    0u8
}

unsafe fn TEXT_WordBreak(
    hdc: HDC,
    str: LPWSTR,
    _max_str: u32,
    len_str: *mut u32,
    width: i32,
    format: u32,
    chars_fit: u32,
    chars_used: *mut u32,
    size: *mut SIZE,
) {
    let mut p: LPWSTR;
    let mut word_fits: i32;

    debug_assert!((format & DT_WORDBREAK) != 0);
    debug_assert!(chars_fit < *len_str);

    p = str.add(chars_fit as usize);
    word_fits = 1;
    if chars_fit == 0 {
        word_fits = 0;
    } else if *p == SPACE as u16 {
        p = p.sub(1);
    } else {
        while p > str
            && {
                p = p.sub(1);
                *p != SPACE as u16
            }
            && (IsCJKT(*p.add(1)) == 0
                || *p.add(1) == 0
                || wcschr(KinsokuClassA.as_ptr(), *p.add(1) as i32) != core::ptr::null_mut())
        {}
        word_fits = (p != str || *p == SPACE as u16 || IsCJKT(*p.add(1)) != 0) as i32;
    }

    if word_fits != 0 {
        let next_is_space: i32;
        if (format & (DT_RIGHT | DT_CENTER)) == 0 || *p != SPACE as u16 {
            p = p.add(1);
        }
        next_is_space = ((p.offset_from(str) as usize) < *len_str as usize && *p == SPACE as u16) as i32;
        *len_str = p.offset_from(str) as u32;
        *chars_used = *len_str;
        if next_is_space != 0 {
            *chars_used += 1;
        }
    } else {
        if (format & (DT_EDITCONTROL | DT_WORD_ELLIPSIS | DT_PATH_ELLIPSIS)) == DT_EDITCONTROL {
            if chars_fit == 0 {
                *len_str = 1;
                *chars_used = 1;
            } else {
                *len_str = chars_fit;
                *chars_used = chars_fit;
            }
        } else {
            let e: LPWSTR = str.add(*len_str as usize);
            p = str.add(chars_fit as usize);
            while p < e && *p != SPACE as u16 {
                p = p.add(1);
            }
            *chars_used = p.offset_from(str) as u32;
            if p < e {
                *chars_used += 1;
            }
            *len_str = p.offset_from(str) as u32;
        }
    }

    // Remeasure the string (usermode public export).
    GetTextExtentExPointW(hdc, str, *len_str as i32, 0, core::ptr::null_mut(), core::ptr::null_mut(), size);
}

unsafe fn TEXT_SkipChars(
    new_count: *mut i32,
    new_str: *mut *const u16,
    start_count: i32,
    start_str: *const u16,
    max: i32,
    n: i32,
    prefix: i32,
) {
    let mut start_count = start_count;
    let mut start_str = start_str;
    let mut max = max;
    let mut n = n;

    if prefix != 0 {
        let str_on_entry = start_str;
        debug_assert!(max >= n);
        max -= n;
        while n > 0 {
            n -= 1;
            if (*start_str == PREFIX as u16 || *start_str == ALPHA_PREFIX as u16) && max > 0 {
                max -= 1;
                start_str = start_str.offset(1);
            }
            start_str = start_str.offset(1);
        }
        start_count -= (start_str as isize - str_on_entry as isize) as i32 / core::mem::size_of::<u16>() as i32;
    } else {
        start_str = start_str.offset(n as isize);
        start_count -= n;
    }
    *new_str = start_str;
    *new_count = start_count;
}

unsafe fn TEXT_Reprefix(
    mut str: *const u16,
    mut ns: u32,
    pe: *const ellipsis_data,
) -> i32 {
    let mut result: i32 = -1;
    let n: u32 = ((*pe).before + (*pe).under + (*pe).after) as u32;
    debug_assert!(n <= ns);

    let mut i: u32 = 0;
    while i < n {
        if i == (*pe).before as u32 {
            if ns < (*pe).under as u32 {
                break;
            }
            str = str.offset((*pe).under as isize);
            ns -= (*pe).under as u32;
            i += (*pe).under as u32;
            if (*pe).after == 0 {
                break;
            }
        }
        if ns == 0 {
            break;
        }
        ns -= 1;
        // C: `if (*str++ == PREFIX || *str == ALPHA_PREFIX)` — post-increment evaluates
        // *str then advances, so the second test reads the NEXT char.
        let first = *str;
        str = str.offset(1);
        if first == PREFIX as u16 || *str == ALPHA_PREFIX as u16 {
            str = str.offset(1);
            if ns == 0 {
                break;
            }
            if *str != PREFIX as u16 {
                result = if i < (*pe).before as u32 || (*pe).under == 0 {
                    i as i32
                } else {
                    (i - (*pe).under as u32 + (*pe).len as u32) as i32
                };
            }
            ns -= 1;
        }
        // for (...; i++, str++) — the loop-increment advances both at the end of
        // every iteration (in addition to the in-body `str++`s above).
        str = str.offset(1);
        i += 1;
    }
    result
}

unsafe fn remainder_is_none_or_newline(num_chars: i32, str: *const u16) -> i32 {
    let mut num_chars = num_chars;
    let mut str = str;

    if num_chars == 0 {
        return 1;
    }
    if *str != LF as u16 && *str != CR as u16 {
        return 0;
    }
    num_chars -= 1;
    if num_chars == 0 {
        return 1;
    }
    if *str == *str.offset(1) {
        return 0;
    }
    str = str.offset(1);
    if *str != CR as u16 && *str != LF as u16 {
        return 0;
    }
    num_chars -= 1;
    if num_chars != 0 {
        return 0;
    }
    return 1;
}

unsafe fn TEXT_NextLineW(
    hdc: HDC,
    str: *const u16,
    count: *mut i32,
    dest: *mut u16,
    len: *mut i32,
    width: i32,
    format: u32,
    retsize: *mut SIZE,
    last_line: i32,
    p_retstr: *mut LPWSTR,
    tabwidth: i32,
    pprefix_offset: *mut i32,
    pellip: *mut ellipsis_data,
) -> *const u16 {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut plen: i32 = 0;
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };
    let maxl: i32 = *len;
    let mut seg_i: i32;
    let mut seg_count: i32;
    let mut seg_j: i32;
    let mut max_seg_width: i32;
    let mut num_fit: i32 = 0;
    let mut word_broken: i32;
    let mut line_fits: i32;
    let mut j_in_seg: u32;
    let mut ellipsified: i32;
    *pprefix_offset = -1;

    (*retsize).cy = 0;
    while *count != 0 {
        if *str.offset(i as isize) == TAB as u16 && (format & DT_EXPANDTABS) != 0 {
            plen = ((plen / tabwidth) + 1) * tabwidth;
            *count -= 1;
            if j < maxl {
                *dest.offset(j as isize) = *str.offset(i as isize);
                j += 1;
                i += 1;
            } else {
                i += 1;
            }
            while *count != 0 && *str.offset(i as isize) == TAB as u16 {
                plen += tabwidth;
                *count -= 1;
                if j < maxl {
                    *dest.offset(j as isize) = *str.offset(i as isize);
                    j += 1;
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }

        seg_i = i;
        seg_count = *count;
        seg_j = j;

        while *count != 0
            && (*str.offset(i as isize) != TAB as u16 || (format & DT_EXPANDTABS) == 0)
            && ((*str.offset(i as isize) != CR as u16 && *str.offset(i as isize) != LF as u16)
                || (format & DT_SINGLELINE) != 0)
        {
            if (format & DT_NOPREFIX) != 0 || *count <= 1 {
                *count -= 1;
                if j < maxl {
                    *dest.offset(j as isize) = *str.offset(i as isize);
                    j += 1;
                    i += 1;
                } else {
                    i += 1;
                }
                continue;
            }

            if *str.offset(i as isize) == PREFIX as u16
                || *str.offset(i as isize) == ALPHA_PREFIX as u16
            {
                *count -= 1;
                i += 1;
                if *str.offset(i as isize) == PREFIX as u16 {
                    *count -= 1;
                    if j < maxl {
                        *dest.offset(j as isize) = *str.offset(i as isize);
                        j += 1;
                        i += 1;
                    } else {
                        i += 1;
                    }
                } else if *pprefix_offset == -1 || *pprefix_offset >= seg_j {
                    *pprefix_offset = j;
                }
            } else if *str.offset(i as isize) == KANA_PREFIX as u16 {
                *count -= 1;
                i += 1;
                *count -= 1;
                i += 1;
            } else {
                *count -= 1;
                if j < maxl {
                    *dest.offset(j as isize) = *str.offset(i as isize);
                    j += 1;
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }

        j_in_seg = (j - seg_j) as u32;
        max_seg_width = width - plen;
        GetTextExtentExPointW(
            hdc,
            dest.offset(seg_j as isize),
            j_in_seg as i32,
            max_seg_width,
            &mut num_fit,
            core::ptr::null_mut(),
            &mut size,
        );

        word_broken = 0;
        line_fits = if num_fit >= j_in_seg as i32 { 1 } else { 0 };
        if line_fits == 0 && (format & DT_WORDBREAK) != 0 {
            let mut s: *const u16 = core::ptr::null();
            let mut chars_used: u32 = 0;
            TEXT_WordBreak(
                hdc,
                dest.offset(seg_j as isize),
                (maxl - seg_j) as u32,
                &mut j_in_seg,
                max_seg_width,
                format,
                num_fit as u32,
                &mut chars_used,
                &mut size,
            );
            line_fits = if size.cx <= max_seg_width { 1 } else { 0 };
            TEXT_SkipChars(
                count,
                &mut s as *mut *const u16,
                seg_count,
                str.offset(seg_i as isize),
                i - seg_i,
                chars_used as i32,
                if (format & DT_NOPREFIX) == 0 { 1 } else { 0 },
            );
            i = s.offset_from(str) as i32;
            word_broken = 1;
        }
        (*pellip).before = j_in_seg as i32;
        (*pellip).under = 0;
        (*pellip).after = 0;
        (*pellip).len = 0;
        ellipsified = 0;
        if line_fits == 0 && (format & DT_PATH_ELLIPSIS) != 0 {
            TEXT_PathEllipsify(
                hdc,
                dest.offset(seg_j as isize),
                (maxl - seg_j) as u32,
                &mut j_in_seg,
                max_seg_width,
                &mut size,
                *p_retstr,
                pellip,
            );
            line_fits = if size.cx <= max_seg_width { 1 } else { 0 };
            ellipsified = 1;
        }
        if (line_fits == 0 && (format & DT_WORD_ELLIPSIS) != 0)
            || ((format & DT_END_ELLIPSIS) != 0
                && ((last_line != 0 && *count != 0)
                    || (remainder_is_none_or_newline(*count, &*str.offset(i as isize)) != 0
                        && line_fits == 0)))
        {
            let mut before: i32 = 0;
            let mut len_ellipsis: i32 = 0;
            TEXT_Ellipsify(
                hdc,
                dest.offset(seg_j as isize),
                (maxl - seg_j) as u32,
                &mut j_in_seg,
                max_seg_width,
                &mut size,
                *p_retstr,
                &mut before,
                &mut len_ellipsis,
            );
            if before > (*pellip).before {
                (*pellip).after = before - (*pellip).before - (*pellip).len;
            } else {
                debug_assert!((*pellip).under == 0 && (*pellip).after == 0);
                (*pellip).before = before;
                (*pellip).len = len_ellipsis;
            }
            line_fits = if size.cx <= max_seg_width { 1 } else { 0 };
            ellipsified = 1;
        }
        if (format & DT_EXPANDTABS) != 0 && ellipsified != 0 {
            if (format & DT_SINGLELINE) != 0 {
                *count = 0;
            } else {
                while *count != 0
                    && *str.offset(i as isize) != CR as u16
                    && *str.offset(i as isize) != LF as u16
                {
                    *count -= 1;
                    i += 1;
                }
            }
        }

        j = seg_j + j_in_seg as i32;
        if *pprefix_offset >= seg_j + (*pellip).before {
            *pprefix_offset = TEXT_Reprefix(str.offset(seg_i as isize), (i - seg_i) as u32, pellip);
            if *pprefix_offset != -1 {
                *pprefix_offset += seg_j;
            }
        }

        plen += size.cx;
        if size.cy > (*retsize).cy {
            (*retsize).cy = size.cy;
        }

        if word_broken != 0 {
            break;
        } else if *count == 0 {
            break;
        } else if *str.offset(i as isize) == CR as u16 || *str.offset(i as isize) == LF as u16 {
            *count -= 1;
            i += 1;
            if *count != 0
                && (*str.offset(i as isize) == CR as u16 || *str.offset(i as isize) == LF as u16)
                && *str.offset(i as isize) != *str.offset((i - 1) as isize)
            {
                *count -= 1;
                i += 1;
            }
            break;
        }
    }

    (*retsize).cx = plen;
    *len = j;
    if *count != 0 {
        &*str.offset(i as isize)
    } else {
        core::ptr::null()
    }
}

unsafe fn TEXT_DrawUnderscore(hdc: HDC, x: i32, y: i32, str: *const u16, offset: i32, rect: *const RECT) {
    let mut prefix_x: i32;
    let mut prefix_end: i32;
    let mut size: SIZE = core::mem::zeroed();
    let hpen: HPEN;
    let oldPen: HGDIOBJ;

    GetTextExtentPointW(hdc, str, offset, &mut size);
    prefix_x = x + size.cx;
    GetTextExtentPointW(hdc, str, offset + 1, &mut size);
    prefix_end = x + size.cx - 1;

    if !rect.is_null() {
        if prefix_x > (*rect).right || prefix_end < (*rect).left ||
           y < (*rect).top || y > (*rect).bottom {
            return;
        }
        if prefix_x < (*rect).left {
            prefix_x = (*rect).left;
        }
        if prefix_end > (*rect).right {
            prefix_end = (*rect).right;
        }
    }

    hpen = CreatePen(PS_SOLID as i32, 1, GetTextColor(hdc));
    oldPen = SelectObject(hdc, hpen.cast());
    MoveToEx(hdc, prefix_x, y, core::ptr::null_mut());
    LineTo(hdc, prefix_end, y);
    SelectObject(hdc, oldPen);
    DeleteObject(hpen.cast());
}

// INT WINAPI DrawTextExWorker — stdcall export (`_DrawTextExWorker@24` on i386);
// signature byte-for-byte matches the forward-decl in font.rs.
#[no_mangle]
pub unsafe extern "system" fn DrawTextExWorker(
    hdc: HDC,
    str: LPWSTR,
    i_count: INT,
    rect: LPRECT,
    flags: UINT,
    dtp: LPDRAWTEXTPARAMS,
) -> INT {
    let mut size: SIZE = core::mem::zeroed();
    let mut strPtr: *const u16;
    let retstr: *mut u16;
    let mut p_retstr: *mut u16;
    let mut size_retstr: usize;
    let mut line: [u16; MAX_BUFFER] = [0; MAX_BUFFER];
    let mut len: i32;
    let lh: i32;
    let mut count: i32 = i_count;
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut lmargin: i32 = 0;
    let mut rmargin: i32 = 0;
    let mut x: i32 = (*rect).left;
    let mut y: i32 = (*rect).top;
    let width: i32 = (*rect).right - (*rect).left;
    let mut max_width: i32 = 0;
    let mut last_line: i32;
    let mut tabwidth: i32 = 0;
    let mut prefix_offset: i32 = 0;
    let mut ellip: ellipsis_data = core::mem::zeroed();
    let mut invert_y: u8 = 0;

    let mut hrgn: HRGN = core::ptr::null_mut();

    let mut flags = flags;

    if str.is_null() {
        return 0;
    }

    strPtr = str;

    if (flags & DT_SINGLELINE) != 0 {
        flags &= !DT_WORDBREAK;
    }

    GetTextMetricsW(hdc, &mut tm);

    if (flags & DT_EXTERNALLEADING) != 0 {
        lh = tm.tmHeight + tm.tmExternalLeading;
    } else {
        lh = tm.tmHeight;
    }

    if *str != 0 && count == 0 {
        return lh;
    }

    if !dtp.is_null() && (*dtp).cbSize != core::mem::size_of::<DRAWTEXTPARAMS>() as u32 {
        return 0;
    }

    if GetGraphicsMode(hdc) == GM_COMPATIBLE as i32 {
        let mut window_ext: SIZE = core::mem::zeroed();
        let mut viewport_ext: SIZE = core::mem::zeroed();
        GetWindowExtEx(hdc, &mut window_ext);
        GetViewportExtEx(hdc, &mut viewport_ext);
        if (window_ext.cy > 0) != (viewport_ext.cy > 0) {
            invert_y = 1;
        }
    }

    if count == -1 {
        count = strlenW(str) as i32;

        if count == 0 {
            if (flags & DT_CALCRECT) != 0 {
                (*rect).right = (*rect).left;
                if (flags & DT_SINGLELINE) != 0 {
                    (*rect).bottom = (*rect).top + (if invert_y != 0 { -lh } else { lh });
                } else {
                    (*rect).bottom = (*rect).top;
                }
            }
            return lh;
        }
    }

    if !dtp.is_null() {
        lmargin = (*dtp).iLeftMargin;
        rmargin = (*dtp).iRightMargin;
        if (flags & (DT_CENTER | DT_RIGHT)) == 0 {
            x += lmargin;
        }
        (*dtp).uiLengthDrawn = 0;
    }

    if (flags & DT_EXPANDTABS) != 0 {
        let tabstop: i32 = if (flags & DT_TABSTOP) != 0 && !dtp.is_null() && (*dtp).iTabLength != 0 {
            (*dtp).iTabLength
        } else {
            8
        };
        tabwidth = tm.tmAveCharWidth * tabstop;
    }

    if (flags & DT_CALCRECT) != 0 {
        flags |= DT_NOCLIP;
    }

    // Usermode clip-region setup (Fix CORE-2201) — public gdi32 exports.
    if (flags & DT_NOCLIP) == 0 {
        let hasClip: i32;
        hrgn = CreateRectRgn(0, 0, 0, 0);
        if !hrgn.is_null() {
            hasClip = GetClipRgn(hdc, hrgn);
            // If the region to be retrieved is NULL, the return value is 0.
            if hasClip != 1 {
                DeleteObject(hrgn.cast());
                hrgn = core::ptr::null_mut();
            }
            IntersectClipRect(hdc, (*rect).left, (*rect).top, (*rect).right, (*rect).bottom);
        }
    }

    if (flags & DT_MODIFYSTRING) != 0 {
        size_retstr = ((count + 4) as usize) * core::mem::size_of::<u16>();
        retstr = HeapAlloc(GetProcessHeap(), 0, size_retstr).cast();
        if retstr.is_null() {
            return 0;
        }
        memcpy(retstr.cast(), str.cast(), size_retstr);
    } else {
        size_retstr = 0;
        retstr = core::ptr::null_mut();
    }
    p_retstr = retstr;

    loop {
        len = (core::mem::size_of_val(&line) / core::mem::size_of::<u16>()) as i32;
        if invert_y != 0 {
            last_line = if (flags & DT_NOCLIP) == 0 && y - (if (flags & DT_EDITCONTROL) != 0 { 2 * lh - 1 } else { lh }) < (*rect).bottom { 1 } else { 0 };
        } else {
            last_line = if (flags & DT_NOCLIP) == 0 && y + (if (flags & DT_EDITCONTROL) != 0 { 2 * lh - 1 } else { lh }) > (*rect).bottom { 1 } else { 0 };
        }
        strPtr = TEXT_NextLineW(
            hdc,
            strPtr,
            &mut count,
            line.as_mut_ptr(),
            &mut len,
            width,
            flags,
            &mut size,
            last_line,
            &mut p_retstr,
            tabwidth,
            &mut prefix_offset,
            &mut ellip,
        );

        if (flags & DT_CENTER) != 0 {
            if (((*rect).right - (*rect).left) < size.cx) && ((flags & DT_CALCRECT) != 0) {
                x = (*rect).left + size.cx;
            } else {
                x = ((*rect).left + (*rect).right - size.cx) / 2;
            }
        } else if (flags & DT_RIGHT) != 0 {
            x = (*rect).right - size.cx;
        }

        if (flags & DT_SINGLELINE) != 0 {
            if (flags & DT_VCENTER) != 0 {
                y = (*rect).top + ((*rect).bottom - (*rect).top + (if invert_y != 0 { size.cy } else { -size.cy })) / 2;
            } else if (flags & DT_BOTTOM) != 0 {
                y = (*rect).bottom + (if invert_y != 0 { size.cy } else { -size.cy });
            }
        }

        if (flags & DT_CALCRECT) == 0 {
            let mut str_inner: *const u16 = line.as_ptr();
            let mut xseg: i32 = x;
            let mut len_inner = len;
            while len_inner != 0 {
                let len_seg: i32;
                let mut size_inner: SIZE = core::mem::zeroed();
                if (flags & DT_EXPANDTABS) != 0 {
                    let mut p: *const u16 = str_inner;
                    while p < str_inner.add(len_inner as usize) && *p != TAB as u16 {
                        p = p.add(1);
                    }
                    len_seg = (p as usize - str_inner as usize) as i32 / 2;
                    if len_seg != len_inner && GetTextExtentPointW(hdc, str_inner, len_seg, &mut size_inner) == 0 {
                        if !retstr.is_null() {
                            HeapFree(GetProcessHeap(), 0, retstr.cast());
                        }
                        return 0;
                    }
                } else {
                    len_seg = len_inner;
                }

                if ExtTextOutW(
                    hdc,
                    xseg,
                    y,
                    (if (flags & DT_NOCLIP) != 0 { 0 } else { ETO_CLIPPED }) | (if (flags & DT_RTLREADING) != 0 { ETO_RTLREADING } else { 0 }),
                    rect,
                    str_inner,
                    len_seg as u32,
                    core::ptr::null(),
                ) == 0
                {
                    if !retstr.is_null() {
                        HeapFree(GetProcessHeap(), 0, retstr.cast());
                    }
                    return 0;
                }

                if prefix_offset != -1 && prefix_offset < len_seg {
                    TEXT_DrawUnderscore(
                        hdc,
                        xseg,
                        y + tm.tmAscent + 1,
                        str_inner,
                        prefix_offset,
                        if (flags & DT_NOCLIP) != 0 { core::ptr::null() } else { rect },
                    );
                }

                len_inner -= len_seg;
                str_inner = str_inner.add(len_seg as usize);
                if len_inner != 0 {
                    debug_assert!((flags & DT_EXPANDTABS) != 0 && *str_inner == TAB as u16);
                    len_inner -= 1;
                    str_inner = str_inner.add(1);
                    xseg += ((size_inner.cx / tabwidth) + 1) * tabwidth;
                    if prefix_offset != -1 {
                        if prefix_offset < len_seg {
                            prefix_offset = -1;
                        } else {
                            prefix_offset -= len_seg;
                        }
                    }
                }
            }
        } else if size.cx > max_width {
            max_width = size.cx;
        }

        y += if invert_y != 0 { -lh } else { lh };
        if !dtp.is_null() {
            (*dtp).uiLengthDrawn += len as u32;
        }

        if strPtr.is_null() || last_line != 0 {
            break;
        }
    }

    // Usermode clip-region teardown. SelectClipRgn + GdiFlush (not NtGdiExtSelectClipRgn)
    // per ReactOS build rules; see CORE-16498.
    if (flags & DT_NOCLIP) == 0 {
        SelectClipRgn(hdc, hrgn);
        GdiFlush();
        if !hrgn.is_null() {
            DeleteObject(hrgn.cast());
        }
    }

    if (flags & DT_CALCRECT) != 0 {
        (*rect).right = (*rect).left + max_width;
        (*rect).bottom = y;
        if !dtp.is_null() {
            (*rect).right += lmargin + rmargin;
        }
    }

    if !retstr.is_null() {
        memcpy(str.cast(), retstr.cast(), size_retstr);
        HeapFree(GetProcessHeap(), 0, retstr.cast());
    }

    y - (*rect).top
}
