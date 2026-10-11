use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _errno() -> *mut ::core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// #if defined(LIBC_SCCS) && !defined(lint)
pub static sccsid: [u8; 32] = *b"@(#)err.c\t8.1 (Berkeley) 6/4/93\0";
// #endif

// extern char *__progname;

#[no_mangle]
pub unsafe extern "C" fn err_v(eval: ::core::ffi::c_int, fmt: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    let mut ap: va_list = *(ap_list as *mut va_list);
    verr(eval, fmt, ap);
}



#[no_mangle]
pub unsafe extern "C" fn verr(eval: ::core::ffi::c_int, fmt: *const ::core::ffi::c_char, ap: va_list) {
    let mut sverrno: ::core::ffi::c_int;

    sverrno = *_errno();
    (fprintf)(__iob_func().add(2), b"%s: \0".as_ptr() as *const ::core::ffi::c_char, __progname);
    if !fmt.is_null() {
        (vfprintf)(__iob_func().add(2), fmt, ap);
        (fprintf)(__iob_func().add(2), b": \0".as_ptr() as *const ::core::ffi::c_char);
    }
    (fprintf)(
        __iob_func().add(2),
        b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
        (strerror)(sverrno),
    );
    (exit)(eval);
}

#[no_mangle]
pub unsafe extern "C" fn errx_v(eval: i32, fmt: *const i8, ap_list: *mut ::core::ffi::c_void) {
    let mut ap: va_list = *(ap_list as *mut va_list);
    verrx(eval, fmt, ap);
}



#[no_mangle]
pub unsafe extern "C" fn verrx(eval: ::core::ffi::c_int, fmt: *const ::core::ffi::c_char, ap: va_list) {
    let stderr: *mut FILE = (__iob_func() as *mut FILE).offset(2);
    fprintf(
        stderr,
        b"%s: \0".as_ptr() as *const ::core::ffi::c_char,
        __progname,
    );
    if !fmt.is_null() {
        vfprintf(stderr, fmt, ap);
    }
    fprintf(stderr, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    exit(eval);
}

#[no_mangle]
pub unsafe extern "C" fn warn_v(fmt: *const i8, ap_list: *mut ::core::ffi::c_void) {
    let mut ap: va_list = *(ap_list as *mut va_list);

    vwarn(fmt, ap);

}



#[no_mangle]
pub unsafe extern "C" fn vwarn(fmt: *const ::core::ffi::c_char, ap: va_list) {
    let mut sverrno: ::core::ffi::c_int = 0;

    sverrno = *_errno();
    (fprintf)(
        (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: \0" as *const u8 as *const ::core::ffi::c_char,
        __progname,
    );
    if !fmt.is_null() {
        (vfprintf)(
            (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            fmt,
            ap,
        );
        (fprintf)(
            (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            b": \0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    (fprintf)(
        (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
        (strerror)(sverrno),
    );
}
#[no_mangle]
pub unsafe extern "C" fn warnx_v(fmt: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    let mut ap: va_list = *(ap_list as *mut va_list);
    vwarnx(fmt, ap);
}



#[no_mangle]
pub unsafe extern "C" fn vwarnx(fmt: *const ::core::ffi::c_char, ap: va_list) {
    let _ = fprintf(
        (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
        b"%s: \0".as_ptr() as *const ::core::ffi::c_char,
        __progname,
    );
    if !fmt.is_null() {
        let _ = vfprintf(
            (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            fmt,
            ap,
        );
    }
    let _ = fprintf(
        (__iob_func() as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
