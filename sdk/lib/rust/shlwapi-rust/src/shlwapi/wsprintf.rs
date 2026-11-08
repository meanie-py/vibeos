use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut null_stringA: [CHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [CHAR; 7]>(*b"(null)\0") };

static mut null_stringW: [WCHAR; 7] = [
    '(' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    ')' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const WPRINTF_LEFTALIGN: UINT = 0x0001;
pub const WPRINTF_PREFIX_HEX: UINT = 0x0002;
pub const WPRINTF_ZEROPAD: UINT = 0x0004;
pub const WPRINTF_LONG: UINT = 0x0008;
pub const WPRINTF_SHORT: UINT = 0x0010;
pub const WPRINTF_UPPER_HEX: UINT = 0x0020;
pub const WPRINTF_WIDE: UINT = 0x0040;
pub const WPRINTF_INTPTR: UINT = 0x0080;
pub const WPRINTF_I64: UINT = 0x0100;

pub type WPRINTF_TYPE = i32;
pub const WPR_UNKNOWN: WPRINTF_TYPE = 0;
pub const WPR_CHAR: WPRINTF_TYPE = 1;
pub const WPR_WCHAR: WPRINTF_TYPE = 2;
pub const WPR_STRING: WPRINTF_TYPE = 3;
pub const WPR_WSTRING: WPRINTF_TYPE = 4;
pub const WPR_SIGNED: WPRINTF_TYPE = 5;
pub const WPR_UNSIGNED: WPRINTF_TYPE = 6;
pub const WPR_HEXA: WPRINTF_TYPE = 7;

#[repr(C)]
pub struct WPRINTF_FORMAT {
    pub flags: UINT,
    pub width: UINT,
    pub precision: UINT,
    pub type_: WPRINTF_TYPE,
}
pub type PWPRINTF_FORMAT = *mut WPRINTF_FORMAT;

#[repr(C)]
pub union WPRINTF_DATA {
    pub wchar_view: WCHAR,
    pub char_view: CHAR,
    pub lpcstr_view: LPCSTR,
    pub lpcwstr_view: LPCWSTR,
    pub int_view: LONGLONG,
}
pub type PWPRINTF_DATA = *mut WPRINTF_DATA;

unsafe extern "C" fn WPRINTF_ParseFormatA(format: LPCSTR, res: *mut WPRINTF_FORMAT) -> INT {
    let mut p: LPCSTR = format;

    (*res).flags = (0) as UINT;
    (*res).width = (0) as UINT;
    (*res).precision = (0) as UINT;
    if *p as ::core::ffi::c_int == '-' as i32 {
        (*res).flags |= (WPRINTF_LEFTALIGN) as UINT;
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == '#' as i32 {
        (*res).flags |= (WPRINTF_PREFIX_HEX) as UINT;
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == '0' as i32 {
        (*res).flags |= (WPRINTF_ZEROPAD) as UINT;
        p = p.offset(1);
    }
    while *p as ::core::ffi::c_int >= '0' as i32 && *p as ::core::ffi::c_int <= '9' as i32 {
        (*res).width = (*res)
            .width
            .wrapping_mul((10) as UINT)
            .wrapping_add(*p as UINT)
            .wrapping_sub('0' as i32 as UINT);
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == '.' as i32 {
        p = p.offset(1);
        while *p as ::core::ffi::c_int >= '0' as i32 && *p as ::core::ffi::c_int <= '9' as i32 {
            (*res).precision = (*res)
                .precision
                .wrapping_mul((10) as UINT)
                .wrapping_add(*p as UINT)
                .wrapping_sub('0' as i32 as UINT);
            p = p.offset(1);
        }
    }
    if *p as ::core::ffi::c_int == 'l' as i32 {
        (*res).flags |= (WPRINTF_LONG) as UINT;
        p = p.offset(1);
    } else if *p as ::core::ffi::c_int == 'h' as i32 {
        (*res).flags |= (WPRINTF_SHORT) as UINT;
        p = p.offset(1);
    } else if *p as ::core::ffi::c_int == 'w' as i32 {
        (*res).flags |= (WPRINTF_WIDE) as UINT;
        p = p.offset(1);
    } else if *p as ::core::ffi::c_int == 'I' as i32 {
        if *p.offset((1) as isize) as ::core::ffi::c_int == '6' as i32
            && *p.offset((2) as isize) as ::core::ffi::c_int == '4' as i32
        {
            (*res).flags |= (WPRINTF_I64) as UINT;
            p = p.offset((3) as isize);
        } else if *p.offset((1) as isize) as ::core::ffi::c_int == '3' as i32
            && *p.offset((2) as isize) as ::core::ffi::c_int == '2' as i32
        {
            p = p.offset((3) as isize);
        } else {
            (*res).flags |= (WPRINTF_INTPTR) as UINT;
            p = p.offset(1);
        }
    }
    match *p as ::core::ffi::c_int {
        99 => {
            (*res).type_ = ((if (*res).flags & ((WPRINTF_LONG) as UINT) != 0 {
                (WPR_WCHAR)
            } else {
                (WPR_CHAR)
            })) as WPRINTF_TYPE;
        }
        67 => {
            (*res).type_ = ((if (*res).flags & ((WPRINTF_SHORT) as UINT) != 0 {
                (WPR_CHAR)
            } else {
                (WPR_WCHAR)
            })) as WPRINTF_TYPE;
        }
        100 | 105 => {
            (*res).type_ = (WPR_SIGNED) as WPRINTF_TYPE;
        }
        115 => {
            (*res).type_ = ((if (*res).flags
                & (((WPRINTF_LONG) | (WPRINTF_WIDE)) as UINT)
                != 0
            {
                (WPR_WSTRING)
            } else {
                (WPR_STRING)
            })) as WPRINTF_TYPE;
        }
        83 => {
            (*res).type_ = ((if (*res).flags
                & (((WPRINTF_SHORT) | (WPRINTF_WIDE)) as UINT)
                != 0
            {
                (WPR_STRING)
            } else {
                (WPR_WSTRING)
            })) as WPRINTF_TYPE;
        }
        117 => {
            (*res).type_ = (WPR_UNSIGNED) as WPRINTF_TYPE;
        }
        112 => {
            (*res).width = ((2) as usize)
                .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_void>() as usize)
                as UINT;
            (*res).flags |= (((WPRINTF_ZEROPAD) | (WPRINTF_INTPTR)) as UINT);
            // fall through to 'X'
            (*res).flags |= (WPRINTF_UPPER_HEX) as UINT;
            // fall through to 'x'
            (*res).type_ = (WPR_HEXA) as WPRINTF_TYPE;
        }
        88 => {
            (*res).flags |= (WPRINTF_UPPER_HEX) as UINT;
            // fall through to 'x'
            (*res).type_ = (WPR_HEXA) as WPRINTF_TYPE;
        }
        120 => {
            (*res).type_ = (WPR_HEXA) as WPRINTF_TYPE;
        }
        _ => {
            (*res).type_ = (WPR_UNKNOWN) as WPRINTF_TYPE;
            p = p.offset(-1);
        }
    }
    return (p.offset_from(format) as INT) + ((1) as INT);
}

unsafe extern "C" fn WPRINTF_ParseFormatW(
    mut format: LPCWSTR,
    mut res: *mut WPRINTF_FORMAT,
) -> INT {
    let mut p: LPCWSTR = format;

    (*res).flags = 0 as UINT;
    (*res).width = 0 as UINT;
    (*res).precision = 0 as UINT;
    if *p as INT == '-' as INT {
        (*res).flags |= WPRINTF_LEFTALIGN as UINT;
        p = p.offset(1);
    }
    if *p as INT == '#' as INT {
        (*res).flags |= WPRINTF_PREFIX_HEX as UINT;
        p = p.offset(1);
    }
    if *p as INT == '0' as INT {
        (*res).flags |= WPRINTF_ZEROPAD as UINT;
        p = p.offset(1);
    }
    while *p as INT >= '0' as INT && *p as INT <= '9' as INT {
        (*res).width = (*res)
            .width
            .wrapping_mul(10 as UINT)
            .wrapping_add(*p as UINT)
            .wrapping_sub('0' as INT as UINT);
        p = p.offset(1);
    }
    if *p as INT == '.' as INT {
        p = p.offset(1);
        while *p as INT >= '0' as INT && *p as INT <= '9' as INT {
            (*res).precision = (*res)
                .precision
                .wrapping_mul(10 as UINT)
                .wrapping_add(*p as UINT)
                .wrapping_sub('0' as INT as UINT);
            p = p.offset(1);
        }
    }
    if *p as INT == 'l' as INT {
        (*res).flags |= WPRINTF_LONG as UINT;
        p = p.offset(1);
    } else if *p as INT == 'h' as INT {
        (*res).flags |= WPRINTF_SHORT as UINT;
        p = p.offset(1);
    } else if *p as INT == 'w' as INT {
        (*res).flags |= WPRINTF_WIDE as UINT;
        p = p.offset(1);
    } else if *p as INT == 'I' as INT {
        if *p.offset(1 as isize) as INT == '6' as INT
            && *p.offset(2 as isize) as INT == '4' as INT
        {
            (*res).flags |= WPRINTF_I64 as UINT;
            p = p.offset(3 as isize);
        } else if *p.offset(1 as isize) as INT == '3' as INT
            && *p.offset(2 as isize) as INT == '2' as INT
        {
            p = p.offset(3 as isize);
        } else {
            (*res).flags |= WPRINTF_INTPTR as UINT;
            p = p.offset(1);
        }
    }
    match *p as INT {
        99 => {
            (*res).type_ = (if ((*res).flags & (WPRINTF_SHORT as UINT)) != 0 {
                WPR_CHAR as INT
            } else {
                WPR_WCHAR as INT
            }) as WPRINTF_TYPE;
        }
        67 => {
            (*res).type_ = (if ((*res).flags & (WPRINTF_LONG as UINT)) != 0 {
                WPR_WCHAR as INT
            } else {
                WPR_CHAR as INT
            }) as WPRINTF_TYPE;
        }
        100 | 105 => {
            (*res).type_ = WPR_SIGNED as WPRINTF_TYPE;
        }
        115 => {
            (*res).type_ = (if ((*res).flags & (WPRINTF_SHORT as UINT)) != 0
                && ((*res).flags & (WPRINTF_WIDE as UINT)) == 0
            {
                WPR_STRING as INT
            } else {
                WPR_WSTRING as INT
            }) as WPRINTF_TYPE;
        }
        83 => {
            (*res).type_ = (if ((*res).flags & ((WPRINTF_LONG | WPRINTF_WIDE) as UINT)) != 0 {
                WPR_WSTRING as INT
            } else {
                WPR_STRING as INT
            }) as WPRINTF_TYPE;
        }
        117 => {
            (*res).type_ = WPR_UNSIGNED as WPRINTF_TYPE;
        }
        112 => {
            (*res).width = (2 as usize)
                .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_void>() as usize)
                as UINT;
            (*res).flags |= (WPRINTF_ZEROPAD | WPRINTF_INTPTR) as UINT;
            // fall through
            (*res).flags |= WPRINTF_UPPER_HEX as UINT;
            (*res).type_ = WPR_HEXA as WPRINTF_TYPE;
        }
        88 => {
            // fall through
            (*res).flags |= WPRINTF_UPPER_HEX as UINT;
            (*res).type_ = WPR_HEXA as WPRINTF_TYPE;
        }
        120 => {
            (*res).type_ = WPR_HEXA as WPRINTF_TYPE;
        }
        _ => {
            (*res).type_ = WPR_UNKNOWN as WPRINTF_TYPE;
            p = p.offset(-1);
        }
    }
    return p.offset_from(format) as INT + 1 as INT;
}

unsafe extern "C" fn WPRINTF_GetLen(
    mut format: *mut WPRINTF_FORMAT,
    mut arg: *mut WPRINTF_DATA,
    mut number: LPSTR,
    mut maxlen: UINT,
) -> UINT {
    let mut len: UINT = 0;
    if (*format).flags & WPRINTF_LEFTALIGN as UINT != 0 {
        (*format).flags &= !WPRINTF_ZEROPAD as UINT;
    }
    if (*format).width > maxlen {
        (*format).width = maxlen;
    }
    match ((*format).type_ as ::core::ffi::c_uint) as i32 {
        WPR_CHAR | WPR_WCHAR => {
            (*format).precision = 1 as UINT;
            return (*format).precision;
        }
        WPR_STRING => {
            if (*arg).lpcstr_view.is_null() {
                (*arg).lpcstr_view = &raw const null_stringA as *const CHAR as LPCSTR;
            }
            len = 0 as UINT;
            while (*format).precision == 0 || len < (*format).precision {
                if *(*arg).lpcstr_view.offset(len as isize) == 0 {
                    break;
                }
                len = len.wrapping_add(1);
            }
            if len > maxlen {
                len = maxlen;
            }
            (*format).precision = len;
            return (*format).precision;
        }
        WPR_WSTRING => {
            if (*arg).lpcwstr_view.is_null() {
                (*arg).lpcwstr_view = &raw const null_stringW as *const WCHAR as LPCWSTR;
            }
            len = 0 as UINT;
            while (*format).precision == 0 || len < (*format).precision {
                if *(*arg).lpcwstr_view.offset(len as isize) == 0 {
                    break;
                }
                len = len.wrapping_add(1);
            }
            if len > maxlen {
                len = maxlen;
            }
            (*format).precision = len;
            return (*format).precision;
        }
        WPR_SIGNED | WPR_UNSIGNED | WPR_HEXA => {
            let mut digits: *const ::core::ffi::c_char =
                if (*format).flags & WPRINTF_UPPER_HEX as UINT != 0 {
                    b"0123456789ABCDEF\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"0123456789abcdef\0" as *const u8 as *const ::core::ffi::c_char
                };
            let mut num: ULONGLONG = (*arg).int_view as ULONGLONG;
            let mut base: ::core::ffi::c_int = if (*format).type_ as ::core::ffi::c_uint
                == WPR_HEXA as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                16 as ::core::ffi::c_int
            } else {
                10 as ::core::ffi::c_int
            };
            let mut buffer: [::core::ffi::c_char; 20] =
                [0; 20];
            let mut p: *mut ::core::ffi::c_char = &raw mut buffer as *mut ::core::ffi::c_char;
            let mut dst: *mut ::core::ffi::c_char = number as *mut ::core::ffi::c_char;
            if (*format).type_ as ::core::ffi::c_uint
                == WPR_SIGNED as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*arg).int_view < 0 as LONGLONG
            {
                let fresh29 = dst;
                dst = dst.offset(1);
                *fresh29 = '-' as i32 as ::core::ffi::c_char;
                num = -(*arg).int_view as ULONGLONG;
            }
            if (*format).flags & WPRINTF_INTPTR as UINT != 0 {
                num = num as UINT_PTR as ULONGLONG;
            } else if (*format).flags & WPRINTF_I64 as UINT == 0 {
                num = num as UINT as ULONGLONG;
            }
            loop {
                let fresh30 = p;
                p = p.offset(1);
                *fresh30 = *digits.offset(num.wrapping_rem(base as ULONGLONG) as isize);
                num = num.wrapping_div(base as ULONGLONG);
                if !(num != 0) {
                    break;
                }
            }
            while p > &raw mut buffer as *mut ::core::ffi::c_char {
                p = p.offset(-1);
                let fresh31 = dst;
                dst = dst.offset(1);
                *fresh31 = *p;
            }
            *dst = 0 as ::core::ffi::c_char;
            len = dst.offset_from(number) as ::core::ffi::c_longlong as UINT;
        }
        _ => return 0 as UINT,
    }
    if len > maxlen {
        len = maxlen;
    }
    if (*format).precision < len {
        (*format).precision = len;
    }
    if (*format).precision > maxlen {
        (*format).precision = maxlen;
    }
    if (*format).flags & WPRINTF_ZEROPAD as UINT != 0 && (*format).width > (*format).precision {
        (*format).precision = (*format).width;
    }
    if (*format).flags & WPRINTF_PREFIX_HEX as UINT != 0 {
        len = len.wrapping_add(2 as UINT);
    }
    return len;
}

