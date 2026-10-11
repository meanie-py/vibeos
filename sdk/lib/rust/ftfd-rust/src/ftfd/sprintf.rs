// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_AAB73FE93FCA: [u16; 7] = [0x003Cu16, 0x004E, 0x0055, 0x004C, 0x004C, 0x003E, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{PANSI_STRING, PUNICODE_STRING, va_list, wchar_t};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ZEROPAD: i32 = 1; // pad with zero

pub const SIGN: i32 = 2; // unsigned/signed long

pub const PLUS: i32 = 4; // show plus

pub const SPACE: i32 = 8; // nolint: const-contradiction (sprintf.c-local printf flag: space if plus)

pub const LEFT: i32 = 16; // left justified

pub const SPECIAL: i32 = 32; // 0x

pub const LARGE: i32 = 64; // use 'ABCDEF' instead of 'abcdef'

pub const REMOVEHEX: i32 = 256; // use 256 as remve 0x frim BASE 16

// C bitfield struct: mantissal:32, mantissah:20, exponent:11, sign:1 (8 bytes total).
// Rust has no bitfields, so the two packed 32-bit units are stored as plain u32s:
//   mantissal                  = bits 0..31 of the first unit
//   mantissah_exp_sign (u32)   = bits 0..19 mantissah, bits 20..30 exponent, bit 31 sign
#[repr(C)]
#[derive(Clone, Copy)]
pub struct double_t {
    pub mantissal: u32,
    pub mantissah_exp_sign: u32,
}

pub type Pdouble_t = *mut double_t;

unsafe extern "C" fn _isinf(__x_bits: u64) -> ::core::ffi::c_int {
    // i386 kernel code is soft-float: an f64 cannot cross a function boundary, so the double
    // travels as its bit pattern and is rebuilt here with a transmute, not f64::from_bits,
    // which is itself a call returning f64 (bit-exact; the C passes it by value).
    let mut __x: ::core::ffi::c_double = core::mem::transmute::<u64, f64>(__x_bits);
    let x: *mut double_t = core::ptr::addr_of_mut!(__x).cast::<double_t>();
    return ((*x).exponent() as ::core::ffi::c_int == 0x7ff
        && ((*x).mantissah() as ::core::ffi::c_int == 0
            && (*x).mantissal() == 0)) as ::core::ffi::c_int;
}

unsafe extern "C" fn _isnan(__x_bits: u64) -> i32 {
    // i386 kernel code is soft-float: an f64 cannot cross a function boundary, so the double
    // travels as its bit pattern and is rebuilt here with a transmute, not f64::from_bits,
    // which is itself a call returning f64 (bit-exact; the C passes it by value).
    let mut __x: f64 = core::mem::transmute::<u64, f64>(__x_bits);
    let mut x: *mut double_t = core::ptr::addr_of_mut!(__x) as *mut double_t;
    return ((*x).exponent() as u32 == 0x7ff
        && ((*x).mantissah() as u32 != 0
            || (*x).mantissal() as u32 != 0)) as i32;
}

unsafe extern "C" fn do_div(n: *mut i64, base: i32) -> i32 {
    let mut a: i32;
    a = ((*n as u64) % (base as u32 as u64)) as i32;
    *n = ((*n as u64) / (base as u32 as u64)) as i64;
    return a;
}

unsafe extern "C" fn skip_atoi(s: *mut *const i8) -> i32 {
    let mut i: i32 = 0;
    while isdigit(*(*s) as i32) != 0 {
        let fresh23 = *s;
        *s = (*s).add(1);
        i = i * 10 + *fresh23 as i32 - '0' as i32;
    }
    return i;
}

