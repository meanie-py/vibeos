use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope types, constants, or tables in this preamble. The two declarations are forward prototypes for static functions, which Rust doesn't need. `validate_rad2angle` and `validate_angle2rad` should be defined as `unsafe extern "C" fn` items in their own function passes. `calc_number_t` is referenced but not defined here, so it is expected to come from the caller's type collection.
#[no_mangle]
pub unsafe extern "C" fn apply_int_mask(mut r: *mut calc_number_t) {
    let mut mask: u64;
    match calc.size {
        IDC_RADIO_QWORD => {
            mask = _UI64_MAX as u64;
        }
        IDC_RADIO_DWORD => {
            mask = ULONG_MAX as u64;
        }
        IDC_RADIO_WORD => {
            mask = USHRT_MAX as u64;
        }
        IDC_RADIO_BYTE => {
            mask = UCHAR_MAX as u64;
        }
        _ => {
            mask = (-(1 as i64)) as u64;
        }
    }
    (*r).i = (((*r).i as u64) & mask) as INT64;
}
#[no_mangle]
pub unsafe extern "C" fn asinh(mut x: ::core::ffi::c_double) -> ::core::ffi::c_double {
    return log(x + sqrt(x * x + 1.0));
}
#[no_mangle]
pub unsafe extern "C" fn acosh(x: f64) -> f64 {
    // must be x>=1, if not return Nan (Not a Number)
    if !(x >= 1.0) {
        return sqrt(-1.0);
    }

    // return only the positive result (as sqrt does).
    return log(x + sqrt(x * x - 1.0));
}

#[no_mangle]
pub unsafe extern "C" fn atanh(mut x: f64) -> f64 {
    // must be x>-1, x<1, if not return Nan (Not a Number)
    if !(x > -1.0 && x < 1.0) {
        return sqrt(-1.0);
    }

    return log((1.0 + x) / (1.0 - x)) / 2.0;
}

unsafe extern "C" fn validate_rad2angle(mut a: f64) -> f64 {
    match calc.degr {
        IDC_RADIO_DEG => {
            a = a * (180.0 / CALC_PI);
        }
        IDC_RADIO_RAD => {}
        IDC_RADIO_GRAD => {
            a = a * (200.0 / CALC_PI);
        }
        _ => {}
    }
    return a;
}

unsafe extern "C" fn validate_angle2rad(c: *mut calc_number_t) -> f64 {
    match calc.degr {
        IDC_RADIO_DEG => {
            (*c).f = (*c).f * (CALC_PI / 180.0);
        }
        IDC_RADIO_RAD => {}
        IDC_RADIO_GRAD => {
            (*c).f = (*c).f * (CALC_PI / 200.0);
        }
        _ => {}
    }
    return (*c).f;
}