#[no_mangle]
pub unsafe extern "system" fn wvnsprintfA(
    mut buffer: LPSTR,
    mut maxlen: INT,
    mut spec: LPCSTR,
    args_list: *mut ::core::ffi::c_void,
) -> INT {
    // C passes `va_list` by value (a `char *` on all three Windows targets); VaArgs reads
    // through a pointer to this frame's copy, as `va_arg` does. The bought A body bumped the
    // pointer by `size_of::<usize>()` per pull, which misreads every argument after a
    // `%I64` on i386.
    let mut __va = args_list;
    let mut args = varargs_rust::VaArgs::from_ptr(core::ptr::addr_of_mut!(__va) as *mut ::core::ffi::c_void);
    let mut format: WPRINTF_FORMAT = WPRINTF_FORMAT {
        flags: 0,
        width: 0,
        precision: 0,
        type_: WPR_UNKNOWN,
    };
    let mut p: LPSTR = buffer;
    let mut i: UINT = 0;
    let mut len: UINT = 0;
    let mut sign: UINT = 0;
    let mut number: [CHAR; 21] = [0; 21];
    let mut argData: WPRINTF_DATA = WPRINTF_DATA { wchar_view: 0 };
    WINE_TRACE!("%p %u %s\n", buffer, maxlen, debugstr_a(spec as *const ::core::ffi::c_char));
    while *spec as ::core::ffi::c_int != 0 && maxlen > 1 { // nolint: dowhile-continue (C is the same `while`: its `case WPR_UNKNOWN: continue;` skips `maxlen -= len` too)
        if *spec as ::core::ffi::c_int != '%' as i32 {
            *p = *spec;
            p = p.offset(1);
            spec = spec.offset(1);
            maxlen -= 1;
            continue;
        }
        spec = spec.offset(1);
        if *spec as ::core::ffi::c_int == '%' as i32 {
            *p = *spec;
            p = p.offset(1);
            spec = spec.offset(1);
            maxlen -= 1;
            continue;
        }
        spec = spec.offset(
            WPRINTF_ParseFormatA(spec, core::ptr::addr_of_mut!(format)) as isize,
        );
        match (format.type_ as ::core::ffi::c_uint) as i32 {
            WPR_WCHAR => {
                argData.wchar_view = args.arg::<::core::ffi::c_int>() as WCHAR;
            }
            WPR_CHAR => {
                argData.char_view = args.arg::<::core::ffi::c_int>() as CHAR;
            }
            WPR_STRING => {
                argData.lpcstr_view = args.arg::<LPCSTR>();
            }
            WPR_WSTRING => {
                argData.lpcwstr_view = args.arg::<LPCWSTR>();
            }
            WPR_HEXA | WPR_SIGNED | WPR_UNSIGNED => {
                if (format.flags & (WPRINTF_INTPTR as UINT)) != 0 {
                    argData.int_view = args.arg::<INT_PTR>() as LONGLONG;
                } else if (format.flags & (WPRINTF_I64 as UINT)) != 0 {
                    argData.int_view = args.arg::<LONGLONG>();
                } else {
                    argData.int_view = args.arg::<INT>() as LONGLONG;
                }
            }
            _ => {
                argData.wchar_view = 0 as WCHAR;
            }
        }
        len = WPRINTF_GetLen(
            core::ptr::addr_of_mut!(format),
            core::ptr::addr_of_mut!(argData),
            core::ptr::addr_of_mut!(number) as LPSTR,
            (maxlen - 1) as UINT,
        );
        sign = 0 as UINT;
        if (format.flags & (WPRINTF_LEFTALIGN as UINT)) == 0 {
            i = format.precision;
            while i < format.width {
                *p = ' ' as i32 as CHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
        }
        // switch(format.type) with fall-through: stage 0 = switch body,
        // 1 = sign transfer (fall-through point), 2 = digits, 3 = done
        let mut stage: u8 = 0;
        match (format.type_ as ::core::ffi::c_uint) as i32 {
            WPR_WCHAR => {
                *p = argData.wchar_view as CHAR;
                p = p.offset(1);
                stage = 3;
            }
            WPR_CHAR => {
                *p = argData.char_view;
                p = p.offset(1);
                stage = 3;
            }
            WPR_STRING => {
                memcpy(p as *mut ::core::ffi::c_void, argData.lpcstr_view as *const ::core::ffi::c_void, len as size_t);
                p = p.offset(len as isize);
                stage = 3;
            }
            WPR_WSTRING => {
                let mut ptr: LPCWSTR = argData.lpcwstr_view;
                i = 0 as UINT;
                while i < len {
                    *p = *ptr as CHAR;
                    p = p.offset(1);
                    ptr = ptr.offset(1);
                    i = i.wrapping_add(1);
                }
                stage = 3;
            }
            WPR_HEXA => {
                if (format.flags & (WPRINTF_PREFIX_HEX as UINT)) != 0
                    && maxlen > 3
                {
                    *p = '0' as i32 as CHAR;
                    p = p.offset(1);
                    *p = (if (format.flags & (WPRINTF_UPPER_HEX as UINT)) != 0 {
                        'X' as i32
                    } else {
                        'x' as i32
                    }) as CHAR;
                    p = p.offset(1);
                    maxlen -= 2;
                    len = len.wrapping_sub(2 as UINT);
                }
                /* fall through */
                stage = 1;
            }
            WPR_SIGNED => {
                /* fall through */
                stage = 1;
            }
            WPR_UNSIGNED => {
                stage = 2;
            }
            WPR_UNKNOWN => {
                continue;
            }
            _ => {
                stage = 3;
            }
        }
        if stage == 1 {
            /* Transfer the sign now, just in case it will be zero-padded*/
            if number[0 as usize] as ::core::ffi::c_int == '-' as i32 {
                *p = '-' as i32 as CHAR;
                p = p.offset(1);
                sign = 1 as UINT;
            }
            stage = 2;
        }
        if stage == 2 {
            i = len;
            while i < format.precision {
                *p = '0' as i32 as CHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
            memcpy(
                p as *mut ::core::ffi::c_void,
                (core::ptr::addr_of_mut!(number) as *mut CHAR).offset(sign as isize)
                    as *const ::core::ffi::c_void,
                len.wrapping_sub(sign) as size_t,
            );
            p = p.offset(len.wrapping_sub(sign) as isize);
        }
        if (format.flags & (WPRINTF_LEFTALIGN as UINT)) != 0 {
            i = format.precision;
            while i < format.width {
                *p = ' ' as i32 as CHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
        }
        maxlen = (maxlen as UINT).wrapping_sub(len) as INT;
    }
    *p = 0 as CHAR;
    WINE_TRACE!("%s\n", debugstr_a(buffer as *const ::core::ffi::c_char));
    return if maxlen > 1 {
        p.offset_from(buffer) as ::core::ffi::c_longlong as INT
    } else {
        -(1 as INT)
    };
}

#[no_mangle]
pub unsafe extern "system" fn wvnsprintfW(
    mut buffer: LPWSTR,
    mut maxlen: INT,
    mut spec: LPCWSTR,
    args_list: *mut ::core::ffi::c_void,
) -> INT {
    // C passes `va_list` by value (a `char *` on all three Windows targets); VaArgs reads
    // through a pointer to this frame's copy, as `va_arg` does. The bought A body bumped the
    // pointer by `size_of::<usize>()` per pull, which misreads every argument after a
    // `%I64` on i386.
    let mut __va = args_list;
    let mut args = varargs_rust::VaArgs::from_ptr(core::ptr::addr_of_mut!(__va) as *mut ::core::ffi::c_void);
    let mut format: WPRINTF_FORMAT = WPRINTF_FORMAT {
        flags: 0,
        width: 0,
        precision: 0,
        type_: WPR_UNKNOWN,
    };
    let mut p: LPWSTR = buffer;
    let mut i: UINT = 0;
    let mut len: UINT = 0;
    let mut sign: UINT = 0;
    let mut number: [CHAR; 21] = [0; 21];
    let mut argData: WPRINTF_DATA = WPRINTF_DATA { wchar_view: 0 };
    WINE_TRACE!("%p %u %s\n", buffer, maxlen, debugstr_w(spec));

    while *spec != 0 && maxlen > 1 { // nolint: dowhile-continue (C is the same `while`: its `case WPR_UNKNOWN: continue;` skips `maxlen -= len` too)
        if *spec as ::core::ffi::c_int != '%' as i32 {
            let fresh32 = spec;
            spec = spec.offset(1);
            *p = *fresh32;
            p = p.offset(1);
            maxlen -= 1;
            continue;
        }
        spec = spec.offset(1);
        if *spec as ::core::ffi::c_int == '%' as i32 {
            *p = *spec;
            p = p.offset(1);
            spec = spec.offset(1);
            maxlen -= 1;
            continue;
        }
        spec = spec.offset(WPRINTF_ParseFormatW(spec, core::ptr::addr_of_mut!(format)) as isize);

        match (format.type_ as UINT) as i32 {
            WPR_WCHAR => {
                argData.wchar_view = args.arg::<::core::ffi::c_int>() as WCHAR;
            }
            WPR_CHAR => {
                argData.char_view = args.arg::<::core::ffi::c_int>() as CHAR;
            }
            WPR_STRING => {
                argData.lpcstr_view = args.arg::<LPCSTR>();
            }
            WPR_WSTRING => {
                argData.lpcwstr_view = args.arg::<LPCWSTR>();
            }
            WPR_HEXA | WPR_SIGNED | WPR_UNSIGNED => {
                if format.flags & (WPRINTF_INTPTR as UINT) != 0 {
                    argData.int_view = args.arg::<INT_PTR>() as LONGLONG;
                } else if format.flags & (WPRINTF_I64 as UINT) != 0 {
                    argData.int_view = args.arg::<LONGLONG>();
                } else {
                    argData.int_view = args.arg::<INT>() as LONGLONG;
                }
            }
            _ => {
                argData.wchar_view = 0 as WCHAR;
            }
        }

        len = WPRINTF_GetLen(
            core::ptr::addr_of_mut!(format),
            core::ptr::addr_of_mut!(argData),
            core::ptr::addr_of_mut!(number) as LPSTR,
            (maxlen - 1) as UINT,
        );
        sign = 0 as UINT;
        if format.flags & (WPRINTF_LEFTALIGN as UINT) == 0 {
            i = format.precision;
            while i < format.width {
                *p = ' ' as i32 as WCHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
        }

        // Second switch: explicit fallthrough state.
        // fall: 0 = none, 1 = sign block pending, 2 = digits block pending
        let mut fall: u8 = 0;
        match (format.type_ as UINT) as i32 {
            WPR_WCHAR => {
                *p = argData.wchar_view;
                p = p.offset(1);
            }
            WPR_CHAR => {
                *p = argData.char_view as WCHAR;
                p = p.offset(1);
            }
            WPR_STRING => {
                let mut ptr: LPCSTR = argData.lpcstr_view;
                i = 0 as UINT;
                while i < len {
                    *p = *ptr as BYTE as WCHAR;
                    p = p.offset(1);
                    ptr = ptr.offset(1);
                    i = i.wrapping_add(1);
                }
            }
            WPR_WSTRING => {
                if len != 0 {
                    memcpy(
                        p as *mut ::core::ffi::c_void,
                        argData.lpcwstr_view as *const ::core::ffi::c_void,
                        (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
                    );
                }
                p = p.offset(len as isize);
            }
            WPR_HEXA => {
                if (format.flags & (WPRINTF_PREFIX_HEX as UINT)) != 0
                    && maxlen > 3
                {
                    *p = '0' as i32 as WCHAR;
                    p = p.offset(1);
                    *p = (if format.flags & (WPRINTF_UPPER_HEX as UINT) != 0 {
                        'X' as i32
                    } else {
                        'x' as i32
                    }) as WCHAR;
                    p = p.offset(1);
                    maxlen -= 2;
                    len = len.wrapping_sub(2 as UINT);
                }
                fall = 1;
            }
            WPR_SIGNED => {
                fall = 1;
            }
            WPR_UNSIGNED => {
                fall = 2;
            }
            WPR_UNKNOWN => {
                continue;
            }
            _ => {}
        }
        if fall == 1 {
            if number[0 as usize] as ::core::ffi::c_int == '-' as i32 {
                *p = '-' as i32 as WCHAR;
                p = p.offset(1);
                sign = 1 as UINT;
            }
            fall = 2;
        }
        if fall == 2 {
            i = len;
            while i < format.precision {
                *p = '0' as i32 as WCHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
            i = sign;
            while i < len {
                *p = number[i as usize] as BYTE as WCHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
            }
        }

        if format.flags & (WPRINTF_LEFTALIGN as UINT) != 0 {
            i = format.precision;
            while i < format.width {
                *p = ' ' as i32 as WCHAR;
                p = p.offset(1);
                i = i.wrapping_add(1);
                maxlen -= 1;
            }
        }
        maxlen = (maxlen as UINT).wrapping_sub(len) as INT;
    }
    *p = 0 as WCHAR;
    WINE_TRACE!("%s\n", debugstr_w(buffer as *const WCHAR));
    return if maxlen > 1 {
        p.offset_from(buffer) as INT
    } else {
        -(1 as INT)
    };
}

#[no_mangle]
pub unsafe extern "C" fn wnsprintfA_v(lpOut: LPSTR, cchLimitIn: INT, lpFmt: LPCSTR, ap_list: *mut ::core::ffi::c_void) -> INT {
    // The shim already va_start'ed; hand on the started list by value, as wvnsprintfA takes it.
    wvnsprintfA(lpOut, cchLimitIn, lpFmt, *(ap_list as *mut *mut ::core::ffi::c_void))
}



#[no_mangle]
pub unsafe extern "C" fn wnsprintfW_v(lpOut: LPWSTR, cchLimitIn: INT, lpFmt: LPCWSTR, ap_list: *mut ::core::ffi::c_void) -> INT {
    wvnsprintfW(lpOut, cchLimitIn, lpFmt, *(ap_list as *mut *mut ::core::ffi::c_void))
}