unsafe extern "C" fn number(mut buf: *mut ::core::ffi::c_char, mut end: *mut ::core::ffi::c_char, mut num: ::core::ffi::c_longlong, mut base: ::core::ffi::c_int, mut size: ::core::ffi::c_int, mut precision: ::core::ffi::c_int, mut type_0: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut c: ::core::ffi::c_char = 0;
    let mut sign: ::core::ffi::c_char = 0;
    let mut tmp: [::core::ffi::c_char; 66] = [0; 66];
    let mut digits: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut small_digits: *const ::core::ffi::c_char =
        b"0123456789abcdefghijklmnopqrstuvwxyz\0" as *const u8 as *const ::core::ffi::c_char;
    let mut large_digits: *const ::core::ffi::c_char =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\0" as *const u8 as *const ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0;
    digits = if type_0 & LARGE != 0 {
        large_digits
    } else {
        small_digits
    };
    if type_0 & LEFT != 0 {
        type_0 &= !ZEROPAD;
    }
    if base < 2 || base > 36 {
        return 0 as *mut ::core::ffi::c_char;
    }
    c = (if type_0 & ZEROPAD != 0 {
        '0' as i32
    } else {
        ' ' as i32
    }) as ::core::ffi::c_char;
    sign = 0 as ::core::ffi::c_char;
    if type_0 & SIGN != 0 {
        if num < 0 as ::core::ffi::c_longlong {
            sign = '-' as i32 as ::core::ffi::c_char;
            num = -num;
            size -= 1;
        } else if type_0 & PLUS != 0 {
            sign = '+' as i32 as ::core::ffi::c_char;
            size -= 1;
        } else if type_0 & SPACE != 0 {
            sign = ' ' as i32 as ::core::ffi::c_char;
            size -= 1;
        }
    }
    if (type_0 & SPECIAL != 0)
        && ((type_0 & REMOVEHEX) == 0)
    {
        if base == 16 {
            size -= 2;
        }
    }
    i = 0;
    if num == 0 as ::core::ffi::c_longlong
        && precision != 0
    {
        let fresh3 = i;
        i = i + 1;
        tmp[fresh3 as usize] = '0' as i32 as ::core::ffi::c_char;
    } else {
        while num != 0 as ::core::ffi::c_longlong {
            let fresh4 = i;
            i = i + 1;
            tmp[fresh4 as usize] = *digits.offset(
                do_div(core::ptr::addr_of_mut!(num), base) as isize,
            );
        }
    }
    if i > precision {
        precision = i;
    }
    size -= precision;
    if type_0 & (ZEROPAD + LEFT) == 0 {
        loop {
            let fresh5 = size;
            size = size - 1;
            if !(fresh5 > 0) {
                break;
            }
            if buf <= end {
                *buf = ' ' as i32 as ::core::ffi::c_char;
            }
            buf = buf.offset(1);
        }
    }
    if sign != 0 {
        if buf <= end {
            *buf = sign;
        }
        buf = buf.offset(1);
    }
    if (type_0 & SPECIAL != 0)
        && ((type_0 & REMOVEHEX) == 0)
    {
        if base == 16 {
            if buf <= end {
                *buf = '0' as i32 as ::core::ffi::c_char;
            }
            buf = buf.offset(1);
            if buf <= end {
                *buf = *digits.offset(33 as isize);
            }
            buf = buf.offset(1);
        }
    }
    if type_0 & LEFT == 0 {
        loop {
            let fresh6 = size;
            size = size - 1;
            if !(fresh6 > 0) {
                break;
            }
            if buf <= end {
                *buf = c;
            }
            buf = buf.offset(1);
        }
    }
    loop {
        let fresh7 = precision;
        precision = precision - 1;
        if !(i < fresh7) {
            break;
        }
        if buf <= end {
            *buf = '0' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    loop {
        let fresh8 = i;
        i = i - 1;
        if !(fresh8 > 0) {
            break;
        }
        if buf <= end {
            *buf = tmp[i as usize];
        }
        buf = buf.offset(1);
    }
    loop {
        let fresh9 = size;
        size = size - 1;
        if !(fresh9 > 0) {
            break;
        }
        if buf <= end {
            *buf = ' ' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    return buf;
}

unsafe extern "C" fn numberf(
    mut buf: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
    num_bits: u64,
    mut base: ::core::ffi::c_int,
    mut size: ::core::ffi::c_int,
    mut precision: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    // i386 kernel code is soft-float: an f64 cannot cross a function boundary, so the double
    // travels as its bit pattern and is rebuilt here with a transmute, not f64::from_bits,
    // which is itself a call returning f64 (bit-exact; the C passes it by value).
    let mut num: f64 = core::mem::transmute::<u64, f64>(num_bits);
    let mut c: ::core::ffi::c_char = 0;
    let mut sign: ::core::ffi::c_char = 0;
    let mut tmp: [::core::ffi::c_char; 66] = [0; 66];
    let mut digits: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut small_digits: *const ::core::ffi::c_char =
        b"0123456789abcdefghijklmnopqrstuvwxyz\0" as *const u8 as *const ::core::ffi::c_char;
    let mut large_digits: *const ::core::ffi::c_char =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\0" as *const u8 as *const ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_longlong = 0;
    digits = if type_0 & LARGE != 0 {
        large_digits
    } else {
        small_digits
    };
    if type_0 & LEFT != 0 {
        type_0 &= !ZEROPAD;
    }
    if base < 2 || base > 36 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    c = (if type_0 & ZEROPAD != 0 {
        '0' as i32
    } else {
        ' ' as i32
    }) as ::core::ffi::c_char;
    sign = 0 as ::core::ffi::c_char;
    if type_0 & SIGN != 0 {
        if num < (0 as f64) {
            sign = '-' as i32 as ::core::ffi::c_char;
            num = -num;
            size -= 1;
        } else if type_0 & PLUS != 0 {
            sign = '+' as i32 as ::core::ffi::c_char;
            size -= 1;
        } else if type_0 & SPACE != 0 {
            sign = ' ' as i32 as ::core::ffi::c_char;
            size -= 1;
        }
    }
    if type_0 & SPECIAL != 0 {
        if base == 16 {
            size -= 2;
        } else if base == 8 {
            size -= 1;
        }
    }
    i = 0;
    if num == (0 as f64) {
        let fresh10 = i;
        i = i + 1;
        tmp[fresh10 as usize] = '0' as i32 as ::core::ffi::c_char;
    } else {
        while num != (0 as f64) {
            x = num as ::core::ffi::c_longlong;
            let fresh11 = i;
            i = i + 1;
            tmp[fresh11 as usize] =
                *digits.offset(do_div(&raw mut x, base) as isize);
            num = x as f64;
        }
    }
    if i > precision {
        precision = i;
    }
    size -= precision;
    if type_0 & (ZEROPAD + LEFT) == 0 {
        loop {
            let fresh12 = size;
            size = size - 1;
            if !(fresh12 > 0) {
                break;
            }
            if buf <= end {
                *buf = ' ' as i32 as ::core::ffi::c_char;
            }
            buf = buf.offset(1);
        }
    }
    if sign != 0 {
        if buf <= end {
            *buf = sign;
        }
        buf = buf.offset(1);
    }
    if type_0 & SPECIAL != 0 {
        if base == 8 {
            if buf <= end {
                *buf = '0' as i32 as ::core::ffi::c_char;
            }
            buf = buf.offset(1);
        } else if base == 16 {
            if buf <= end {
                *buf = '0' as i32 as ::core::ffi::c_char;
            }
            buf = buf.offset(1);
            if buf <= end {
                *buf = *digits.offset(33 as isize);
            }
            buf = buf.offset(1);
        }
    }
    if type_0 & LEFT == 0 {
        loop {
            let fresh13 = size;
            size = size - 1;
            if !(fresh13 > 0) {
                break;
            }
            if buf <= end {
                *buf = c;
            }
            buf = buf.offset(1);
        }
    }
    loop {
        let fresh14 = precision;
        precision = precision - 1;
        if !(i < fresh14) {
            break;
        }
        if buf <= end {
            *buf = '0' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    loop {
        let fresh15 = i;
        i = i - 1;
        if !(fresh15 > 0) {
            break;
        }
        if buf <= end {
            *buf = tmp[i as usize];
        }
        buf = buf.offset(1);
    }
    loop {
        let fresh16 = size;
        size = size - 1;
        if !(fresh16 > 0) {
            break;
        }
        if buf <= end {
            *buf = ' ' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    return buf;
}

unsafe extern "C" fn string(
    mut buf: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut field_width: ::core::ffi::c_int,
    mut precision: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_char = 0;
    c = (if flags & ZEROPAD != 0 {
        '0' as i32
    } else {
        ' ' as i32
    }) as ::core::ffi::c_char;
    if s.is_null() {
        s = b"<NULL>\0" as *const u8 as *const ::core::ffi::c_char;
        len = 6 as ::core::ffi::c_int;
    } else if len == -(1 as ::core::ffi::c_int) {
        len = 0 as ::core::ffi::c_int;
        while (len as ::core::ffi::c_uint) < precision as ::core::ffi::c_uint
            && *s.offset(len as isize) as ::core::ffi::c_int != 0
        {
            len += 1;
        }
    } else if len as ::core::ffi::c_uint > precision as ::core::ffi::c_uint {
        len = precision;
    }
    if flags & LEFT == 0 {
        loop {
            let fresh17 = field_width;
            field_width = field_width - 1;
            if !(len < fresh17) {
                break;
            }
            if buf <= end {
                *buf = c;
            }
            buf = buf.offset(1);
        }
    }
    i = 0 as ::core::ffi::c_int;
    while i < len {
        if buf <= end {
            let fresh18 = s;
            s = s.offset(1);
            *buf = *fresh18;
        }
        buf = buf.offset(1);
        i += 1;
    }
    loop {
        let fresh19 = field_width;
        field_width = field_width - 1;
        if !(len < fresh19) {
            break;
        }
        if buf <= end {
            *buf = ' ' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    return buf;
}

unsafe extern "C" fn stringw(
    mut buf: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
    mut sw: *const wchar_t,
    mut len: ::core::ffi::c_int,
    mut field_width: ::core::ffi::c_int,
    mut precision: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_char = 0;
    c = (if flags & ZEROPAD != 0 {
        '0' as i32
    } else {
        ' ' as i32
    }) as ::core::ffi::c_char;
    if sw.is_null() {
        sw = RUSTCONV_WSTR_AAB73FE93FCA.as_ptr() as *const wchar_t;
        len = 6 as ::core::ffi::c_int;
    } else if len == -(1 as ::core::ffi::c_int) {
        len = 0 as ::core::ffi::c_int;
        while (len as ::core::ffi::c_uint) < precision as ::core::ffi::c_uint
            && *sw.offset(len as isize) as ::core::ffi::c_int != 0
        {
            len += 1;
        }
    } else if len as ::core::ffi::c_uint > precision as ::core::ffi::c_uint {
        len = precision;
    }
    if flags & LEFT == 0 {
        loop {
            let fresh20 = field_width;
            field_width = field_width - 1;
            if !(len < fresh20) {
                break;
            }
            if buf <= end {
                *buf = c;
            }
            buf = buf.offset(1);
        }
    }
    i = 0 as ::core::ffi::c_int;
    while i < len {
        if buf <= end {
            let fresh21 = sw;
            sw = sw.offset(1);
            *buf = *fresh21 as ::core::ffi::c_uchar as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
        i += 1;
    }
    loop {
        let fresh22 = field_width;
        field_width = field_width - 1;
        if !(len < fresh22) {
            break;
        }
        if buf <= end {
            *buf = ' ' as i32 as ::core::ffi::c_char;
        }
        buf = buf.offset(1);
    }
    return buf;
}

// nolint: export-shadow (ftfd imports only win32k; this is sprintf.c's private copy, which
// the statically linked FreeType resolves against, exactly as the C build does)
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf(
    buf: *mut ::core::ffi::c_char,
    mut cnt: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut args: va_list,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    let mut num: ::core::ffi::c_ulonglong = 0;
    let mut _double: ::core::ffi::c_double = 0.0;
    let mut base: ::core::ffi::c_int = 0;
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut sw: *const wchar_t = ::core::ptr::null();
    let mut flags: ::core::ffi::c_int = 0;
    let mut field_width: ::core::ffi::c_int = 0;
    let mut precision: ::core::ffi::c_int = 0;
    let mut qualifier: ::core::ffi::c_int = 0;
    // C's va_arg on `args`: walk this frame's copy of the va_list through the varargs-rust
    // pullers (stable Rust has no va_arg). The parameter stays live as the cursor.
    let mut args = varargs_rust::VaArgs::from_ptr(core::ptr::addr_of_mut!(args) as *mut c_void);

    str = buf;
    end = buf
        .offset(cnt as isize)
        .offset(-(1 as isize));
    if end < buf.offset(-(1 as isize)) {
        end = (-(1 as isize)) as *mut ::core::ffi::c_char;
        cnt = (end.offset_from(buf) as ::core::ffi::c_longlong
            + 1 as ::core::ffi::c_longlong) as size_t;
    }

    while *fmt != 0 {
        if *fmt as ::core::ffi::c_int != '%' as i32 {
            if str <= end {
                *str = *fmt;
            }
            str = str.offset(1);
        } else {
            flags = 0 as ::core::ffi::c_int;
            loop {
                fmt = fmt.offset(1);
                match *fmt as ::core::ffi::c_int {
                    45 => {
                        flags |= LEFT as ::core::ffi::c_int;
                    }
                    43 => {
                        flags |= PLUS as ::core::ffi::c_int;
                    }
                    32 => {
                        flags |= SPACE as ::core::ffi::c_int;
                    }
                    35 => {
                        flags |= SPECIAL as ::core::ffi::c_int;
                    }
                    48 => {
                        flags |= ZEROPAD as ::core::ffi::c_int;
                    }
                    _ => {
                        break;
                    }
                }
            }

            field_width = -(1 as ::core::ffi::c_int);
            if isdigit(*fmt as ::core::ffi::c_int) != 0 {
                field_width = skip_atoi(::core::ptr::addr_of_mut!(fmt));
            } else if *fmt as ::core::ffi::c_int == '*' as i32 {
                fmt = fmt.offset(1);
                field_width = args.arg::<::core::ffi::c_int>();
                if field_width < 0 as ::core::ffi::c_int {
                    field_width = -field_width;
                    flags |= LEFT as ::core::ffi::c_int;
                }
            }

            precision = -(1 as ::core::ffi::c_int);
            if *fmt as ::core::ffi::c_int == '.' as i32 {
                fmt = fmt.offset(1);
                if isdigit(*fmt as ::core::ffi::c_int) != 0 {
                    precision = skip_atoi(::core::ptr::addr_of_mut!(fmt));
                } else if *fmt as ::core::ffi::c_int == '*' as i32 {
                    fmt = fmt.offset(1);
                    precision = args.arg::<::core::ffi::c_int>();
                }
                if precision < 0 as ::core::ffi::c_int {
                    precision = 0 as ::core::ffi::c_int;
                }
            }

            qualifier = -(1 as ::core::ffi::c_int);
            if *fmt as ::core::ffi::c_int == 'h' as i32
                || *fmt as ::core::ffi::c_int == 'l' as i32
                || *fmt as ::core::ffi::c_int == 'L' as i32
                || *fmt as ::core::ffi::c_int == 'w' as i32
            {
                qualifier = *fmt as ::core::ffi::c_int;
                fmt = fmt.offset(1);
            } else if *fmt as ::core::ffi::c_int == 'I' as i32
                && *fmt.offset(1 as isize) as ::core::ffi::c_int == '6' as i32
                && *fmt.offset(2 as isize) as ::core::ffi::c_int == '4' as i32
            {
                qualifier = *fmt as ::core::ffi::c_int;
                fmt = fmt.offset(3 as isize);
            } else if *fmt as ::core::ffi::c_int == 'I' as i32
                && *fmt.offset(1 as isize) as ::core::ffi::c_int == '3' as i32
                && *fmt.offset(2 as isize) as ::core::ffi::c_int == '2' as i32
            {
                qualifier = 'l' as i32;
                fmt = fmt.offset(3 as isize);
            } else if *fmt as ::core::ffi::c_int == 'F' as i32
                && *fmt.offset(1 as isize) as ::core::ffi::c_int == 'p' as i32
            {
                fmt = fmt.offset(1 as isize);
                flags |= REMOVEHEX as ::core::ffi::c_int;
            }

            base = 10 as ::core::ffi::c_int;

            let mut number_path = false;
            match *fmt as ::core::ffi::c_int {
                99 => {
                    /* 'c' */
                    if qualifier == 'l' as i32 || qualifier == 'w' as i32 {
                        let mut sw1: [wchar_t; 2 as usize] =
                            [0; 2 as usize];
                        sw1[0 as usize] =
                            args.arg::<::core::ffi::c_int>() as wchar_t;
                        sw1[1 as usize] = 0 as wchar_t;
                        str = stringw(
                            str,
                            end,
                            ::core::ptr::addr_of_mut!(sw1) as *mut wchar_t,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    } else {
                        let mut s1: [::core::ffi::c_char; 2 as usize] =
                            [0; 2 as usize];
                        s1[0 as usize] = args.arg::<::core::ffi::c_int>()
                            as ::core::ffi::c_uchar
                            as ::core::ffi::c_char;
                        s1[1 as usize] =
                            0 as ::core::ffi::c_char;
                        str = string(
                            str,
                            end,
                            ::core::ptr::addr_of_mut!(s1) as *mut ::core::ffi::c_char,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    }
                }
                67 => {
                    /* 'C' */
                    if (flags & LEFT as ::core::ffi::c_int) == 0 {
                        loop {
                            field_width -= 1;
                            if !(field_width > 0 as ::core::ffi::c_int) {
                                break;
                            }
                            if str <= end {
                                *str = ' ' as i32 as ::core::ffi::c_char;
                            }
                            str = str.offset(1);
                        }
                    }
                    if qualifier == 'h' as i32 {
                        if str <= end {
                            *str = args.arg::<::core::ffi::c_int>() as ::core::ffi::c_uchar
                                as ::core::ffi::c_char;
                        }
                        str = str.offset(1);
                    } else {
                        if str <= end {
                            *str = args.arg::<::core::ffi::c_int>() as wchar_t
                                as ::core::ffi::c_uchar
                                as ::core::ffi::c_char;
                        }
                        str = str.offset(1);
                    }
                    loop {
                        field_width -= 1;
                        if !(field_width > 0 as ::core::ffi::c_int) {
                            break;
                        }
                        if str <= end {
                            *str = ' ' as i32 as ::core::ffi::c_char;
                        }
                        str = str.offset(1);
                    }
                }
                115 => {
                    /* 's' */
                    if qualifier == 'l' as i32 || qualifier == 'w' as i32 {
                        sw = args.arg::<*mut wchar_t>();
                        str = stringw(
                            str,
                            end,
                            sw,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    } else {
                        s = args.arg::<*mut ::core::ffi::c_char>();
                        str = string(
                            str,
                            end,
                            s,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    }
                }
                83 => {
                    /* 'S' */
                    if qualifier == 'h' as i32 {
                        s = args.arg::<*mut ::core::ffi::c_char>();
                        str = string(
                            str,
                            end,
                            s,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    } else {
                        sw = args.arg::<*mut wchar_t>();
                        str = stringw(
                            str,
                            end,
                            sw,
                            -(1 as ::core::ffi::c_int),
                            field_width,
                            precision,
                            flags,
                        );
                    }
                }
                90 => {
                    /* 'Z' */
                    if qualifier == 'w' as i32 {
                        let mut pus: PUNICODE_STRING =
                            args.arg::<PUNICODE_STRING>();
                        if pus.is_null() || (*pus).Buffer.is_null() {
                            sw = ::core::ptr::null::<wchar_t>();
                            len = -(1 as ::core::ffi::c_int);
                        } else {
                            sw = (*pus).Buffer as *const wchar_t;
                            len = ((*pus).Length as usize
                                / ::core::mem::size_of::<WCHAR>())
                                as ::core::ffi::c_int;
                        }
                        str = stringw(
                            str,
                            end,
                            sw,
                            len,
                            field_width,
                            precision,
                            flags,
                        );
                    } else {
                        let mut pus_0: PANSI_STRING =
                            args.arg::<PANSI_STRING>();
                        if pus_0.is_null() || (*pus_0).Buffer.is_null() {
                            s = ::core::ptr::null::<::core::ffi::c_char>();
                            len = -(1 as ::core::ffi::c_int);
                        } else {
                            s = (*pus_0).Buffer as *const ::core::ffi::c_char;
                            len = (*pus_0).Length as ::core::ffi::c_int;
                        }
                        str = string(
                            str,
                            end,
                            s,
                            len,
                            field_width,
                            precision,
                            flags,
                        );
                    }
                }
                112 => {
                    /* 'p' */
                    if (flags & LARGE as ::core::ffi::c_int)
                        == 0 as ::core::ffi::c_int
                    {
                        flags |= LARGE as ::core::ffi::c_int;
                    }
                    if field_width == -(1 as ::core::ffi::c_int) {
                        field_width = (2 as usize
                            * ::core::mem::size_of::<*mut ::core::ffi::c_void>())
                            as ::core::ffi::c_int;
                        flags |= ZEROPAD as ::core::ffi::c_int;
                    }
                    str = number(
                        str,
                        end,
                        (args.arg::<*mut ::core::ffi::c_void>() as ULONG_PTR)
                            as ::core::ffi::c_longlong,
                        16 as ::core::ffi::c_int,
                        field_width,
                        precision,
                        flags,
                    );
                }
                110 => {
                    /* 'n' */
                    if qualifier == 'l' as i32 {
                        let mut ip: *mut ::core::ffi::c_long =
                            args.arg::<*mut ::core::ffi::c_long>();
                        *ip = str.offset_from(buf) as ::core::ffi::c_long;
                    } else {
                        let mut ip_0: *mut ::core::ffi::c_int =
                            args.arg::<*mut ::core::ffi::c_int>();
                        *ip_0 = str.offset_from(buf) as ::core::ffi::c_int;
                    }
                }
                101 | 69 | 102 | 103 | 71 => {
                    /* 'e' 'E' 'f' 'g' 'G' */
                    // The 8-byte va_arg slot read as an integer: the same slot va_arg(double) reads.
                    _double = core::mem::transmute::<u64, f64>(args.arg::<u64>());
                    if _isnan(core::mem::transmute::<f64, u64>(_double)) != 0 {
                        s = b"Nan\0" as *const u8 as *const ::core::ffi::c_char;
                        len = 3 as ::core::ffi::c_int;
                        while len > 0 as ::core::ffi::c_int {
                            if str <= end {
                                let fresh0 = s;
                                s = s.offset(1);
                                *str = *fresh0;
                            }
                            str = str.offset(1);
                            len -= 1;
                        }
                    } else if _isinf(core::mem::transmute::<f64, u64>(_double))
                        < 0 as ::core::ffi::c_int
                    {
                        s = b"-Inf\0" as *const u8 as *const ::core::ffi::c_char;
                        len = 4 as ::core::ffi::c_int;
                        while len > 0 as ::core::ffi::c_int {
                            if str <= end {
                                let fresh1 = s;
                                s = s.offset(1);
                                *str = *fresh1;
                            }
                            str = str.offset(1);
                            len -= 1;
                        }
                    } else if _isinf(core::mem::transmute::<f64, u64>(_double))
                        > 0 as ::core::ffi::c_int
                    {
                        s = b"+Inf\0" as *const u8 as *const ::core::ffi::c_char;
                        len = 4 as ::core::ffi::c_int;
                        while len > 0 as ::core::ffi::c_int {
                            if str <= end {
                                let fresh2 = s;
                                s = s.offset(1);
                                *str = *fresh2;
                            }
                            str = str.offset(1);
                            len -= 1;
                        }
                    } else {
                        if precision == -(1 as ::core::ffi::c_int) {
                            precision = 6 as ::core::ffi::c_int;
                        }
                        str = numberf(
                            str,
                            end,
                            core::mem::transmute::<f64, u64>((_double as ::core::ffi::c_int) as ::core::ffi::c_double),
                            base,
                            field_width,
                            precision,
                            flags,
                        );
                    }
                }
                111 => {
                    /* 'o' */
                    base = 8 as ::core::ffi::c_int;
                    number_path = true;
                }
                98 => {
                    /* 'b' */
                    base = 2 as ::core::ffi::c_int;
                    number_path = true;
                }
                88 => {
                    /* 'X' falls through to 'x' */
                    flags |= LARGE as ::core::ffi::c_int;
                    base = 16 as ::core::ffi::c_int;
                    number_path = true;
                }
                120 => {
                    /* 'x' */
                    base = 16 as ::core::ffi::c_int;
                    number_path = true;
                }
                100 | 105 => {
                    /* 'd' 'i' fall through to 'u' */
                    flags |= SIGN as ::core::ffi::c_int;
                    number_path = true;
                }
                117 => {
                    /* 'u' */
                    number_path = true;
                }
                _ => {
                    if *fmt != 0 {
                        if str <= end {
                            *str = *fmt;
                        }
                        str = str.offset(1);
                    } else {
                        fmt = fmt.offset(-1);
                    }
                }
            }

            if number_path {
                if qualifier == 'I' as i32 {
                    num = args.arg::<::core::ffi::c_ulonglong>();
                } else if qualifier == 'l' as i32 {
                    if (flags & SIGN as ::core::ffi::c_int) != 0 {
                        num = args.arg::<::core::ffi::c_long>() as ::core::ffi::c_ulonglong;
                    } else {
                        num = args.arg::<::core::ffi::c_ulong>() as ::core::ffi::c_ulonglong;
                    }
                } else if qualifier == 'h' as i32 {
                    if (flags & SIGN as ::core::ffi::c_int) != 0 {
                        num = args.arg::<::core::ffi::c_int>() as ::core::ffi::c_ulonglong;
                    } else {
                        num = args.arg::<::core::ffi::c_uint>() as ::core::ffi::c_ulonglong;
                    }
                } else {
                    if (flags & SIGN as ::core::ffi::c_int) != 0 {
                        num = args.arg::<::core::ffi::c_int>() as ::core::ffi::c_ulonglong;
                    } else {
                        num = args.arg::<::core::ffi::c_uint>() as ::core::ffi::c_ulonglong;
                    }
                }
                str = number(
                    str,
                    end,
                    num as ::core::ffi::c_longlong,
                    base,
                    field_width,
                    precision,
                    flags,
                );
            }
        }
        fmt = fmt.offset(1);
    }

    if str <= end {
        *str = '\0' as i32 as ::core::ffi::c_char;
    } else if cnt > 0 as size_t {
        /* don't write out a null byte if the buf size is zero */
        *end = '\0' as i32 as ::core::ffi::c_char;
    }
    return str.offset_from(buf) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn sprintf_v(buf: *mut i8, fmt: *const i8, ap_list: *mut ::core::ffi::c_void) -> i32 {
    // The C trampoline passes &ap (rust_varargs_shim.c); the C body only forwards it.
    _vsnprintf(buf, MAXLONG as usize, fmt, *(ap_list as *mut va_list))
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `sprintf_v` above.
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
}


#[no_mangle]
pub unsafe extern "C" fn _snprintf_v(buf: *mut i8, cnt: usize, fmt: *const i8, ap_list: *mut ::core::ffi::c_void) -> i32 {
    // The C trampoline passes &ap (rust_varargs_shim.c); the C body only forwards it.
    _vsnprintf(buf, cnt, fmt, *(ap_list as *mut va_list))
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `_snprintf_v` above.
    pub fn _snprintf(buf: *mut i8, cnt: usize, fmt: *const i8, ...) -> i32;
}


// nolint: export-shadow (ftfd imports only win32k; this is sprintf.c's private copy, which
// the statically linked FreeType resolves against, exactly as the C build does)
#[no_mangle]
pub unsafe extern "C" fn vsprintf(buf: *mut core::ffi::c_char, fmt: *const core::ffi::c_char, args: va_list) -> i32 {
    return _vsnprintf(buf, MAXLONG as usize, fmt, args);
}
