use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut optind: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[no_mangle]
pub static mut optopt: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut opterr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[no_mangle]
pub static mut optarg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();

pub static EMSG: [::core::ffi::c_char; 1] =
    unsafe { ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0") };

#[allow(dead_code)]
pub static sccsid: [u8; 37] = *b"@(#)getopt.c\t4.13 (Berkeley) 2/23/91\0";

pub const BADCH: i32 = b'?' as i32;

#[no_mangle]
pub unsafe extern "C" fn getopt(nargc: ::core::ffi::c_int, nargv: *const *mut ::core::ffi::c_char, ostr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    static mut place: *const c_char = (EMSG).as_ptr();
    let mut oli: *mut c_char = core::ptr::null_mut();
    let mut p: *mut c_char = core::ptr::null_mut();

    if *place == 0 {
        if optind >= nargc || {
            place = *nargv.offset(optind as isize);
            *place as c_int != '-' as c_int
        } {
            place = (EMSG).as_ptr();
            return EOF;
        }
        if *place.offset(1 as isize) as c_int != 0 && {
            place = place.offset(1);
            *place as c_int == '-' as c_int
        } {
            optind += 1;
            place = (EMSG).as_ptr();
            return EOF;
        }
    }

    let fresh15 = place;
    place = place.offset(1);
    optopt = *fresh15 as c_int;
    if optopt == ':' as c_int || {
        oli = strchr(ostr, optopt);
        oli.is_null()
    } {
        if optopt == '-' as c_int {
            return EOF;
        }
        if *place == 0 {
            optind += 1;
        }
        if opterr != 0 {
            p = (strrchr(*nargv as *const u8, '/' as c_int)).cast();
            if p.is_null() {
                p = *nargv;
            } else {
                p = p.offset(1);
            }
            fprintf(
                (__iob_func() as *mut FILE).offset(2 as isize) as *mut FILE,
                b"%s: illegal option -- %c\n\0".as_ptr() as *const c_char,
                p,
                optopt,
            );
        }
        return BADCH;
    }

    oli = oli.offset(1);
    if *oli as c_int != ':' as c_int {
        optarg = core::ptr::null::<c_char>();
        if *place == 0 {
            optind += 1;
        }
    } else {
        if *place != 0 {
            optarg = place;
        } else {
            optind += 1;
            if nargc <= optind {
                place = (EMSG).as_ptr();
                p = (strrchr(*nargv as *const u8, '/' as c_int)).cast();
                if p.is_null() {
                    p = *nargv;
                } else {
                    p = p.offset(1);
                }
                if opterr != 0 {
                    fprintf(
                        (__iob_func() as *mut FILE).offset(2 as isize) as *mut FILE,
                        b"%s: option requires an argument -- %c\n\0".as_ptr() as *const c_char,
                        p,
                        optopt,
                    );
                }
                return BADCH;
            } else {
                optarg = *nargv.offset(optind as isize);
            }
        }
        place = (EMSG).as_ptr();
        optind += 1;
    }
    return optopt;
}
