use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut rev: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut help: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut sortcol: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const MAXRECORDS: i32 = 65536;
pub const MAXLEN: i32 = 4095;
#[no_mangle]
pub unsafe extern "C" fn cmpr(
    a: *const ::core::ffi::c_void,
    b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut A: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut B: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();

    A = *(a as *mut *mut ::core::ffi::c_char);
    B = *(b as *mut *mut ::core::ffi::c_char);

    if sortcol > 0 {
        if strlen(A) > sortcol as usize {
            A = A.offset(sortcol as isize);
        } else {
            A = b"\0".as_ptr() as *mut ::core::ffi::c_char;
        }
        if strlen(B) > sortcol as usize {
            B = B.offset(sortcol as isize);
        } else {
            B = b"\0".as_ptr() as *mut ::core::ffi::c_char;
        }
    }

    if rev == 0 {
        return strcmp(A, B);
    } else {
        return strcmp(B, A);
    }
}

#[no_mangle]
pub unsafe extern "C" fn usage() {
    fputs(b"SORT\n\0".as_ptr() as *const i8, __iob_func().add(2));
    fputs(b"Sorts input and writes output to a file, console or a device.\n\0".as_ptr() as *const i8, __iob_func().add(2));

    if err != 0 {
        fputs(b"Invalid parameter\n\0".as_ptr() as *const i8, __iob_func().add(2));
    }

    fputs(b"    SORT [options] < [drive:1][path1]file1 > [drive2:][path2]file2\n\0".as_ptr() as *const i8, __iob_func().add(2));

    fputs(b"    Command | SORT [options] > [drive:][path]file\n\0".as_ptr() as *const i8, __iob_func().add(2));
    fputs(b"    Options:\n\0".as_ptr() as *const i8, __iob_func().add(2));
    fputs(b"    /R   Reverse order\n\0".as_ptr() as *const i8, __iob_func().add(2));
    fputs(b"    /+n  Start sorting with column n\n\0".as_ptr() as *const i8, __iob_func().add(2));
    fputs(b"    /?   Help\n\0".as_ptr() as *const i8, __iob_func().add(2));
}

#[no_mangle]
pub unsafe extern "C" fn main(mut argc: c_int, mut argv: *mut *mut c_char) -> c_int {
    let mut temp = [0 as c_char; (MAXLEN + 1) as usize];
    let mut list: *mut *mut c_char = core::ptr::null_mut();

    /* Option character pointer */
    let mut cp: *mut c_char = core::ptr::null_mut();
    let mut i: c_int = 0;
    let mut nr: c_int = 0;

    sortcol = 0;
    rev = 0;
    while {
        argc -= 1;
        argc
    } != 0
    {
        argv = argv.add(1);
        cp = *argv;
        if *cp == b'/' as c_char {
            match *cp.add(1 as usize) as u8 {
                b'R' | b'r' => {
                    rev = 1;
                }

                b'?' | b'h' | b'H' => {
                    help = 1;
                }

                b'+' => {
                    sortcol = atoi(cp.add(1 as usize) as *const u8);
                    if sortcol != 0 {
                        sortcol -= 1;
                    }
                }

                _ => {
                    err += 1;
                }
            }
        }
    }

    if err != 0 || help != 0 {
        usage();
        exit(1);
    }

    list = malloc((MAXRECORDS as usize) * core::mem::size_of::<*mut c_char>())
        as *mut *mut c_char;
    if list.is_null() {
        fputs(b"SORT: Insufficient memory\n\0".as_ptr() as *const c_char, __iob_func().add(2));
        exit(3);
    }

    nr = 0;
    while nr < MAXRECORDS {
        if fgets(temp.as_mut_ptr(), MAXLEN as c_int, __iob_func().add(0))
            .is_null()
        {
            break;
        }

        if strlen(temp.as_ptr()) != 0 {
            temp[strlen(temp.as_ptr()) - 1 as usize] = 0;
        }

        *list.offset(nr as isize) = malloc(
            strlen(temp.as_ptr()) + 1 as usize,
        ) as *mut c_char;
        if (*list.offset(nr as isize)).is_null() {
            fputs(b"SORT: Insufficient memory\n\0".as_ptr() as *const c_char, __iob_func().add(2));

            /* Cleanup memory */
            i = 0;
            while i < nr {
                free(*list.offset(i as isize) as *mut core::ffi::c_void);
                i += 1;
            }
            free(list as *mut core::ffi::c_void);
            exit(3);
        }

        strcpy(*list.offset(nr as isize), temp.as_ptr());
        nr += 1;
    }

    if nr == MAXRECORDS {
        fputs(
            b"SORT: number of records exceeds maximum\n\0".as_ptr() as *const c_char,
            __iob_func().add(2),
        );

        /* Cleanup memory */
        i = 0;
        while i < nr {
            free(*list.offset(i as isize) as *mut core::ffi::c_void);
            i += 1;
        }
        free(list as *mut core::ffi::c_void);

        /* Bail out */
        exit(4);
    }

    qsort(
        list as *mut core::ffi::c_void,
        nr as usize,
        core::mem::size_of::<*mut c_char>(),
        Some(cmpr),
    );

    i = 0;
    while i < nr {
        fputs(*list.offset(i as isize), __iob_func().add(1));
        fputs(b"\n\0".as_ptr() as *const c_char, __iob_func().add(1));
        i += 1;
    }

    /* Cleanup memory */
    i = 0;
    while i < nr {
        free(*list.offset(i as isize) as *mut core::ffi::c_void);
        i += 1;
    }
    free(list as *mut core::ffi::c_void);
    return 0;
}
