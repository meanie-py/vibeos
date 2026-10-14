use crate::ftp::main::getcmd;
use crate::ftp::main::makeargv;
use crate::ftp::ftp::null;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static sccsid: [u8; 37] = *b"@(#)domacro.c\t1.6 (Berkeley) 2/28/89\0";
#[no_mangle]
pub unsafe extern "C" fn domacro(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut cp1: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut cp2: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut count: ::core::ffi::c_int = 2;
    let mut loopflg: ::core::ffi::c_int = 0;
    let mut line2: [::core::ffi::c_char; 200] = [0; 200];
    let mut c: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let _ = getcmd;
    if argc < 2 {
        strcat(
            &raw mut line as *mut ::core::ffi::c_char,
            b" \0" as *const u8 as *const ::core::ffi::c_char,
        );
        printf(b"(macro name) \0" as *const u8 as *const ::core::ffi::c_char);
        fflush(__iob_func().add(1));
        gets(
            (&raw mut line as *mut ::core::ffi::c_char).offset(
                strlen(&raw mut line as *mut ::core::ffi::c_char) as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"Usage: %s macro_name.\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    i = 0;
    while i < macnum {
        if strncmp(
            *argv.offset(1 as isize),
            &raw mut (*(&raw mut macros as *mut macel).offset(i as isize)).mac_name
                as *mut ::core::ffi::c_char,
            9 as usize,
        ) == 0
        {
            break;
        }
        i += 1;
    }
    if i == macnum {
        printf(
            b"'%s' macro not found.\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
        );
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    strcpy(
        &raw mut line2 as *mut ::core::ffi::c_char,
        &raw mut line as *mut ::core::ffi::c_char,
    );
    loop {
        cp1 = macros[i as usize].mac_start;
        while cp1 != macros[i as usize].mac_end as *const ::core::ffi::c_char {
            while isspace(*cp1 as ::core::ffi::c_int) != 0 {
                cp1 = cp1.offset(1);
            }
            cp2 = &raw mut line as *mut ::core::ffi::c_char;
            while *cp1 as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                match *cp1 as ::core::ffi::c_int {
                    92 => {
                        cp1 = cp1.offset(1);
                        *cp2 = *cp1;
                        cp2 = cp2.offset(1);
                    }
                    36 => {
                        if isdigit(
                            *cp1.offset(1 as isize) as ::core::ffi::c_int,
                        ) != 0
                        {
                            j = 0;
                            loop {
                                cp1 = cp1.offset(1);
                                if !(isdigit(*cp1 as ::core::ffi::c_int) != 0) {
                                    break;
                                }
                                j = 10 * j + *cp1 as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int;
                            }
                            cp1 = cp1.offset(-1);
                            if argc - 2 >= j {
                                strcpy(
                                    cp2,
                                    *argv.offset((j + 1) as isize),
                                );
                                cp2 = cp2.offset(strlen(
                                    *argv.offset((j + 1) as isize),
                                ) as isize);
                            }
                        } else if *cp1.offset(1 as isize)
                            as ::core::ffi::c_int
                            == 'i' as ::core::ffi::c_int
                        {
                            loopflg = 1;
                            cp1 = cp1.offset(1);
                            if count < argc {
                                strcpy(cp2, *argv.offset(count as isize));
                                cp2 = cp2.offset(strlen(
                                    *argv.offset(count as isize),
                                ) as isize);
                            }
                        } else {
                            *cp2 = *cp1;
                            cp2 = cp2.offset(1);
                        }
                    }
                    _ => {
                        *cp2 = *cp1;
                        cp2 = cp2.offset(1);
                    }
                }
                if *cp1 as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                    cp1 = cp1.offset(1);
                }
            }
            *cp2 = '\0' as ::core::ffi::c_char;
            makeargv();
            c = getcmd(margv[0 as usize]) as *mut cmd;
            if c == (1 as usize).wrapping_neg() as *mut cmd {
                printf(b"?Ambiguous command\n\0" as *const u8 as *const ::core::ffi::c_char);
                code = -1;
            } else if c == 0 as *mut cmd {
                printf(b"?Invalid command\n\0" as *const u8 as *const ::core::ffi::c_char);
                code = -1;
            } else if (*c).c_conn as ::core::ffi::c_int != 0 && connected == 0 {
                printf(b"Not connected.\n\0" as *const u8 as *const ::core::ffi::c_char);
                code = -1;
            } else {
                if verbose != 0 {
                    printf(
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        &raw mut line as *mut ::core::ffi::c_char,
                    );
                }
                Some((*c).c_handler.unwrap_unchecked())
                    .unwrap_unchecked()(
                    margc,
                    &raw mut margv as *mut *const ::core::ffi::c_char,
                );
                if bell != 0 && (*c).c_bell as ::core::ffi::c_int != 0 {
                    putchar('\u{7}' as ::core::ffi::c_int);
                }
                strcpy(
                    &raw mut line as *mut ::core::ffi::c_char,
                    &raw mut line2 as *mut ::core::ffi::c_char,
                );
                makeargv();
                argc = margc;
                argv = &raw mut margv as *mut *const ::core::ffi::c_char;
            }
            if cp1 != macros[i as usize].mac_end as *const ::core::ffi::c_char {
                cp1 = cp1.offset(1);
            }
            fflush(__iob_func().add(1));
        }
        if !(loopflg != 0 && {
            count += 1;
            count < argc
        }) {
            break;
        }
    }
}
