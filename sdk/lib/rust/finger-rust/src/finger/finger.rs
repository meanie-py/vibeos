use crate::finger::getopt::getopt;
use crate::finger::net::netfinger;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut __progname: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut now: time_t = 0;

#[no_mangle]
pub static mut sflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut lflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut mflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut pplan: ::core::ffi::c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn main(argc: i32, argv: *mut *mut i8) -> i32 {
    let mut argc = argc;
    let mut argv = argv;
    let mut ch: i32;

    while {
        ch = getopt(argc, argv, b"lmps\0".as_ptr() as *const i8);
        ch
    } != EOF
    {
        match ch {
            108 => {
                // 'l' long format
                lflag = 1;
            }
            109 => {
                // 'm' force exact match of names
                mflag = 1;
            }
            112 => {
                // 'p' don't show .plan/.project
                pplan = 1;
            }
            115 => {
                // 's' short format
                sflag = 1;
            }
            63 | _ => {
                // '?' and default
                let _ = fprintf(
                    __iob_func().add(2),
                    b"usage: finger [-lmps] login [...]\n\0".as_ptr() as *const i8,
                );
                exit(1);
            }
        }
    }
    argc -= optind;
    argv = argv.offset(optind as isize);

    let _ = time((core::ptr::addr_of_mut!(now)).cast());
    if (*argv).is_null() {
        usage();
    } else {
        userlist(argc, argv);
        // Assign explicit "large" format if names given and -s not
        // explicitly stated.  Force the -l AFTER we get names so any
        // remote finger attempts specified won't be mishandled.
        if sflag == 0 {
            lflag = 1; // if -s not explicit, force -l
        }
    }
    0
}

unsafe extern "C" fn userlist(argc: ::core::ffi::c_int, argv: *mut *mut ::core::ffi::c_char) {
    let mut used: *mut ::core::ffi::c_int = ::core::ptr::null_mut();
    let mut ap: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut nargv: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut np: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut p: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut wVersionRequested: WORD = 0;
    let mut wsaData: WSADATA = core::mem::zeroed();
    let mut iErr: ::core::ffi::c_int = 0;

    nargv = malloc(
        ((argc + 1) as usize)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    if nargv.is_null() || {
        used = calloc(
            (argc as usize) as _,
            (::core::mem::size_of::<::core::ffi::c_int>()) as _,
        ) as *mut ::core::ffi::c_int;
        used.is_null()
    } {
        err(
            1,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }

    /* Pull out all network requests into nargv. */
    p = argv;
    ap = p;
    np = nargv;
    while !(*p).is_null() {
        if !index!(*p, '@' as ::core::ffi::c_int).is_null() {
            let fresh15 = np;
            np = np.offset(1);
            *fresh15 = *p;
        } else {
            let fresh16 = ap;
            ap = ap.offset(1);
            *fresh16 = *p;
        }
        p = p.offset(1);
    }

    let fresh17 = np;
    np = np.offset(1);
    *fresh17 = ::core::ptr::null_mut();
    let fresh18 = ap;
    ap = ap.offset(1);
    *fresh18 = ::core::ptr::null_mut();

    /* If there are local requests */
    if !(*argv).is_null() {
        fprintf(
            __iob_func().add(2),
            b"Warning: Can't do local finger\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }

    /* Start winsock */
    wVersionRequested = MAKEWORD!(1, 1);
    iErr = WSAStartup(wVersionRequested, ::core::ptr::addr_of_mut!(wsaData));
    if iErr != 0 {
        /* Tell the user that we couldn't find a usable */
        /* WinSock DLL.                                  */
        fprintf(
            __iob_func().add(2),
            b"WSAStartup failed\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        free(nargv as *mut ::core::ffi::c_void);
        free(used as *mut ::core::ffi::c_void);
        return;
    }

    /* Handle network requests. */
    p = nargv;
    while !(*p).is_null() {
        let fresh19 = p;
        p = p.offset(1);
        netfinger(*fresh19);
    }

    /* Bring down winsock */
    WSACleanup();
    free(nargv as *mut ::core::ffi::c_void);
    free(used as *mut ::core::ffi::c_void);
    exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn usage() {
    fprintf(
        __iob_func().add(2),
        b"usage: finger [-lmps] login [...]\n\0".as_ptr() as *const i8,
    );
    exit(1);
}