#[no_mangle]
pub unsafe extern "C" fn rpn_sin(c: *mut calc_number_t) {
    let angle: f64 = validate_angle2rad(c);

    if angle == 0.0 || angle == CALC_PI {
        (*c).f = 0.0;
    } else if angle == CALC_3_PI_2 {
        (*c).f = -1.0;
    } else if angle == CALC_2_PI {
        (*c).f = 1.0;
    } else {
        (*c).f = sin(angle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_cos(c: *mut calc_number_t) {
    let angle: f64 = validate_angle2rad(c);

    if angle == CALC_PI_2 || angle == CALC_3_PI_2 {
        (*c).f = 0.0;
    } else if angle == CALC_PI {
        (*c).f = -1.0;
    } else if angle == CALC_2_PI {
        (*c).f = 1.0;
    } else {
        (*c).f = cos(angle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_tan(c: *mut calc_number_t) {
    let mut angle: ::core::ffi::c_double = validate_angle2rad(c);

    if angle == CALC_PI_2 || angle == CALC_3_PI_2 {
        calc.is_nan = TRUE as BOOL;
    } else if angle == CALC_PI || angle == CALC_2_PI {
        (*c).f = 0.0;
    } else {
        (*c).f = tan(angle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_asin(c: *mut calc_number_t) {
    (*c).f = validate_rad2angle(asin((*c).f));
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_acos(c: *mut calc_number_t) {
    (*c).f = validate_rad2angle(acos((*c).f));
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_atan(c: *mut calc_number_t) {
    (*c).f = validate_rad2angle(atan((*c).f));
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_sinh(c: *mut calc_number_t) {
    (*c).f = sinh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_cosh(c: *mut calc_number_t) {
    (*c).f = cosh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_tanh(c: *mut calc_number_t) {
    (*c).f = tanh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_asinh(c: *mut calc_number_t) {
    (*c).f = asinh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_acosh(c: *mut calc_number_t) {
    (*c).f = acosh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_atanh(c: *mut calc_number_t) {
    (*c).f = atanh((*c).f);
    if _isnan((*c).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_int(c: *mut calc_number_t) {
    let mut int_part: f64 = 0.0;
    modf(calc.code.f, core::ptr::addr_of_mut!(int_part));
    (*c).f = int_part;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_frac(c: *mut calc_number_t) {
    let mut int_part: f64 = 0.0;
    (*c).f = modf(calc.code.f, core::ptr::addr_of_mut!(int_part));
}
#[no_mangle]
pub unsafe extern "C" fn rpn_reci(c: *mut calc_number_t) {
    if (*c).f == 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*c).f = 1.0 / (*c).f;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_fact(c: *mut calc_number_t) {
    let mut fact: f64;
    let mut mult: f64;
    let mut num: f64;

    if calc.base == IDC_RADIO_DEC {
        num = (*c).f;
    } else {
        num = (*c).i as f64;
    }
    if num > 1000.0 {
        calc.is_nan = (TRUE) as i32;
        return;
    }
    if num < 0.0 {
        calc.is_nan = (TRUE) as i32;
        return;
    } else if num == 0.0 {
        fact = 1.0;
    } else {
        rpn_int(c);
        fact = 1.0;
        mult = 2.0;
        while mult <= num {
            fact *= mult;
            mult += 1.0;
        }
        (*c).f = fact;
    }
    if _finite(fact) == 0 {
        calc.is_nan = (TRUE) as i32;
    } else if calc.base == IDC_RADIO_DEC {
        (*c).f = fact;
    } else {
        (*c).i = fact as i64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn logic_dbl2int(a: *mut calc_number_t) -> ::core::ffi::c_longlong {
    let mut int_part: f64 = 0.0;
    let mut width: i32 = 0;

    modf((*a).f, core::ptr::addr_of_mut!(int_part));
    width = if int_part == (0 as f64) {
        1
    } else {
        (log10(fabs(int_part)) as i32) + 1
    };
    if width > 63 {
        calc.is_nan = (TRUE) as i32;
        return 0;
    }
    return int_part as i64;
}
#[no_mangle]
pub unsafe extern "C" fn logic_int2dbl(a: *mut calc_number_t) -> ::core::ffi::c_double {
    return (*a).i as f64;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_not(mut c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        let mut n: calc_number_t = calc_number_t { f: 0. };
        n.i = logic_dbl2int(c) as INT64;
        (*c).f = ((!n.i) as f64);
    } else {
        (*c).i = !(*c).i;
    }
}

#[no_mangle]
pub unsafe extern "C" fn rpn_pi(c: *mut calc_number_t) {
    (*c).f = CALC_PI;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_2pi(c: *mut calc_number_t) {
    (*c).f = CALC_PI * 2.0;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_sign(c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        (*c).f = 0 as ::core::ffi::c_double - (*c).f;
    } else {
        (*c).i = 0 as INT64 - (*c).i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_exp2(c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        (*c).f *= (*c).f;
        if _finite((*c).f) == 0 as ::core::ffi::c_int {
            calc.is_nan = TRUE as BOOL;
        }
    } else {
        (*c).i *= (*c).i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_exp3(c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        (*c).f = pow((*c).f, 3.0);
        if _finite((*c).f) == 0 {
            calc.is_nan = TRUE as BOOL;
        }
    } else {
        (*c).i *= (*c).i * (*c).i;
    }
}

unsafe extern "C" fn myabs64(mut number: i64) -> i64 {
    return if number < 0 {
        0 - number
    } else {
        number
    };
}

unsafe extern "C" fn sqrti(mut number: u64) -> u64 {
    let mut n: u64 = 0;
    let mut n1: u64 = 0;
    if number == 0xffffffffffffffff {
        return 0xffffffff;
    }
    n = 1;
    n1 = NEXT!(n, number);
    while myabs64(n1.wrapping_sub(n) as i64) > 1 {
        n = n1;
        n1 = NEXT!(n, number);
    }
    while n1.wrapping_mul(n1) > number {
        n1 = n1.wrapping_sub(1);
    }
    return n1;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_sqrt(c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        if (*c).f < 0 as f64 {
            calc.is_nan = TRUE as BOOL;
        } else {
            (*c).f = sqrt((*c).f);
        }
    } else {
        (*c).i = sqrti((*c).i as u64) as INT64;
    }
}

unsafe extern "C" fn cbrti(mut x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong {
    let mut s: ::core::ffi::c_longlong;
    let mut y: ::core::ffi::c_longlong;
    let mut b: ::core::ffi::c_longlong;

    s = 60 as ::core::ffi::c_longlong;
    y = 0 as ::core::ffi::c_longlong;
    while s >= 0 as ::core::ffi::c_longlong {
        y = 2 as ::core::ffi::c_longlong * y;
        b = (3 as ::core::ffi::c_longlong
            * y
            * (y + 1 as ::core::ffi::c_longlong)
            + 1 as ::core::ffi::c_longlong)
            << s;
        s = s - 3 as ::core::ffi::c_longlong;
        if x >= b {
            x = x - b;
            y = y + 1 as ::core::ffi::c_longlong;
        }
    }
    return y;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_cbrt(mut c: *mut calc_number_t) {
    if calc.base == IDC_RADIO_DEC as DWORD {
        // C: #if defined(__GNUC__) && !defined(__REACTOS__) -> cbrt; #else -> pow(x, 1./3.)
        // cfg!(any()) is always false, so the #else (pow) path is taken, matching the anchor.
        (*c).f = if cfg!(any()) {
            cbrt((*c).f)
        } else {
            pow(
                (*c).f,
                1 as f64 / 3 as f64,
            )
        };
    } else {
        (*c).i = cbrti((*c).i as i64) as INT64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_exp(mut c: *mut calc_number_t) {
    (*c).f = exp((*c).f);
    if _finite((*c).f) == 0 {
        calc.is_nan = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_exp10(c: *mut calc_number_t) {
    let mut int_part: f64 = 0.0;
    modf((*c).f, core::ptr::addr_of_mut!(int_part));
    if fmod(int_part, 2 as f64) == 0 as f64 {
        calc.is_nan = TRUE as BOOL;
    } else {
        (*c).f = pow(10 as f64, (*c).f);
        if _finite((*c).f) == 0 {
            calc.is_nan = TRUE as BOOL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_ln(c: *mut calc_number_t) {
    if (*c).f <= 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*c).f = log((*c).f);
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_log(c: *mut calc_number_t) {
    if (*c).f <= 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*c).f = log10((*c).f);
    }
}

unsafe extern "C" fn stat_sum() -> ::core::ffi::c_double {
    let mut sum: ::core::ffi::c_double = 0 as ::core::ffi::c_double;
    let mut p: *mut statistic_t = calc.stat;
    while !p.is_null() {
        if (*p).base == IDC_RADIO_DEC as DWORD {
            sum += (*p).num.f;
        } else {
            sum += (*p).num.i as ::core::ffi::c_double;
        }
        p = (*p).next as *mut statistic_t;
    }
    return sum;
}

unsafe extern "C" fn stat_sum2() -> ::core::ffi::c_double {
    let mut sum: ::core::ffi::c_double = (0) as ::core::ffi::c_double;
    let mut p: *mut statistic_t = calc.stat;
    while !p.is_null() {
        if (*p).base == (IDC_RADIO_DEC) as DWORD {
            sum += (*p).num.f * (*p).num.f;
        } else {
            sum += (*p).num.i as ::core::ffi::c_double * (*p).num.i as ::core::ffi::c_double;
        }
        p = (*p).next as *mut statistic_t;
    }
    return sum;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_ave(c: *mut calc_number_t) {
    let mut ave: f64 = 0.0;
    let mut n: i32;

    ave = stat_sum();
    n = SendDlgItemMessage(calc.hStatWnd, (IDC_LIST_STAT as i32), LB_GETCOUNT, 0, 0) as i32;

    if n != 0 {
        ave = ave / (n as f64);
    }
    if calc.base == IDC_RADIO_DEC {
        (*c).f = ave;
    } else {
        (*c).i = ave as i64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_ave2(c: *mut calc_number_t) {
    let mut ave: f64 = 0.0;
    let mut n: i32;

    ave = stat_sum2();
    n = SendDlgItemMessage(calc.hStatWnd, (IDC_LIST_STAT as i32), LB_GETCOUNT, 0, 0) as i32;

    if n != 0 {
        ave = ave / (n as f64);
    }
    if calc.base == IDC_RADIO_DEC {
        (*c).f = ave;
    } else {
        (*c).i = ave as i64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_sum(c: *mut calc_number_t) {
    let mut sum: f64 = stat_sum();

    if calc.base == IDC_RADIO_DEC {
        (*c).f = sum;
    } else {
        (*c).i = sum as i64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpn_sum2(c: *mut calc_number_t) {
    let sum: f64 = stat_sum2();

    if calc.base == IDC_RADIO_DEC {
        (*c).f = sum;
    } else {
        (*c).i = sum as i64;
    }
}

unsafe extern "C" fn rpn_s_ex(mut c: *mut calc_number_t, mut pop_type: ::core::ffi::c_int) {
    let mut ave: ::core::ffi::c_double = 0 as ::core::ffi::c_double;
    let mut n: ::core::ffi::c_double = 0 as ::core::ffi::c_double;
    let mut dev: ::core::ffi::c_double = 0 as ::core::ffi::c_double;
    let mut num: ::core::ffi::c_double = 0 as ::core::ffi::c_double;
    let mut p: *mut statistic_t = calc.stat;
    ave = stat_sum();
    n = SendDlgItemMessage(
        calc.hStatWnd,
        IDC_LIST_STAT as ::core::ffi::c_int,
        LB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_double;
    if n == 0 as ::core::ffi::c_double {
        (*c).f = 0 as ::core::ffi::c_double;
        return;
    }
    ave = ave / n;
    dev = 0 as ::core::ffi::c_double;
    p = calc.stat;
    while !p.is_null() {
        if (*p).base == IDC_RADIO_DEC as DWORD {
            num = (*p).num.f;
        } else {
            num = (*p).num.i as ::core::ffi::c_double;
        }
        dev += pow(num - ave, (2) as ::core::ffi::c_double);
        p = (*p).next as *mut statistic_t;
    }
    dev = sqrt(
        dev / (if pop_type != 0 {
            n - (1) as ::core::ffi::c_double
        } else {
            n
        }),
    );
    if calc.base == IDC_RADIO_DEC as DWORD {
        (*c).f = dev;
    } else {
        (*c).i = dev as INT64;
    };
}
#[no_mangle]
pub unsafe extern "C" fn rpn_s(c: *mut calc_number_t) {
    rpn_s_ex(c, 0);
}
#[no_mangle]
pub unsafe extern "C" fn rpn_s_m1(c: *mut calc_number_t) {
    rpn_s_ex(c, 1);
}
#[no_mangle]
pub unsafe extern "C" fn rpn_dms2dec(c: *mut calc_number_t) {
    let mut d: f64 = 0.0;
    let mut m: f64;
    let mut s: f64;

    m = modf((*c).f, core::ptr::addr_of_mut!(d)) * (100 as f64);
    s = (modf(m, core::ptr::addr_of_mut!(m)) * (100 as f64)) + 0.5 as f64;
    modf(s, core::ptr::addr_of_mut!(s));

    m = m / (60 as f64);
    s = s / (3600 as f64);

    (*c).f = d + m + s;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_dec2dms(c: *mut calc_number_t) {
    let mut d: f64 = 0.0;
    let mut m: f64 = 0.0;
    let mut s: f64 = 0.0;
    m = modf((*c).f, core::ptr::addr_of_mut!(d)) * (60 as f64);
    s = ceil(modf(m, core::ptr::addr_of_mut!(m)) * (60 as f64));
    (*c).f = d + m / (100 as f64) + s / (10000 as f64);
}

#[no_mangle]
pub unsafe extern "C" fn rpn_zero(c: *mut calc_number_t) {
    (*c).f = 0.0;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_copy(dst: *mut calc_number_t, src: *mut calc_number_t) {
    *dst = *src;
}
#[no_mangle]
pub unsafe extern "C" fn rpn_is_zero(c: *mut calc_number_t) -> ::core::ffi::c_int {
    return ((*c).f == 0.0) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn rpn_alloc(c: *mut calc_number_t) {}

#[no_mangle]
pub unsafe extern "C" fn rpn_free(c: *mut calc_number_t) {}
