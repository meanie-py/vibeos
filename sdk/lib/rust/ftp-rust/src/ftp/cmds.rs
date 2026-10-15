use crate::ftp::main::r#type as type_0;
use crate::ftp::fake::blkfree;
use crate::ftp::fake::getpass;
use crate::ftp::ftp::getreply;
use crate::ftp::fake::glob;
use crate::ftp::main::home;
use crate::ftp::ftp::hookup;
use crate::ftp::ftp::login;
use crate::ftp::main::makeargv;
use crate::ftp::ftp::null;
use crate::ftp::ftp::pswitch;
use crate::ftp::ftp::reply_string;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static mut cout: SOCKET;
    static mut data: ::core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut mname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut jabort: jmp_buf = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut types: [types; 6] = unsafe {
    [
        types {
            t_name: b"ascii\0" as *const u8 as *const ::core::ffi::c_char,
            t_mode: b"A\0" as *const u8 as *const ::core::ffi::c_char,
            t_type: (TYPE_A as i32),
            t_arg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        },
        types {
            t_name: b"binary\0" as *const u8 as *const ::core::ffi::c_char,
            t_mode: b"I\0" as *const u8 as *const ::core::ffi::c_char,
            t_type: (TYPE_I as i32),
            t_arg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        },
        types {
            t_name: b"image\0" as *const u8 as *const ::core::ffi::c_char,
            t_mode: b"I\0" as *const u8 as *const ::core::ffi::c_char,
            t_type: (TYPE_I as i32),
            t_arg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        },
        types {
            t_name: b"ebcdic\0" as *const u8 as *const ::core::ffi::c_char,
            t_mode: b"E\0" as *const u8 as *const ::core::ffi::c_char,
            t_type: (TYPE_E as i32),
            t_arg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        },
        types {
            t_name: b"tenex\0" as *const u8 as *const ::core::ffi::c_char,
            t_mode: b"L\0" as *const u8 as *const ::core::ffi::c_char,
            t_type: (TYPE_L as i32),
            t_arg: &raw const bytename as *mut ::core::ffi::c_char,
        },
        types {
            t_name: ::core::ptr::null::<::core::ffi::c_char>(),
            t_mode: ::core::ptr::null::<::core::ffi::c_char>(),
            t_type: 0,
            t_arg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        },
    ]
};

#[no_mangle]
pub static mut stype: [*const ::core::ffi::c_char; 3] = [
    b"type\0" as *const u8 as *const ::core::ffi::c_char,
    b"\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut abortprox: jmp_buf = unsafe { core::mem::zeroed() };

pub static _PATH_TMP: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"/tmp/ftpXXXXXX\0") };

#[allow(dead_code)]
pub static mut sccsid: [u8; 35] = *b"@(#)cmds.c\t5.18 (Berkeley) 4/20/89\0";


#[repr(C)]
pub struct types {
    pub t_name: *const i8,
    pub t_mode: *const i8,
    pub t_type: i32,
    pub t_arg: *mut i8,
}

pub type Ptypes = *mut types;
#[no_mangle]
pub unsafe extern "C" fn setpeer(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut host: *mut c_char = core::ptr::null_mut();

    if connected != 0 {
        printf(
            b"Already connected to %s, use close first.\n\0".as_ptr() as *const c_char,
            hostname,
        );
        (fflush(__iob_func().add(1)));
        code = -(1);
        return;
    }
    if argc < 2 {
        (strcat(
            core::ptr::addr_of_mut!(line) as *mut c_char,
            b" \0".as_ptr() as *const c_char,
        ));
        printf(b"(to) \0".as_ptr() as *const c_char);
        (fflush(__iob_func().add(1)));
        (gets(
            (core::ptr::addr_of_mut!(line) as *mut c_char).offset(
                strlen(core::ptr::addr_of!(line) as *const c_char) as isize,
            ),
        ));
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const c_char;
    }
    if argc > 3 {
        printf(
            b"usage: %s host-name [port]\n\0".as_ptr() as *const c_char,
            *argv.offset(0 as isize),
        );
        (fflush(__iob_func().add(1)));
        code = -(1);
        return;
    }
    if argc > 2 {
        portnum = atoi(*argv.offset(2 as isize)) as i16;
        if (portnum as i32) <= 0 {
            printf(
                b"%s: bad port number-- %s\n\0".as_ptr() as *const c_char,
                *argv.offset(1 as isize),
                *argv.offset(2 as isize),
            );
            printf(
                b"usage: %s host-name [port]\n\0".as_ptr() as *const c_char,
                *argv.offset(0 as isize),
            );
            (fflush(__iob_func().add(1)));
            code = -(1);
            return;
        }
        portnum = htons(portnum as u16) as i16;
    }
    host = hookup(
        *argv.offset(1 as isize),
        portnum as c_int,
    );
    if !host.is_null() {
        let mut overbose: c_int = 0;
        connected = 1;
        if autologin != 0 {
            (login(
                *argv.offset(1 as isize) as *mut c_char,
            ));
        }

        overbose = verbose;
        if debug == 0 {
            verbose = -(1);
        }
        allbinary = 0;
        if command(b"SYST\0".as_ptr() as *const c_char) == (COMPLETE as i32)
            && overbose != 0
        {
            let mut cp: *mut c_char = core::ptr::null_mut();
            let mut c: c_char = core::mem::zeroed();
            cp = index(
                (core::ptr::addr_of_mut!(reply_string) as *mut c_char)
                    .offset(4 as isize),
                b' ' as c_int,
            );
            if cp.is_null() {
                cp = index(
                    (core::ptr::addr_of_mut!(reply_string) as *mut c_char)
                        .offset(4 as isize),
                    b'\r' as c_int,
                );
            }
            if !cp.is_null() {
                if *cp.offset(-(1 as isize)) as i32 == b'.' as i32 {
                    cp = cp.offset(-1);
                }
                c = *cp;
                *cp = 0;
            }

            printf(
                b"Remote system type is %s.\n\0".as_ptr() as *const c_char,
                (core::ptr::addr_of_mut!(reply_string) as *mut c_char)
                    .offset(4 as isize),
            );
            if !cp.is_null() {
                *cp = c;
            }
        }
        if strncmp(
            core::ptr::addr_of_mut!(reply_string) as *mut c_char,
            b"215 UNIX Type: L8\0".as_ptr() as *const c_char,
            17 as usize,
        ) == 0
        {
            setbinary(0, core::ptr::null_mut());
            /* allbinary = 1; this violates the RFC */
            if overbose != 0 {
                printf(
                    b"Using %s mode to transfer files.\n\0".as_ptr() as *const c_char,
                    core::ptr::addr_of_mut!(typename) as *mut c_char,
                );
            }
        } else if overbose != 0
            && strncmp(
                core::ptr::addr_of_mut!(reply_string) as *mut c_char,
                b"215 TOPS20\0".as_ptr() as *const c_char,
                10 as usize,
            ) == 0
        {
            printf(
                b"Remember to set tenex mode when transfering binary files from this machine.\n\0"
                    .as_ptr() as *const c_char,
            );
        }
        verbose = overbose;
    }
    (fflush(__iob_func().add(1)));
}
#[no_mangle]
pub unsafe extern "C" fn settype(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let mut p: *mut types = ::core::ptr::null_mut::<types>();
    let mut comret: ::core::ffi::c_int = 0;
    if argc > 2 {
        let mut sep: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        printf(
            b"usage: %s [\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        sep = b" \0" as *const u8 as *const ::core::ffi::c_char;
        p = ::core::ptr::addr_of_mut!(types) as *mut types;
        while !(*p).t_name.is_null() {
            printf(
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                sep,
                (*p).t_name,
            );
            if *sep as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                sep = b" | \0" as *const u8 as *const ::core::ffi::c_char;
            }
            p = p.offset(1);
        }
        printf(b" ]\n\0" as *const u8 as *const ::core::ffi::c_char);
        let _ = fflush(__iob_func().add(1));
        code = -(1);
        return;
    }
    if argc < 2 {
        printf(
            b"Using %s mode to transfer files.\n\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::ptr::addr_of_mut!(typename) as *mut ::core::ffi::c_char,
        );
        let _ = fflush(__iob_func().add(1));
        code = 0;
        return;
    }
    p = ::core::ptr::addr_of_mut!(types) as *mut types;
    while !(*p).t_name.is_null() {
        if strcmp(*argv.offset(1 as isize), (*p).t_name)
            == 0
        {
            break;
        }
        p = p.offset(1);
    }
    if (*p).t_name == (0 as *const ::core::ffi::c_char) {
        printf(
            b"%s: unknown mode\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
        );
        let _ = fflush(__iob_func().add(1));
        code = -(1);
        return;
    }
    if !(*p).t_arg.is_null() && *(*p).t_arg as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
        comret = command(
            b"TYPE %s %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*p).t_mode,
            (*p).t_arg,
        );
    } else {
        comret = command(
            b"TYPE %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*p).t_mode,
        );
    }
    if comret == (COMPLETE as i32) {
        strcpy(
            ::core::ptr::addr_of_mut!(typename) as *mut ::core::ffi::c_char,
            (*p).t_name,
        );
        type_0 = (*p).t_type;
    }
}
#[no_mangle]
pub unsafe extern "C" fn setbinary(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    stype[1] = b"binary\0".as_ptr() as *const i8;
    settype(2, core::ptr::addr_of_mut!(stype) as *mut *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn setascii(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    stype[1 as usize] = b"ascii\0".as_ptr() as *const core::ffi::c_char;
    settype(2 as core::ffi::c_int, core::ptr::addr_of_mut!(stype) as *mut *const core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn settenex(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    stype[1] = b"tenex\0".as_ptr() as *const i8;
    settype(2, core::ptr::addr_of_mut!(stype) as *mut *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn setebcdic() {
    stype[1] = b"ebcdic\0".as_ptr() as *const i8;
    settype(2, core::ptr::addr_of_mut!(stype) as *mut *const i8);
}

#[no_mangle]
pub unsafe extern "C" fn fsetmode(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    printf(
        b"We only support %s mode, sorry.\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!(modename) as *mut i8,
    );
    fflush(
        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
    );
    code = -1;
}
#[no_mangle]
pub unsafe extern "C" fn setform(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    printf(
        b"We only support %s format, sorry.\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!(formname) as *mut i8,
    );
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = -1;
}
#[no_mangle]
pub unsafe extern "C" fn setstruct(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    printf(
        b"We only support %s structure, sorry.\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!(structname) as *mut i8,
    );
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = -1;
}
#[no_mangle]
pub unsafe extern "C" fn reget(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    getit(argc, argv, 1, b"r+w\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn get(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    getit(
        argc,
        argv,
        0,
        if restart_point != 0 {
            b"r+w\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"w\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn getit(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *const ::core::ffi::c_char,
    mut restartit: ::core::ffi::c_int,
    mut mode_local: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut loc: ::core::ffi::c_int = 0;
    let mut oldargv1: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut oldargv2: *const ::core::ffi::c_char = ::core::ptr::null();

    if argc == 2 {
        argc += 1;
        *argv.offset(2 as isize) = *argv.offset(1 as isize);
        loc += 1;
    }
    if argc < 2 {
        strcat(&raw mut line as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(remote-file) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE);
        gets(
            (&raw mut line as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut line as *mut ::core::ffi::c_char) as isize),
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"usage: %s remote-file [ local-file ]\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush((__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE);
        code = -(1);
        return 0;
    }
    if argc < 3 {
        strcat(&raw mut line as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(local-file) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE);
        gets(
            (&raw mut line as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut line as *mut ::core::ffi::c_char) as isize),
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char;
    }
    if argc < 3 {
        printf(
            b"usage: %s remote-file [ local-file ]\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush((__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE);
        code = -(1);
        return 0;
    }
    oldargv1 = *argv.offset(1 as isize);
    oldargv2 = *argv.offset(2 as isize);
    if globulize(argv.offset(2 as isize) as *mut *const ::core::ffi::c_char) == 0 {
        code = -(1);
        return 0;
    }
    if loc != 0 && mcase != 0 {
        let mut tp: *const ::core::ffi::c_char = *argv.offset(1 as isize);
        let mut tp2: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
        let mut tmpbuf: [::core::ffi::c_char; (MAXPATHLEN as _)] = [0; (MAXPATHLEN as _)];

        while *tp != 0 && islower(*tp as ::core::ffi::c_int) == 0 {
            tp = tp.offset(1);
        }
        if *tp == 0 {
            tp = *argv.offset(2 as isize);
            tp2 = &raw mut tmpbuf as *mut ::core::ffi::c_char;
            loop {
                *tp2 = *tp;
                if !(*tp2 != 0) {
                    break;
                }
                if isupper(*tp2 as ::core::ffi::c_int) != 0 {
                    *tp2 = ('a' as ::core::ffi::c_int + *tp2 as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        as ::core::ffi::c_char;
                }
                tp = tp.offset(1);
                tp2 = tp2.offset(1);
            }
            *argv.offset(2 as isize) = &raw mut tmpbuf as *mut ::core::ffi::c_char;
        }
    }
    if loc != 0 && ntflag != 0 {
        *argv.offset(2 as isize) =
            dotrans(*argv.offset(2 as isize));
    }
    if loc != 0 && mapflag != 0 {
        *argv.offset(2 as isize) =
            domap(*argv.offset(2 as isize));
    }
    if restartit != 0 {
        let mut stbuf: stat = core::mem::zeroed();
        let mut ret: ::core::ffi::c_int = 0;

        ret = stat(
            *argv.offset(2 as isize),
            &raw mut stbuf,
        );
        if restartit == 1 {
            if ret < 0 {
                perror(*argv.offset(2 as isize));
                return 0;
            }
            restart_point = stbuf.st_size as off_t;
        } else {
            if ret == 0 {
                let mut overbose: ::core::ffi::c_int = 0;

                overbose = verbose;
                if debug == 0 {
                    verbose = -(1);
                }
                if command(
                    b"MDTM %s\0".as_ptr() as *const ::core::ffi::c_char,
                    *argv.offset(1 as isize),
                ) == (COMPLETE as i32)
                {
                    let mut yy: ::core::ffi::c_int = 0;
                    let mut mo: ::core::ffi::c_int = 0;
                    let mut day: ::core::ffi::c_int = 0;
                    let mut hour: ::core::ffi::c_int = 0;
                    let mut min: ::core::ffi::c_int = 0;
                    let mut sec: ::core::ffi::c_int = 0;
                    let mut tm: *mut tm = ::core::ptr::null_mut();

                    verbose = overbose;
                    sscanf(
                        &raw mut reply_string as *mut ::core::ffi::c_char,
                        b"%*s %04d%02d%02d%02d%02d%02d\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut yy,
                        &raw mut mo,
                        &raw mut day,
                        &raw mut hour,
                        &raw mut min,
                        &raw mut sec,
                    );
                    tm = gmtime(&raw mut stbuf.st_mtime);
                    (*tm).tm_mon += 1;
                    if (*tm).tm_year > yy % 100 {
                        return 1;
                    } else if (*tm).tm_year == yy % 100 {
                        if (*tm).tm_mon > mo {
                            return 1;
                        }
                    } else if (*tm).tm_mon == mo {
                        if (*tm).tm_mday > day {
                            return 1;
                        }
                    } else if (*tm).tm_mday == day {
                        if (*tm).tm_hour > hour {
                            return 1;
                        }
                    } else if (*tm).tm_hour == hour {
                        if (*tm).tm_min > min {
                            return 1;
                        }
                    } else if (*tm).tm_min == min {
                        if (*tm).tm_sec > sec {
                            return 1;
                        }
                    }
                } else {
                    printf(
                        b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut reply_string as *mut ::core::ffi::c_char,
                    );
                    fflush((__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE);
                    verbose = overbose;
                    return 0;
                }
            }
        }
    }

    recvrequest(
        b"RETR\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(2 as isize),
        *argv.offset(1 as isize),
        mode_local,
        (*argv.offset(1 as isize) != oldargv1
            || *argv.offset(2 as isize) != oldargv2) as ::core::ffi::c_int,
    );
    restart_point = 0 as off_t;
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn remglob(
    mut argv: *mut *const ::core::ffi::c_char,
    mut doswitch: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut temp: [::core::ffi::c_char; 16] = [0; 16];
    static mut buf: [::core::ffi::c_char; (MAXPATHLEN as _)] = [0; (MAXPATHLEN as _)];
    static mut ftemp: *mut FILE = ::core::ptr::null_mut::<FILE>();
    static mut args: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut oldverbose: ::core::ffi::c_int = 0;
    let mut oldhash: ::core::ffi::c_int = 0;
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut mode_local: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut terminator: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if mflag == 0 {
        if doglob == 0 {
            args = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
        } else if !ftemp.is_null() {
            fclose(ftemp);
            ftemp = ::core::ptr::null_mut::<FILE>();
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if doglob == 0 {
        if args.is_null() {
            args = argv as *mut *const ::core::ffi::c_char;
        }
        args = args.offset(1);
        cp = *args;
        if cp.is_null() {
            args = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
        }
        return cp;
    }
    if ftemp.is_null() {
        strcpy(
            &raw mut temp as *mut ::core::ffi::c_char,
            _PATH_TMP.as_ptr(),
        );
        mktemp(&raw mut temp as *mut ::core::ffi::c_char);
        oldverbose = verbose;
        verbose = 0 as ::core::ffi::c_int;
        oldhash = hash;
        hash = 0 as ::core::ffi::c_int;
        if doswitch != 0 {
            pswitch((proxy == 0) as ::core::ffi::c_int);
        }
        mode_local = b"w\0" as *const u8 as *const ::core::ffi::c_char;
        loop {
            argv = argv.offset(1);
            if (*argv).is_null() {
                break;
            }
            recvrequest(
                b"NLST\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut temp as *mut ::core::ffi::c_char,
                *argv,
                mode_local,
                0 as ::core::ffi::c_int,
            );
            mode_local = b"a\0" as *const u8 as *const ::core::ffi::c_char;
        }
        if doswitch != 0 {
            pswitch((proxy == 0) as ::core::ffi::c_int);
        }
        verbose = oldverbose;
        hash = oldhash;
        ftemp = fopen(
            &raw mut temp as *mut ::core::ffi::c_char,
            b"r\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut FILE;
        unlink(&raw mut temp as *mut ::core::ffi::c_char);
        if ftemp.is_null() {
            printf(
                b"can't find list of remote files, oops\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            fflush(__iob_func().add(1));
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
    }
    if fgets(
        &raw mut buf as *mut ::core::ffi::c_char,
        core::mem::size_of::<[::core::ffi::c_char; (MAXPATHLEN as _)]>()
            as ::core::ffi::c_int,
        ftemp,
    )
    .is_null()
    {
        fclose(ftemp);
        ftemp = ::core::ptr::null_mut::<FILE>();
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    terminator = index(&raw mut buf as *mut ::core::ffi::c_char, '\n' as i32);
    if !terminator.is_null() {
        *terminator = '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut buf as *mut ::core::ffi::c_char;
}

unsafe extern "C" fn onoff(bool: core::ffi::c_int) -> *const core::ffi::c_char {
    return if bool != 0 {
        b"on\0".as_ptr() as *const core::ffi::c_char
    } else {
        b"off\0".as_ptr() as *const core::ffi::c_char
    };
}
#[no_mangle]
pub unsafe extern "C" fn status(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0;
    if connected != 0 {
        printf(
            b"Connected to %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            hostname,
        );
    } else {
        printf(b"Not connected.\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if proxy == 0 {
        pswitch(1);
        if connected != 0 {
            printf(
                b"Connected for proxy commands to %s.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                hostname,
            );
        } else {
            printf(
                b"No proxy connection.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        pswitch(0);
    }
    printf(
        b"Mode: %s; Type: %s; Form: %s; Structure: %s\n\0" as *const u8
            as *const ::core::ffi::c_char,
        &raw mut modename as *mut ::core::ffi::c_char,
        &raw mut typename as *mut ::core::ffi::c_char,
        &raw mut formname as *mut ::core::ffi::c_char,
        &raw mut structname as *mut ::core::ffi::c_char,
    );
    printf(
        b"Verbose: %s; Bell: %s; Prompting: %s; Globbing: %s\n\0" as *const u8
            as *const ::core::ffi::c_char,
        onoff(verbose),
        onoff(bell),
        onoff(interactive),
        onoff(doglob),
    );
    printf(
        b"Store unique: %s; Receive unique: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        onoff(sunique),
        onoff(runique),
    );
    printf(
        b"Case: %s; CR stripping: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        onoff(mcase),
        onoff(crflag),
    );
    if ntflag != 0 {
        printf(
            b"Ntrans: (in) %s (out) %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut ntin as *mut ::core::ffi::c_char,
            &raw mut ntout as *mut ::core::ffi::c_char,
        );
    } else {
        printf(b"Ntrans: off\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if mapflag != 0 {
        printf(
            b"Nmap: (in) %s (out) %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut mapin as *mut ::core::ffi::c_char,
            &raw mut mapout as *mut ::core::ffi::c_char,
        );
    } else {
        printf(b"Nmap: off\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
    printf(
        b"Hash mark printing: %s; Use of PORT cmds: %s\n\0" as *const u8
            as *const ::core::ffi::c_char,
        onoff(hash),
        onoff(sendport),
    );
    if macnum > 0 {
        printf(b"Macros:\n\0" as *const u8 as *const ::core::ffi::c_char);
        i = 0;
        while i < macnum {
            printf(
                b"\t%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut (*(&raw mut macros as *mut macel).offset(i as isize)).mac_name
                    as *mut ::core::ffi::c_char,
            );
            i += 1;
        }
    }
    fflush(
        (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
    );
    code = 0;
}
#[no_mangle]
pub unsafe extern "C" fn setbell(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    bell = (bell == 0) as i32;
    printf(
        b"Bell mode %s.\n\0".as_ptr() as *const i8,
        onoff(bell),
    );
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = bell;
}
#[no_mangle]
pub unsafe extern "C" fn settrace(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    trace = (trace == 0) as ::core::ffi::c_int;
    printf(
        b"Packet tracing %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
        onoff(trace),
    );
    let _ = fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = trace;
}

#[no_mangle]
pub unsafe extern "C" fn sethash(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    hash = (hash == 0) as i32;
    printf(b"Hash mark printing %s\0".as_ptr() as *const i8, onoff(hash));
    code = hash;
    if hash != 0 {
        printf(b" (%d bytes/hash mark)\0".as_ptr() as *const i8, 1024);
    }
    printf(b".\n\0".as_ptr() as *const i8);
    let _ = fflush(__iob_func().add(1) as *mut FILE);
}
#[no_mangle]
pub unsafe extern "C" fn setverbose(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    verbose = (verbose == 0) as ::core::ffi::c_int;
    printf(
        b"Verbose mode %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
        onoff(verbose),
    );
    let _ = (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE;
    fflush((__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE);
    code = verbose;
}
#[no_mangle]
pub unsafe extern "C" fn setport(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    sendport = (sendport == 0) as i32;
    printf(b"Use of PORT cmds %s.\n\0".as_ptr() as *const i8, onoff(sendport));
    let _ = fflush(__iob_func().add(1));
    code = sendport;
}
#[no_mangle]
pub unsafe extern "C" fn setprompt(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    interactive = (interactive == 0) as ::core::ffi::c_int;
    printf(
        b"Interactive mode %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
        onoff(interactive),
    );
    let _ = fflush((__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE);
    code = interactive;
}
#[no_mangle]
pub unsafe extern "C" fn setglob(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    doglob = (doglob == 0) as i32;
    printf(
        b"Globbing %s.\n\0".as_ptr() as *const i8,
        onoff(doglob),
    );
    let _ = fflush(
        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
    );
    code = doglob;
}
#[no_mangle]
pub unsafe extern "C" fn setdebug(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut val: ::core::ffi::c_int = 0;
    if argc > 1 {
        val = atoi(*argv.offset(1 as isize));
        if val < 0 {
            printf(
                b"%s: bad debugging value.\n\0".as_ptr() as *const ::core::ffi::c_char,
                *argv.offset(1 as isize),
            );
            fflush(__iob_func().add(1));
            code = -1;
            return;
        }
    } else {
        val = (debug == 0) as ::core::ffi::c_int;
    }
    debug = val;
    if debug != 0 {
        options |= (SO_DEBUG as i32);
    } else {
        options &= (!SO_DEBUG) as i32;
    }
    printf(
        b"Debugging %s (debug=%d).\n\0".as_ptr() as *const ::core::ffi::c_char,
        onoff(debug),
        debug,
    );
    fflush(__iob_func().add(1));
    code = (debug > 0) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cd(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc < 2 as ::core::ffi::c_int {
        strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(remote-directory) \0".as_ptr() as *const ::core::ffi::c_char);
        let _ = fflush(__iob_func().add(1));
        let _ = gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char)
                    as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 as ::core::ffi::c_int {
        printf(
            b"usage:%s remote-directory\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(__iob_func().add(1));
        code = -(1 as ::core::ffi::c_int);
        return;
    }
    if command(
        b"CWD %s\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    ) == ERROR as ::core::ffi::c_int
        && code == 500 as ::core::ffi::c_int
    {
        if verbose != 0 {
            printf(
                b"CWD command not recognized, trying XCWD\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            let _ = fflush(__iob_func().add(1));
        }
        let _ = command(
            b"XCWD %s\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn lcd(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut buf: [::core::ffi::c_char; (MAXPATHLEN as _)] = [0; (MAXPATHLEN as _)];
    if argc < 2 {
        argc += 1;
        let ref mut fresh37 = *argv.offset(1 as isize);
        *fresh37 = &raw mut home as *const ::core::ffi::c_char;
    }
    if argc != 2 {
        printf(
            b"usage:%s local-directory\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(
            (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        );
        code = -(1);
        return;
    }
    if globulize(
        &raw mut *argv.offset(1 as isize) as *mut *const ::core::ffi::c_char,
    ) == 0
    {
        code = -(1);
        return;
    }
    if chdir(*argv.offset(1 as isize))
        < 0
    {
        perror(*argv.offset(1 as isize));
        code = -(1);
        return;
    }
    printf(
        b"Local directory now %s\n\0" as *const u8 as *const ::core::ffi::c_char,
        getcwd(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; (MAXPATHLEN as _)]>()
                as ::core::ffi::c_int,
        ),
    );
    fflush(
        (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
    );
    code = 0;
}
#[no_mangle]
pub unsafe extern "C" fn delete(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc < 2 {
        strcat(
            &raw mut line as *mut ::core::ffi::c_char,
            b" \0" as *const u8 as *const ::core::ffi::c_char,
        );
        printf(b"(remote-file) \0" as *const u8 as *const ::core::ffi::c_char);
        fflush(__iob_func().add(1));
        gets(
            (&raw mut line as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut line as *mut ::core::ffi::c_char) as isize)
                as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"usage:%s remote-file\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -(1);
        return;
    }
    command(
        b"DELE %s\0" as *const u8 as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn renamefile(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut at: u8 = 0;
    loop {
        match at {
            0 => {
                if argc < 2 {
                    strcat(
                        core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    printf(b"(from-name) \0".as_ptr() as *const ::core::ffi::c_char);
                    fflush(
                        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
                    );
                    gets(
                        (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                            strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char)
                                as isize,
                        ) as *mut ::core::ffi::c_char,
                    );
                    makeargv();
                    argc = margc;
                    argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
                }
                at = 1;
                continue;
            }
            1 => {
                if argc < 2 {
                    at = 2;
                    continue;
                }
                at = 3;
                continue;
            }
            2 => {
                // usage:
                printf(
                    b"%s from-name to-name\n\0".as_ptr() as *const ::core::ffi::c_char,
                    *argv.offset(0 as isize),
                );
                fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
                code = -1;
                return;
            }
            3 => {
                if argc < 3 {
                    strcat(
                        core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    printf(b"(to-name) \0".as_ptr() as *const ::core::ffi::c_char);
                    fflush(
                        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
                    );
                    gets(
                        (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                            strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char)
                                as isize,
                        ) as *mut ::core::ffi::c_char,
                    );
                    makeargv();
                    argc = margc;
                    argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
                }
                at = 4;
                continue;
            }
            4 => {
                if argc < 3 {
                    at = 2;
                    continue;
                }
                if command(
                    b"RNFR %s\0".as_ptr() as *const ::core::ffi::c_char,
                    *argv.offset(1 as isize),
                ) == (CONTINUE as i32)
                {
                    command(
                        b"RNTO %s\0".as_ptr() as *const ::core::ffi::c_char,
                        *argv.offset(2 as isize),
                    );
                }
                return;
            }
            _ => break,
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ls(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if argc < 2 as ::core::ffi::c_int {
        argc += 1;
        *argv.offset(1 as isize) = ::core::ptr::null::<::core::ffi::c_char>();
    }
    if argc < 3 as ::core::ffi::c_int {
        argc += 1;
        *argv.offset(2 as isize) = b"-\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if argc > 3 as ::core::ffi::c_int {
        printf(
            b"usage: %s remote-directory local-file\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush((__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE);
        code = -(1 as ::core::ffi::c_int);
        return;
    }
    cmd = if *(*argv.offset(0 as isize)).offset(0 as isize)
        as ::core::ffi::c_int
        == 'n' as ::core::ffi::c_int
    {
        b"NLST\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"LIST\0".as_ptr() as *const ::core::ffi::c_char
    };
    if strcmp(
        *argv.offset(2 as isize),
        b"-\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
        && globulize(argv.offset(2 as isize) as *mut *const ::core::ffi::c_char) == 0
    {
        code = -(1 as ::core::ffi::c_int);
        return;
    }
    if strcmp(
        *argv.offset(2 as isize),
        b"-\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
        && **argv.offset(2 as isize) as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        if globulize(argv.offset(2 as isize) as *mut *const ::core::ffi::c_char) == 0
            || confirm(
                b"output to local-file:\0".as_ptr() as *const ::core::ffi::c_char,
                *argv.offset(2 as isize),
            ) == 0
        {
            code = -(1 as ::core::ffi::c_int);
            return;
        }
    }
    recvrequest(
        cmd,
        *argv.offset(2 as isize),
        *argv.offset(1 as isize),
        b"w\0".as_ptr() as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn shell(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let mut AppName: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut ShellCmd: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut CmdLine: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut i: ::core::ffi::c_int = 0;
    let mut ProcessInformation: PROCESS_INFORMATION = ::core::mem::zeroed();
    let mut Result: BOOL = 0;
    let mut StartupInfo: STARTUPINFO = ::core::mem::zeroed();
    let mut ShellName: [::core::ffi::c_char; 8] = [
        'C' as ::core::ffi::c_char,
        'O' as ::core::ffi::c_char,
        'M' as ::core::ffi::c_char,
        'S' as ::core::ffi::c_char,
        'P' as ::core::ffi::c_char,
        'E' as ::core::ffi::c_char,
        'C' as ::core::ffi::c_char,
        0,
    ];
    let mut NumBytes: ::core::ffi::c_int = 0;

    NumBytes = GetEnvironmentVariable(
        core::ptr::addr_of_mut!(ShellName) as *mut ::core::ffi::c_char as LPCSTR,
        core::ptr::addr_of_mut!(ShellCmd) as *mut ::core::ffi::c_char as LPSTR,
        MAX_PATH,
    ) as ::core::ffi::c_int;

    if NumBytes == 0 {
        return;
    }

    AppName = core::ptr::addr_of_mut!(ShellCmd) as *mut ::core::ffi::c_char;
    strcpy(
        core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(ShellCmd) as *mut ::core::ffi::c_char,
    );

    if argc > 1 {
        strncat(
            core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
            b" /C\0".as_ptr() as *const ::core::ffi::c_char,
            (MAX_PATH as size_t)
                .wrapping_sub(strlen(
                    core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
                ))
                .wrapping_sub(1 as size_t),
        );
    }

    i = 1;
    while i < argc {
        strncat(
            core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
            (MAX_PATH as size_t)
                .wrapping_sub(strlen(
                    core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
                ))
                .wrapping_sub(1 as size_t),
        );
        strncat(
            core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
            *argv.offset(i as isize),
            (MAX_PATH as size_t)
                .wrapping_sub(strlen(
                    core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char,
                ))
                .wrapping_sub(1 as size_t),
        );
        i += 1;
    }

    StartupInfo.cb = ::core::mem::size_of::<STARTUPINFO>() as DWORD;
    StartupInfo.lpReserved = core::ptr::null_mut();
    StartupInfo.lpDesktop = core::ptr::null_mut();
    StartupInfo.lpTitle = core::ptr::null_mut();
    StartupInfo.dwX = 0;
    StartupInfo.dwY = 0;
    StartupInfo.dwXSize = 0;
    StartupInfo.dwYSize = 0;
    StartupInfo.dwFlags = 0;
    StartupInfo.wShowWindow = 0;
    StartupInfo.cbReserved2 = 0;
    StartupInfo.lpReserved2 = core::ptr::null_mut();

    Result = CreateProcess(
        AppName as LPCSTR,
        core::ptr::addr_of_mut!(CmdLine) as *mut ::core::ffi::c_char as LPSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE as i32),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(StartupInfo),
        core::ptr::addr_of_mut!(ProcessInformation),
    );

    if Result != 0 {
        WaitForSingleObject(ProcessInformation.hProcess, 0xffffffff);

        CloseHandle(ProcessInformation.hProcess);
    }
}
#[no_mangle]
pub unsafe extern "C" fn user(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let _ = getpass as usize;
    let mut acct: [::core::ffi::c_char; 80] = [0; 80];
    let mut n: ::core::ffi::c_int = 0;
    let mut aflag: ::core::ffi::c_int = 0;
    if argc < 2 {
        let _ = strcat(
            &raw mut line as *mut ::core::ffi::c_char,
            b" \0" as *const u8 as *const ::core::ffi::c_char,
        );
        printf(b"(username) \0" as *const u8 as *const ::core::ffi::c_char);
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
        );
        let _ = gets(
            (&raw mut line as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut line as *mut ::core::ffi::c_char) as isize)
                as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char as *mut *const ::core::ffi::c_char;
    }
    if argc > 4 {
        printf(
            b"usage: %s username [password] [account]\n\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
        );
        code = -(1);
        return;
    }
    n = command(
        b"USER %s\0" as *const u8 as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    );
    if n == (CONTINUE as i32) {
        if argc < 3 {
            let fresh47 = &mut *argv.offset(2 as isize);
            *fresh47 = getpass(
                b"Password: \0" as *const u8 as *const ::core::ffi::c_char,
            );
            argc += 1;
        }
        n = command(
            b"PASS %s\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(2 as isize),
        );
    }
    if n == (CONTINUE as i32) {
        if argc < 4 {
            printf(b"Account: \0" as *const u8 as *const ::core::ffi::c_char);
            let _ = fflush(
                (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
            );
            let _ = fflush(
                (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
            );
            let _ = fgets(
                &raw mut acct as *mut ::core::ffi::c_char,
                (::core::mem::size_of::<[::core::ffi::c_char; 80]>() as usize)
                    .wrapping_sub(1 as usize) as ::core::ffi::c_int,
                (__iob_func() as *mut FILE).offset(0 as isize) as *mut FILE,
            );
            acct[(strlen(&raw mut acct as *mut ::core::ffi::c_char) as usize)
                .wrapping_sub(1 as usize)] = '\0' as i32 as ::core::ffi::c_char;
            let fresh48 = &mut *argv.offset(3 as isize);
            *fresh48 = &raw mut acct as *mut ::core::ffi::c_char as *const ::core::ffi::c_char;
            argc += 1;
        }
        n = command(
            b"ACCT %s\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(3 as isize),
        );
        aflag += 1;
    }
    if n != (COMPLETE as i32) {
        fprintf(
            (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
            b"Login failed.\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1 as isize) as *mut FILE,
        );
        return;
    }
    if aflag == 0 && argc == 4 {
        let _ = command(
            b"ACCT %s\0" as *const u8 as *const ::core::ffi::c_char,
            *argv.offset(3 as isize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn pwd(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let mut oldverbose: ::core::ffi::c_int = verbose;

    /*
     * If we aren't verbose, this doesn't do anything!
     */
    verbose = 1;
    if command(b"PWD\0".as_ptr() as *const ::core::ffi::c_char) == (ERROR as i32)
        && code == 500
    {
        printf(
            b"PWD command not recognized, trying XPWD\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        command(b"XPWD\0".as_ptr() as *const ::core::ffi::c_char);
    }
    verbose = oldverbose;
}
#[no_mangle]
pub unsafe extern "C" fn makedir(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc < 2 {
        strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(directory-name) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush(__iob_func().add(1));
        gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                strlen(
                    core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
                ) as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"usage: %s directory-name\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    if command(
        b"MKD %s\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    ) == (ERROR as i32)
        && code == 500
    {
        if verbose != 0 {
            printf(
                b"MKD command not recognized, trying XMKD\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            fflush(__iob_func().add(1));
        }
        command(
            b"XMKD %s\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn removedir(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc < 2 as ::core::ffi::c_int {
        strcat(
            &raw mut line as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(directory-name) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush(__iob_func().add(1));
        gets(
            (&raw mut line as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut line as *mut ::core::ffi::c_char) as isize)
                as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 as ::core::ffi::c_int {
        printf(
            b"usage: %s directory-name\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -(1 as ::core::ffi::c_int);
        return;
    }
    if command(
        b"RMD %s\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    ) == ERROR as ::core::ffi::c_int
        && code == 500 as ::core::ffi::c_int
    {
        if verbose != 0 {
            printf(
                b"RMD command not recognized, trying XRMD\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            fflush(__iob_func().add(1));
        }
        command(
            b"XRMD %s\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn quote(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; (BUFSIZ) as _] = [0; (BUFSIZ) as _];
    if argc < 2 {
        strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(command line to send) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush(__iob_func().add(1));
        gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char) as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"usage: %s line-to-send\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    strcpy(
        core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
        *argv.offset(1 as isize),
    );
    i = 2;
    while i < argc {
        strcat(
            core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        strcat(
            core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
            *argv.offset(i as isize),
        );
        i += 1;
    }
    if command(core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char) == (PRELIM as i32) {
        while getreply(0) == (PRELIM as i32) {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn site(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int;
    let mut buf: [::core::ffi::c_char; (BUFSIZ) as _] = [0; (BUFSIZ) as _];

    if argc < 2 {
        let _ = strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        let _ = printf(b"(arguments to SITE command) \0".as_ptr() as *const ::core::ffi::c_char);
        let _ = fflush(__iob_func().add(1));
        let _ = gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char) as isize,
            ),
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        let _ = printf(
            b"usage: %s line-to-send\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    let _ = strcpy(
        core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
        b"SITE \0".as_ptr() as *const ::core::ffi::c_char,
    );
    let _ = strcat(
        core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
        *argv.offset(1 as isize),
    );
    i = 2;
    while i < argc {
        let _ = strcat(
            core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        let _ = strcat(
            core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
            *argv.offset(i as isize),
        );
        i += 1;
    }
    if command(core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char) == (PRELIM as i32) {
        while getreply(0) == (PRELIM as i32) {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_chmod(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc == 2 {
        printf(
            b"usage: %s mode file-name\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(__iob_func().add(1));
        code = -(1);
        return;
    }
    if argc < 3 {
        let _ = strcat(
            &raw mut line as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(mode and file-name) \0".as_ptr() as *const ::core::ffi::c_char);
        let _ = fflush(__iob_func().add(1));
        let _ = gets(
            (&raw mut line as *mut ::core::ffi::c_char).offset(
                strlen(&raw mut line as *mut ::core::ffi::c_char) as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = &raw mut margv as *mut *const ::core::ffi::c_char as *mut *const ::core::ffi::c_char;
    }
    if argc != 3 {
        printf(
            b"usage: %s mode file-name\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(__iob_func().add(1));
        code = -(1);
        return;
    }
    let _ = command(
        b"SITE CHMOD %s %s\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
        *argv.offset(2 as isize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn do_umask(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut oldverbose: ::core::ffi::c_int = verbose;
    verbose = 1;
    let _ = command(
        if argc == 1 {
            b"SITE UMASK\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"SITE UMASK %s\0".as_ptr() as *const ::core::ffi::c_char
        },
        *argv.offset(1 as isize),
    );
    verbose = oldverbose;
}
#[no_mangle]
pub unsafe extern "C" fn idle(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let mut oldverbose: i32 = verbose;
    verbose = 1;
    command(
        if argc == 1 {
            b"SITE IDLE\0".as_ptr() as *const i8
        } else {
            b"SITE IDLE %s\0".as_ptr() as *const i8
        },
        *argv.offset(1 as isize),
    );
    verbose = oldverbose;
}
#[no_mangle]
pub unsafe extern "C" fn rmthelp(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let oldverbose: i32 = verbose;
    verbose = 1;
    command(
        if argc == 1 {
            b"HELP\0".as_ptr() as *const i8
        } else {
            b"HELP %s\0".as_ptr() as *const i8
        },
        *argv.offset(1 as isize),
    );
    verbose = oldverbose;
}
#[no_mangle]
pub unsafe extern "C" fn quit(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if connected != 0 {
        disconnect(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        );
    }
    pswitch(1 as ::core::ffi::c_int);
    if connected != 0 {
        disconnect(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        );
    }
    exit(0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn disconnect(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if connected == 0 {
        return;
    }
    let _ = command(b"QUIT\0".as_ptr() as *const ::core::ffi::c_char);
    cout = 0;
    connected = 0;
    data = -1;
    if proxy == 0 {
        macnum = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn confirm(
    cmd: *const ::core::ffi::c_char,
    file: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut line_local: [::core::ffi::c_char; (BUFSIZ) as _] = [0; (BUFSIZ) as _];

    if interactive == 0 {
        return 1 as ::core::ffi::c_int;
    }
    printf(
        b"%s %s? \0".as_ptr() as *const ::core::ffi::c_char,
        cmd,
        file,
    );
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    gets(&raw mut line_local as *mut ::core::ffi::c_char);
    return (*(&raw mut line_local as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 'n' as i32
        && *(&raw mut line_local as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 'N' as i32)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn globulize(cpp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut globbed: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if doglob == 0 {
        return 1;
    }
    globbed = glob(*cpp);
    if !globerr.is_null() {
        printf(
            b"%s: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            *cpp,
            globerr,
        );
        fflush(
            (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize)
                as *mut FILE,
        );
        if !globbed.is_null() {
            blkfree(globbed);
            free(globbed as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
        }
        return 0;
    }
    if !globbed.is_null() {
        *cpp = *globbed;
        if !(*globbed.offset(1 as isize)).is_null() {
            blkfree(
                globbed.offset(1 as isize) as *mut *mut ::core::ffi::c_char
            );
        }
        free(globbed as *mut ::core::ffi::c_void);
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn account(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let _getpass_decl = getpass as usize;
    let mut acct: [::core::ffi::c_char; 50] = [0; 50];
    let mut ap: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if argc > 1 as ::core::ffi::c_int {
        argv = argv.offset(1);
        argc -= 1;
        strncpy(
            &raw mut acct as *mut ::core::ffi::c_char,
            *argv,
            49 as usize,
        );
        acct[49 as usize] = '\0' as ::core::ffi::c_char;
        while argc > 1 as ::core::ffi::c_int {
            argc -= 1;
            argv = argv.offset(1);
            strncat(
                &raw mut acct as *mut ::core::ffi::c_char,
                *argv,
                (49 as usize).wrapping_sub(
                    strlen(&raw mut acct as *mut ::core::ffi::c_char) as usize,
                ),
            );
        }
        ap = &raw mut acct as *mut ::core::ffi::c_char;
    } else {
        ap = getpass(b"Account:\0" as *const u8 as *const ::core::ffi::c_char);
    }
    command(b"ACCT %s\0" as *const u8 as *const ::core::ffi::c_char, ap);
}
#[no_mangle]
pub unsafe extern "C" fn setcase(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    mcase = (mcase == 0) as i32;
    printf(b"Case mapping %s.\n\0".as_ptr() as *const i8, onoff(mcase));
    let _ = fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = mcase;
}
#[no_mangle]
pub unsafe extern "C" fn setcr(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    crflag = (crflag == 0) as core::ffi::c_int;
    printf(
        b"Carriage Return stripping %s.\n\0".as_ptr() as *const core::ffi::c_char,
        onoff(crflag),
    );
    fflush(__iob_func().add(1));
    code = crflag;
}
#[no_mangle]
pub unsafe extern "C" fn setntrans(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc == 1 {
        ntflag = 0;
        printf(b"Ntrans off.\n\0".as_ptr() as *const i8);
        let _ = fflush(__iob_func().add(1));
        code = ntflag;
        return;
    }
    ntflag += 1;
    code = ntflag;
    let _ = strncpy(
        core::ptr::addr_of_mut!(ntin) as *mut i8,
        *argv.offset(1 as isize),
        16 as usize,
    );
    ntin[16 as usize] = b'\0' as i8;
    if argc == 2 {
        ntout[0 as usize] = b'\0' as i8;
        return;
    }
    let _ = strncpy(
        core::ptr::addr_of_mut!(ntout) as *mut i8,
        *argv.offset(2 as isize),
        16 as usize,
    );
    ntout[16 as usize] = b'\0' as i8;
}
#[no_mangle]
pub unsafe extern "C" fn dotrans(name: *const core::ffi::c_char) -> *const core::ffi::c_char {
    static mut new: [core::ffi::c_char; (MAXPATHLEN as _)] = [0; (MAXPATHLEN as _)];
    let mut cp1: *const core::ffi::c_char = core::ptr::null();
    let mut cp2: *mut core::ffi::c_char = core::ptr::addr_of_mut!(new) as *mut core::ffi::c_char;
    let mut i: core::ffi::c_int = 0;
    let mut ostop: core::ffi::c_int = 0;
    let mut found: core::ffi::c_int = 0;

    ostop = 0;
    while ostop < 16
        && *(core::ptr::addr_of_mut!(ntout) as *mut core::ffi::c_char).offset(ostop as isize)
            as core::ffi::c_int
            != 0
    {
        ostop += 1;
    }
    cp1 = name;
    while *cp1 != 0 {
        found = 0;
        i = 0;
        while i < 16
            && *(core::ptr::addr_of_mut!(ntin) as *mut core::ffi::c_char).offset(i as isize)
                as core::ffi::c_int
                != 0
        {
            if *cp1 as core::ffi::c_int
                == *(core::ptr::addr_of_mut!(ntin) as *mut core::ffi::c_char).offset(i as isize)
                    as core::ffi::c_int
            {
                found += 1;
                if i < ostop {
                    *cp2 = *(core::ptr::addr_of_mut!(ntout) as *mut core::ffi::c_char)
                        .offset(i as isize);
                    cp2 = cp2.offset(1);
                }
                break;
            }
            i += 1;
        }
        if found == 0 {
            *cp2 = *cp1;
            cp2 = cp2.offset(1);
        }
        cp1 = cp1.offset(1);
    }
    *cp2 = '\0' as core::ffi::c_char;
    core::ptr::addr_of_mut!(new) as *mut core::ffi::c_char as *const core::ffi::c_char
}
#[no_mangle]
pub unsafe extern "C" fn setpassive(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    passivemode = (passivemode == 0) as ::core::ffi::c_int;
    printf(
        b"Passive mode %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
        onoff(passivemode),
    );
    let _ = fflush(
        (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
    );
    code = passivemode;
}
#[no_mangle]
pub unsafe extern "C" fn setnmap(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if argc == 1 {
        mapflag = 0;
        printf(b"Nmap off.\n\0".as_ptr() as *const ::core::ffi::c_char);
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
        );
        code = mapflag;
        return;
    }
    if argc < 3 {
        let _ = strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(mapout) \0".as_ptr() as *const ::core::ffi::c_char);
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
        );
        let _ = gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char)
                .offset(strlen(core::ptr::addr_of_mut!(line) as *const ::core::ffi::c_char) as isize),
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 3 {
        printf(
            b"Usage: %s [mapin mapout]\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        let _ = fflush(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
        );
        code = -1;
        return;
    }
    mapflag = 1;
    code = 1;
    cp = index(altarg, ' ' as ::core::ffi::c_int);
    if proxy != 0 {
        loop {
            cp = cp.offset(1);
            if !(*cp as ::core::ffi::c_int == ' ' as ::core::ffi::c_int) {
                break;
            }
        }
        altarg = cp;
        cp = index(altarg, ' ' as ::core::ffi::c_int);
    }
    *cp = '\0' as ::core::ffi::c_char;
    let _ = strncpy(
        core::ptr::addr_of_mut!(mapin) as *mut ::core::ffi::c_char,
        altarg,
        (MAXPATHLEN - 1) as usize,
    );
    loop {
        cp = cp.offset(1);
        if !(*cp as ::core::ffi::c_int == ' ' as ::core::ffi::c_int) {
            break;
        }
    }
    let _ = strncpy(
        core::ptr::addr_of_mut!(mapout) as *mut ::core::ffi::c_char,
        cp,
        (MAXPATHLEN - 1) as usize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn domap(name: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    static mut new: [::core::ffi::c_char; (MAXPATHLEN as _)] = [0; (MAXPATHLEN as _)];
    let mut cp1: *const ::core::ffi::c_char = name;
    let mut cpn: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut cp2: *mut ::core::ffi::c_char = &raw mut mapin as *mut ::core::ffi::c_char;
    let mut tp: [*const ::core::ffi::c_char; 9] = [::core::ptr::null(); 9];
    let mut te: [*const ::core::ffi::c_char; 9] = [::core::ptr::null(); 9];
    let mut i: ::core::ffi::c_int;
    let mut toks: [::core::ffi::c_int; 9] = [0; 9];
    let mut toknum: ::core::ffi::c_int = 0;
    let mut match_0: ::core::ffi::c_int = 1;

    i = 0;
    while i < 9 {
        toks[i as usize] = 0;
        i += 1;
    }
    while match_0 != 0 && *cp1 as ::core::ffi::c_int != 0 && *cp2 as ::core::ffi::c_int != 0 {
        match *cp2 as ::core::ffi::c_int {
            92 => {
                cp2 = cp2.offset(1);
                if *cp2 as ::core::ffi::c_int != *cp1 as ::core::ffi::c_int {
                    match_0 = 0;
                }
            }
            36 => {
                if *cp2.offset(1 as isize) as ::core::ffi::c_int >= '1' as ::core::ffi::c_int
                    && (*cp2 as ::core::ffi::c_int + (1)) <= '9' as ::core::ffi::c_int
                {
                    cp2 = cp2.offset(1);
                    if *cp1 as ::core::ffi::c_int
                        != *cp2.offset(1 as isize) as ::core::ffi::c_int
                    {
                        toknum = *cp2 as ::core::ffi::c_int - '1' as ::core::ffi::c_int;
                        toks[toknum as usize] += 1;
                        tp[toknum as usize] = cp1;
                        loop {
                            cp1 = cp1.offset(1);
                            if !(*cp1 as ::core::ffi::c_int != 0
                                && *cp2.offset(1 as isize) as ::core::ffi::c_int
                                    != *cp1 as ::core::ffi::c_int)
                            {
                                break;
                            }
                        }
                        te[toknum as usize] = cp1;
                    }
                    cp2 = cp2.offset(1);
                } else {
                    if *cp2 as ::core::ffi::c_int != *cp1 as ::core::ffi::c_int {
                        match_0 = 0;
                    }
                }
            }
            _ => {
                if *cp2 as ::core::ffi::c_int != *cp1 as ::core::ffi::c_int {
                    match_0 = 0;
                }
            }
        }
        if match_0 != 0 && *cp1 as ::core::ffi::c_int != 0 {
            cp1 = cp1.offset(1);
        }
        if match_0 != 0 && *cp2 as ::core::ffi::c_int != 0 {
            cp2 = cp2.offset(1);
        }
    }
    if match_0 == 0 && *cp1 as ::core::ffi::c_int != 0 {
        toks[toknum as usize] = 0;
    }

    cpn = &raw mut new as *mut ::core::ffi::c_char;
    *cpn = 0;
    cp2 = &raw mut mapout as *mut ::core::ffi::c_char;
    while *cp2 as ::core::ffi::c_int != 0 {
        match_0 = 0;
        let mut at: u8 = match *cp2 as ::core::ffi::c_int {
            92 => 0,
            91 => 1,
            36 => 2,
            _ => 3,
        };
        'sw: loop {
            match at {
                0 => {
                    if *cp2.offset(1 as isize) != 0 {
                        cp2 = cp2.offset(1);
                        *cpn = *cp2;
                        cpn = cpn.offset(1);
                    }
                    break 'sw;
                }
                1 => {
                    'LOOP: loop {
                        cp2 = cp2.offset(1);
                        if *cp2 as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                            && ::core::ffi::c_int::from(isdigit(
                                *cp2.offset(1 as isize) as ::core::ffi::c_int,
                            ) != 0) != 0
                        {
                            cp2 = cp2.offset(1);
                            if *cp2 as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                                let mut cp3: *const ::core::ffi::c_char = name;
                                while *cp3 != 0 {
                                    *cpn = *cp3;
                                    cpn = cpn.offset(1);
                                    cp3 = cp3.offset(1);
                                }
                                match_0 = 1;
                            } else {
                                toknum = *cp2 as ::core::ffi::c_int - '1' as ::core::ffi::c_int;
                                if toks[toknum as usize] != 0 {
                                    let mut cp3: *const ::core::ffi::c_char = tp[toknum as usize];
                                    while cp3 != te[toknum as usize] {
                                        *cpn = *cp3;
                                        cpn = cpn.offset(1);
                                        cp3 = cp3.offset(1);
                                    }
                                    match_0 = 1;
                                }
                            }
                        } else {
                            while *cp2 as ::core::ffi::c_int != 0
                                && *cp2 as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                                && *cp2 as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                            {
                                if *cp2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                                    cp2 = cp2.offset(1);
                                } else if *cp2 as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                                    && isdigit(
                                        *cp2.offset(1 as isize) as ::core::ffi::c_int,
                                    ) != 0
                                {
                                    cp2 = cp2.offset(1);
                                    if *cp2 as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                                        let mut cp3: *const ::core::ffi::c_char = name;
                                        while *cp3 != 0 {
                                            *cpn = *cp3;
                                            cpn = cpn.offset(1);
                                            cp3 = cp3.offset(1);
                                        }
                                    } else {
                                        toknum = *cp2 as ::core::ffi::c_int - '1' as ::core::ffi::c_int;
                                        if toks[toknum as usize] != 0 {
                                            let mut cp3: *const ::core::ffi::c_char = tp[toknum as usize];
                                            while cp3 != te[toknum as usize] {
                                                *cpn = *cp3;
                                                cpn = cpn.offset(1);
                                                cp3 = cp3.offset(1);
                                            }
                                        }
                                    }
                                } else if *cp2 != 0 {
                                    *cpn = *cp2;
                                    cpn = cpn.offset(1);
                                    cp2 = cp2.offset(1);
                                }
                            }
                            if *cp2 == 0 {
                                printf(
                                    b"nmap: unbalanced brackets\n\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                let _ = fflush(__iob_func().add(1));
                                return name;
                            }
                            match_0 = 1;
                            cp2 = cp2.offset(-1);
                        }
                        if match_0 != 0 {
                            loop {
                                cp2 = cp2.offset(1);
                                if !(*cp2 as ::core::ffi::c_int != 0
                                    && *cp2 as ::core::ffi::c_int != ']' as ::core::ffi::c_int)
                                {
                                    break;
                                }
                                if *cp2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                    && *cp2.offset(1 as isize) != 0
                                {
                                    cp2 = cp2.offset(1);
                                }
                            }
                            if *cp2 == 0 {
                                printf(
                                    b"nmap: unbalanced brackets\n\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                let _ = fflush(__iob_func().add(1));
                                return name;
                            }
                            break 'sw;
                        }
                        cp2 = cp2.offset(1);
                        match *cp2 as ::core::ffi::c_int {
                            44 => {
                                continue 'LOOP;
                            }
                            93 => {
                                break 'sw;
                            }
                            _ => {
                                cp2 = cp2.offset(-1);
                                at = 2;
                                continue 'sw;
                            }
                        }
                    }
                }
                2 => {
                    'LOOP: loop {
                        if isdigit(
                            *cp2.offset(1 as isize) as ::core::ffi::c_int,
                        ) != 0
                        {
                            cp2 = cp2.offset(1);
                            if *cp2 as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                                let mut cp3: *const ::core::ffi::c_char = name;
                                while *cp3 != 0 {
                                    *cpn = *cp3;
                                    cpn = cpn.offset(1);
                                    cp3 = cp3.offset(1);
                                }
                            } else {
                                toknum = *cp2 as ::core::ffi::c_int - '1' as ::core::ffi::c_int;
                                if toks[toknum as usize] != 0 {
                                    let mut cp3: *const ::core::ffi::c_char = tp[toknum as usize];
                                    while cp3 != te[toknum as usize] {
                                        *cpn = *cp3;
                                        cpn = cpn.offset(1);
                                        cp3 = cp3.offset(1);
                                    }
                                }
                            }
                            break 'sw;
                        }
                        *cpn = *cp2;
                        cpn = cpn.offset(1);
                        break 'sw;
                    }
                }
                _ => {
                    *cpn = *cp2;
                    cpn = cpn.offset(1);
                    break 'sw;
                }
            }
        }
        cp2 = cp2.offset(1);
    }
    *cpn = 0;
    if *(&raw mut new as *mut ::core::ffi::c_char) == 0 {
        return name;
    }
    return &raw mut new as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn setsunique(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let _ = (argc, argv);
    sunique = (sunique == 0) as core::ffi::c_int;
    printf(
        b"Store unique %s.\n\0".as_ptr() as *const core::ffi::c_char,
        onoff(sunique),
    );
    let _ = fflush(__iob_func().offset(1) as *mut FILE);
    code = sunique;
}
#[no_mangle]
pub unsafe extern "C" fn setrunique(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    runique = (runique == 0) as ::core::ffi::c_int;
    printf(
        b"Receive unique %s.\n\0".as_ptr() as *const ::core::ffi::c_char,
        onoff(runique),
    );
    fflush(
        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
    );
    code = runique;
}
#[no_mangle]
pub unsafe extern "C" fn cdup(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if command(b"CDUP\0".as_ptr() as *const ::core::ffi::c_char) == (ERROR as i32)
        && code == 500
    {
        if verbose != 0 {
            printf(
                b"CDUP command not recognized, trying XCUP\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            fflush(
                (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
            );
        }
        command(b"XCUP\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn restart(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    if argc != 2 {
        printf(b"restart: offset not specified\n\0".as_ptr() as *const i8);
    } else {
        restart_point = atol(*argv.offset(1 as isize)) as off_t;
        printf(
            b"restarting at %ld. %s\n\0".as_ptr() as *const i8,
            restart_point,
            b"execute get, put or append to initiate transfer\0".as_ptr() as *const i8,
        );
    }
    fflush(__iob_func().add(1));
}
#[no_mangle]
pub unsafe extern "C" fn syst(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let _ = argc;
    let _ = argv;
    let _ = command(b"SYST\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn macdef(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut tmp: *mut c_char = core::ptr::null_mut();
    let mut c: c_int = 0;

    if macnum == 16 {
        printf(b"Limit of 16 macros have already been defined\n\0".as_ptr() as *const c_char);
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    if argc < 2 {
        strcat(core::ptr::addr_of_mut!(line) as *mut c_char, b" \0".as_ptr() as *const c_char);
        printf(b"(macro name) \0".as_ptr() as *const c_char);
        fflush(__iob_func().add(1));
        gets(
            (core::ptr::addr_of_mut!(line) as *mut c_char).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut c_char) as isize,
            ),
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const c_char;
    }
    if argc != 2 {
        printf(
            b"Usage: %s macro_name\n\0".as_ptr() as *const c_char,
            *argv.offset(0 as isize),
        );
        fflush(__iob_func().add(1));
        code = -1;
        return;
    }
    if interactive != 0 {
        printf(
            b"Enter macro line by line, terminating it with a null line\n\0".as_ptr() as *const c_char,
        );
        fflush(__iob_func().add(1));
    }
    strncpy(
        core::ptr::addr_of_mut!((*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(macnum as isize)).mac_name)
            as *mut c_char,
        *argv.offset(1 as isize),
        8 as usize,
    );
    if macnum == 0 {
        (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(macnum as isize)).mac_start =
            core::ptr::addr_of_mut!(macbuf) as *mut c_char;
    } else {
        (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(macnum as isize)).mac_start =
            (*(core::ptr::addr_of_mut!(macros) as *mut macel)
                .offset((macnum - 1) as isize))
                .mac_end
                .offset(1 as isize);
    }
    tmp = (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(macnum as isize)).mac_start;
    while tmp
        != (core::ptr::addr_of_mut!(macbuf) as *mut c_char).offset(4096 as isize)
    {
        c = getchar();
        if c == EOF {
            printf(b"macdef:end of file encountered\n\0".as_ptr() as *const c_char);
            fflush(__iob_func().add(1));
            code = -1;
            return;
        }
        *tmp = c as c_char;
        if *tmp as c_int == '\n' as c_int {
            if tmp
                == (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(macnum as isize)).mac_start
            {
                let fresh45 = macnum;
                macnum = macnum + 1;
                (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(fresh45 as isize)).mac_end = tmp;
                code = 0;
                return;
            }
            if *tmp.offset(-(1 as isize)) as c_int == '\0' as c_int {
                let fresh46 = macnum;
                macnum = macnum + 1;
                (*(core::ptr::addr_of_mut!(macros) as *mut macel).offset(fresh46 as isize)).mac_end =
                    tmp.offset(-(1 as isize));
                code = 0;
                return;
            }
            *tmp = '\0' as c_char;
        }
        tmp = tmp.offset(1);
    }
    while 1 != 0 {
        loop {
            c = getchar();
            if !(c != '\n' as c_int && c != EOF) {
                break;
            }
        }
        if c == EOF || getchar() == '\n' as c_int {
            printf(b"Macro not defined - 4k buffer exceeded\n\0".as_ptr() as *const c_char);
            fflush(__iob_func().add(1));
            code = -1;
            return;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn sizecmd(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    if argc < 2 {
        strcat(core::ptr::addr_of_mut!(line) as *mut i8, b" \0".as_ptr() as *const i8);
        printf(b"(filename) \0".as_ptr() as *const i8);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets(
            (core::ptr::addr_of_mut!(line) as *mut i8).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut i8) as isize,
            ) as *mut i8,
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const i8;
    }
    if argc < 2 {
        printf(
            b"usage:%s filename\n\0".as_ptr() as *const i8,
            *argv.offset(0 as isize),
        );
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    command(
        b"SIZE %s\0".as_ptr() as *const i8,
        *argv.offset(1 as isize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn modtime(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut overbose: ::core::ffi::c_int = 0;
    if argc < 2 {
        strcat(
            core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char,
            b" \0".as_ptr() as *const ::core::ffi::c_char,
        );
        printf(b"(filename) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
        );
        gets(
            (core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char).offset(
                strlen(core::ptr::addr_of_mut!(line) as *mut ::core::ffi::c_char)
                    as isize,
            ) as *mut ::core::ffi::c_char,
        );
        makeargv();
        argc = margc;
        argv = core::ptr::addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(
            b"usage:%s filename\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0 as isize),
        );
        fflush(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
        );
        code = -1;
        return;
    }
    overbose = verbose;
    if debug == 0 {
        verbose = -1;
    }
    if command(
        b"MDTM %s\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(1 as isize),
    ) == (COMPLETE as i32)
    {
        let mut yy: ::core::ffi::c_int = 0;
        let mut mo: ::core::ffi::c_int = 0;
        let mut day: ::core::ffi::c_int = 0;
        let mut hour: ::core::ffi::c_int = 0;
        let mut min: ::core::ffi::c_int = 0;
        let mut sec: ::core::ffi::c_int = 0;
        sscanf(
            core::ptr::addr_of_mut!(reply_string) as *mut ::core::ffi::c_char,
            b"%*s %04d%02d%02d%02d%02d%02d\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(yy),
            core::ptr::addr_of_mut!(mo),
            core::ptr::addr_of_mut!(day),
            core::ptr::addr_of_mut!(hour),
            core::ptr::addr_of_mut!(min),
            core::ptr::addr_of_mut!(sec),
        );
        /* might want to print this in local time */
        printf(
            b"%s\t%02d/%02d/%04d %02d:%02d:%02d GMT\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(1 as isize),
            mo,
            day,
            yy,
            hour,
            min,
            sec,
        );
    } else {
        printf(
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(reply_string) as *mut ::core::ffi::c_char,
        );
    }
    verbose = overbose;
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
}
#[no_mangle]
pub unsafe extern "C" fn rmtstatus(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let _ = command(
        if argc > 1 {
            b"STAT %s\0".as_ptr() as *const i8
        } else {
            b"STAT\0".as_ptr() as *const i8
        },
        *argv.add(1),
    );
}
#[no_mangle]
pub unsafe extern "C" fn newer(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    if getit(argc, argv, -1, b"w\0".as_ptr() as *const i8) != 0 {
        printf(
            b"Local file \"%s\" is newer than remote file \"%s\"\n\0".as_ptr() as *const i8,
            *argv.offset(1 as isize),
            *argv.offset(2 as isize),
        );
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    }
}

// uses: crate::ftp::main::{line, margc, margv, code, mflag, interactive, fromatty, mcase, ntflag,
//                          mapflag, sunique, doglob, proxy, connected, proxflag, globerr, getcmd,
//                          makeargv}
//       crate::ftp::cmds::{mname, remglob, confirm, dotrans, domap, globulize}  (same module)
//       crate::ftp::fake::{glob, blkfree}
//       crate::ftp::ftp::pswitch
//       crate::prelude::{strcat, strcmp, strlen, printf, fflush, gets, free, islower, isupper,
//                        __iob_func, FILE, cmd, MAXPATHLEN, command}
// needs (NOT yet declared anywhere in ftp-rust; ftp.c's, prototypes.h:39-41):
//       fn sendrequest(cmd: *const c_char, local: *const c_char, remote: *const c_char,
//                      printnames: c_int);
//       fn recvrequest(cmd: *const c_char, local: *const c_char, remote: *const c_char,
//                      mode: *const c_char, printnames: c_int);

// hand-ported: the machine translation lowered `goto usage` into a state loop with placeholder
// tokens; this lowers it structurally (a labeled block) with the C's exact semantics.
#[no_mangle]
pub unsafe extern "C" fn put(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let cmd: *const ::core::ffi::c_char;
    let mut loc: ::core::ffi::c_int = 0;
    let oldargv1: *const ::core::ffi::c_char;
    let oldargv2: *const ::core::ffi::c_char;

    // `goto usage` (cmds.c:299,317): breaking out of this block runs the usage tail below.
    'usage: {
        if argc == 2 {
            argc += 1;
            *argv.offset(2) = *argv.offset(1);
            loc += 1;
        }
        if argc < 2 {
            strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
            printf(b"(local-file) \0".as_ptr() as *const ::core::ffi::c_char);
            fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
            gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
                .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
            makeargv();
            argc = margc;
            argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
        }
        if argc < 2 {
            break 'usage;
        }
        if argc < 3 {
            strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
            printf(b"(remote-file) \0".as_ptr() as *const ::core::ffi::c_char);
            fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
            gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
                .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
            makeargv();
            argc = margc;
            argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
        }
        if argc < 3 {
            break 'usage;
        }
        oldargv1 = *argv.offset(1);
        oldargv2 = *argv.offset(2);
        if globulize(argv.offset(1)) == 0 {
            code = -1;
            return;
        }
        // If "globulize" modifies argv[1], and argv[2] is a copy of
        // the old argv[1], make it a copy of the new argv[1].
        if *argv.offset(1) != oldargv1 && *argv.offset(2) == oldargv1 {
            *argv.offset(2) = *argv.offset(1);
        }
        cmd = if *(*argv.offset(0)) as ::core::ffi::c_int == 'a' as i32 {
            b"APPE\0".as_ptr() as *const ::core::ffi::c_char
        } else if sunique != 0 {
            b"STOU\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"STOR\0".as_ptr() as *const ::core::ffi::c_char
        };
        if loc != 0 && ntflag != 0 {
            *argv.offset(2) = dotrans(*argv.offset(2));
        }
        if loc != 0 && mapflag != 0 {
            *argv.offset(2) = domap(*argv.offset(2));
        }
        sendrequest(
            cmd,
            *argv.offset(1),
            *argv.offset(2),
            (*argv.offset(1) != oldargv1 || *argv.offset(2) != oldargv2) as ::core::ffi::c_int,
        );
        return;
    }
    // usage:
    printf(
        b"usage:%s local-file remote-file\n\0".as_ptr() as *const ::core::ffi::c_char,
        *argv.offset(0),
    );
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    code = -1;
}

// hand-ported: setjmp(jabort) is dead (mabort is never installed), and the C's mcase path reads
// an uninitialised tmpbuf; both need judgment the machine translation cannot make.
#[no_mangle]
pub unsafe extern "C" fn mput(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut ointer: ::core::ffi::c_int;
    let mut tp: *const ::core::ffi::c_char;

    if argc < 2 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(local-files) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(b"usage:%s local-files\n\0".as_ptr() as *const ::core::ffi::c_char, *argv.offset(0));
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    mname = *argv.offset(0);
    mflag = 1;
    // `(void) setjmp(jabort);` dropped: mabort is never installed (signal() commented out,
    // mabort under #if 0), so nothing ever longjmps here and only the 0-return path exists.
    if proxy != 0 {
        let mut cp: *const ::core::ffi::c_char;
        let mut tp2: *mut ::core::ffi::c_char;
        let mut tmpbuf: [::core::ffi::c_char; MAXPATHLEN as usize] = [0; MAXPATHLEN as usize];

        loop {
            cp = remglob(argv, 0);
            if cp.is_null() {
                break;
            }
            if *cp as ::core::ffi::c_int == 0 {
                mflag = 0;
                continue;
            }
            if mflag != 0 && confirm(*argv.offset(0), cp) != 0 {
                tp = cp;
                if mcase != 0 {
                    while *tp as ::core::ffi::c_int != 0 && islower(*tp as i32) == 0 {
                        tp = tp.offset(1);
                    }
                    if *tp as ::core::ffi::c_int == 0 {
                        tp = cp;
                        tp2 = tmpbuf.as_mut_ptr();
                        loop {
                            *tp2 = *tp;
                            if *tp2 as ::core::ffi::c_int == 0 {
                                break;
                            }
                            if isupper(*tp2 as i32) != 0 {
                                *tp2 = ('a' as i32 + *tp2 as i32 - 'A' as i32) as ::core::ffi::c_char;
                            }
                            tp = tp.offset(1);
                            tp2 = tp2.offset(1);
                        }
                        // Diverges from the C (cmds.c:389): the C sets `tp = tmpbuf` even when
                        // the name already held a lowercase letter and the copy loop above never
                        // ran, sending an uninitialised stack buffer as the remote name. Use
                        // tmpbuf only when it was actually filled; otherwise tp falls back to cp.
                        tp = tmpbuf.as_ptr();
                    } else {
                        tp = cp;
                    }
                }
                if ntflag != 0 {
                    tp = dotrans(tp);
                }
                if mapflag != 0 {
                    tp = domap(tp);
                }
                sendrequest(
                    if sunique != 0 {
                        b"STOU\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"STOR\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    cp,
                    tp,
                    (cp != tp || interactive == 0) as ::core::ffi::c_int,
                );
                if mflag == 0 && fromatty != 0 {
                    ointer = interactive;
                    interactive = 1;
                    if confirm(
                        b"Continue with\0".as_ptr() as *const ::core::ffi::c_char,
                        b"mput\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0
                    {
                        mflag += 1;
                    }
                    interactive = ointer;
                }
            }
        }
        mflag = 0;
        return;
    }
    let mut i: ::core::ffi::c_int = 1;
    while i < argc {
        let mut cpp: *mut *mut ::core::ffi::c_char;
        let gargs: *mut *mut ::core::ffi::c_char;

        if doglob == 0 {
            if mflag != 0 && confirm(*argv.offset(0), *argv.offset(i as isize)) != 0 {
                tp = if ntflag != 0 { dotrans(*argv.offset(i as isize)) } else { *argv.offset(i as isize) };
                tp = if mapflag != 0 { domap(tp) } else { tp };
                sendrequest(
                    if sunique != 0 {
                        b"STOU\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"STOR\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    *argv.offset(i as isize),
                    tp,
                    (tp != *argv.offset(i as isize) || interactive == 0) as ::core::ffi::c_int,
                );
                if mflag == 0 && fromatty != 0 {
                    ointer = interactive;
                    interactive = 1;
                    if confirm(
                        b"Continue with\0".as_ptr() as *const ::core::ffi::c_char,
                        b"mput\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0
                    {
                        mflag += 1;
                    }
                    interactive = ointer;
                }
            }
            i += 1;
            continue;
        }
        gargs = glob(*argv.offset(i as isize));
        if !globerr.is_null() {
            printf(b"%s\n\0".as_ptr() as *const ::core::ffi::c_char, globerr);
            fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
            if !gargs.is_null() {
                blkfree(gargs);
                free(gargs as *mut ::core::ffi::c_void);
            }
            i += 1;
            continue;
        }
        cpp = gargs;
        while !cpp.is_null() && !(*cpp).is_null() {
            if mflag != 0 && confirm(*argv.offset(0), *cpp) != 0 {
                tp = if ntflag != 0 { dotrans(*cpp) } else { *cpp as *const ::core::ffi::c_char };
                tp = if mapflag != 0 { domap(tp) } else { tp };
                sendrequest(
                    if sunique != 0 {
                        b"STOU\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"STOR\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    *cpp,
                    tp,
                    (*cpp as *const ::core::ffi::c_char != tp || interactive == 0) as ::core::ffi::c_int,
                );
                if mflag == 0 && fromatty != 0 {
                    ointer = interactive;
                    interactive = 1;
                    if confirm(
                        b"Continue with\0".as_ptr() as *const ::core::ffi::c_char,
                        b"mput\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0
                    {
                        mflag += 1;
                    }
                    interactive = ointer;
                }
            }
            cpp = cpp.offset(1);
        }
        if !gargs.is_null() {
            blkfree(gargs);
            free(gargs as *mut ::core::ffi::c_void);
        }
        i += 1;
    }
    mflag = 0;
}

// hand-ported: setjmp(jabort) is dead (mabort is never installed), and the C's mcase path reads
// an uninitialised tmpbuf; both need judgment the machine translation cannot make.
#[no_mangle]
pub unsafe extern "C" fn mget(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut cp: *const ::core::ffi::c_char;
    let mut tp: *const ::core::ffi::c_char;
    let mut tp2: *mut ::core::ffi::c_char;
    let mut tmpbuf: [::core::ffi::c_char; MAXPATHLEN as usize] = [0; MAXPATHLEN as usize];
    let mut ointer: ::core::ffi::c_int;

    if argc < 2 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(remote-files) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(b"usage:%s remote-files\n\0".as_ptr() as *const ::core::ffi::c_char, *argv.offset(0));
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    mname = *argv.offset(0);
    mflag = 1;
    // `(void) setjmp(jabort);` dropped: mabort is never installed (signal() commented out,
    // mabort under #if 0), so nothing ever longjmps here and only the 0-return path exists.
    loop {
        cp = remglob(argv, proxy);
        if cp.is_null() {
            break;
        }
        if *cp as ::core::ffi::c_int == 0 {
            mflag = 0;
            continue;
        }
        if mflag != 0 && confirm(*argv.offset(0), cp) != 0 {
            tp = cp;
            if mcase != 0 {
                while *tp as ::core::ffi::c_int != 0 && islower(*tp as i32) == 0 {
                    tp = tp.offset(1);
                }
                if *tp as ::core::ffi::c_int == 0 {
                    tp = cp;
                    tp2 = tmpbuf.as_mut_ptr();
                    loop {
                        *tp2 = *tp;
                        if *tp2 as ::core::ffi::c_int == 0 {
                            break;
                        }
                        if isupper(*tp2 as i32) != 0 {
                            *tp2 = ('a' as i32 + *tp2 as i32 - 'A' as i32) as ::core::ffi::c_char;
                        }
                        tp = tp.offset(1);
                        tp2 = tp2.offset(1);
                    }
                    // Diverges from the C (cmds.c:686): the C sets `tp = tmpbuf` even when the
                    // name already held a lowercase letter and the copy loop above never ran,
                    // using an uninitialised stack buffer as the local file name. Use tmpbuf only
                    // when it was actually filled; otherwise tp falls back to cp.
                    tp = tmpbuf.as_ptr();
                } else {
                    tp = cp;
                }
            }
            if ntflag != 0 {
                tp = dotrans(tp);
            }
            if mapflag != 0 {
                tp = domap(tp);
            }
            recvrequest(
                b"RETR\0".as_ptr() as *const ::core::ffi::c_char,
                tp,
                cp,
                b"w\0".as_ptr() as *const ::core::ffi::c_char,
                (tp != cp || interactive == 0) as ::core::ffi::c_int,
            );
            if mflag == 0 && fromatty != 0 {
                ointer = interactive;
                interactive = 1;
                if confirm(
                    b"Continue with\0".as_ptr() as *const ::core::ffi::c_char,
                    b"mget\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0
                {
                    mflag += 1;
                }
                interactive = ointer;
            }
        }
    }
    mflag = 0;
}

// hand-ported: setjmp(jabort) is dead (mabort is never installed); only the 0-return path kept.
#[no_mangle]
pub unsafe extern "C" fn mdelete(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let mut cp: *const ::core::ffi::c_char;
    let mut ointer: ::core::ffi::c_int;

    if argc < 2 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(remote-files) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(b"usage:%s remote-files\n\0".as_ptr() as *const ::core::ffi::c_char, *argv.offset(0));
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    mname = *argv.offset(0);
    mflag = 1;
    // `(void) setjmp(jabort);` dropped: mabort is never installed (signal() commented out,
    // mabort under #if 0), so nothing ever longjmps here and only the 0-return path exists.
    loop {
        cp = remglob(argv, 0);
        if cp.is_null() {
            break;
        }
        if *cp as ::core::ffi::c_int == 0 {
            mflag = 0;
            continue;
        }
        if mflag != 0 && confirm(*argv.offset(0), cp) != 0 {
            command(b"DELE %s\0".as_ptr() as *const ::core::ffi::c_char, cp);
            if mflag == 0 && fromatty != 0 {
                ointer = interactive;
                interactive = 1;
                if confirm(
                    b"Continue with\0".as_ptr() as *const ::core::ffi::c_char,
                    b"mdelete\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0
                {
                    mflag += 1;
                }
                interactive = ointer;
            }
        }
    }
    mflag = 0;
}

// hand-ported: setjmp(jabort) is dead (mabort is never installed), and the C passes an
// unterminated 1-char mode string to fopen.
#[no_mangle]
pub unsafe extern "C" fn mls(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let cmd: *const ::core::ffi::c_char;
    let mut dest: *const ::core::ffi::c_char;
    // Diverges from the C (cmds.c:1171): `char mode[1]` is handed to recvrequest -> fopen as a
    // mode STRING with no NUL terminator; fopen reads past it. Two bytes, NUL-terminated.
    let mut mode_buf: [::core::ffi::c_char; 2] = [0; 2];
    let mut ointer: ::core::ffi::c_int;

    if argc < 2 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(remote-files) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 3 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(local-file) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 3 {
        printf(
            b"usage:%s remote-files local-file\n\0".as_ptr() as *const ::core::ffi::c_char,
            *argv.offset(0),
        );
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    dest = *argv.offset((argc - 1) as isize);
    *argv.offset((argc - 1) as isize) = ::core::ptr::null();
    if strcmp(dest, b"-\0".as_ptr() as *const ::core::ffi::c_char) != 0
        && *dest as ::core::ffi::c_int != '|' as i32
    {
        if globulize(&mut dest) == 0
            || confirm(b"output to local-file:\0".as_ptr() as *const ::core::ffi::c_char, dest) == 0
        {
            code = -1;
            return;
        }
    }
    cmd = if *(*argv.offset(0)).offset(1) as ::core::ffi::c_int == 'l' as i32 {
        b"NLST\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"LIST\0".as_ptr() as *const ::core::ffi::c_char
    };
    mname = *argv.offset(0);
    mflag = 1;
    // `(void) setjmp(jabort);` dropped: mabort is never installed (signal() commented out,
    // mabort under #if 0), so nothing ever longjmps here and only the 0-return path exists.
    let mut i: ::core::ffi::c_int = 1;
    while mflag != 0 && i < argc - 1 {
        mode_buf[0] = (if i == 1 { 'w' } else { 'a' }) as ::core::ffi::c_char;
        recvrequest(cmd, dest, *argv.offset(i as isize), mode_buf.as_ptr(), 0);
        if mflag == 0 && fromatty != 0 {
            ointer = interactive;
            interactive = 1;
            if confirm(b"Continue with\0".as_ptr() as *const ::core::ffi::c_char, *argv.offset(0)) != 0 {
                mflag += 1;
            }
            interactive = ointer;
        }
        i += 1;
    }
    mflag = 0;
}

// hand-ported: the `if (setjmp(abortprox))` branch is dead (proxabort is never installed) and the
// `(struct cmd *)-1` sentinel compare plus the handler call through cmdtab need care.
#[no_mangle]
pub unsafe extern "C" fn doproxy(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let c: *mut cmd;

    if argc < 2 {
        strcat(addr_of_mut!(line) as *mut ::core::ffi::c_char, b" \0".as_ptr() as *const ::core::ffi::c_char);
        printf(b"(command) \0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        gets((addr_of_mut!(line) as *mut ::core::ffi::c_char)
            .add(strlen(addr_of_mut!(line) as *const ::core::ffi::c_char)));
        makeargv();
        argc = margc;
        argv = addr_of_mut!(margv) as *mut *const ::core::ffi::c_char;
    }
    if argc < 2 {
        printf(b"usage:%s command\n\0".as_ptr() as *const ::core::ffi::c_char, *argv.offset(0));
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    c = getcmd(*argv.offset(1));
    if c as isize == -1 {
        printf(b"?Ambiguous command\n\0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    if c.is_null() {
        printf(b"?Invalid command\n\0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    if (*c).c_proxy as ::core::ffi::c_int == 0 {
        printf(b"?Invalid proxy command\n\0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        code = -1;
        return;
    }
    // `if (setjmp(abortprox)) { code = -1; return; }` dropped: proxabort is never installed
    // (signal() commented out, proxabort under #if 0), so setjmp only ever returns 0 here.
    pswitch(1);
    if (*c).c_conn as ::core::ffi::c_int != 0 && connected == 0 {
        printf(b"Not connected\n\0".as_ptr() as *const ::core::ffi::c_char);
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        pswitch(0);
        code = -1;
        return;
    }
    // Every named cmdtab entry has a handler; the C calls it unconditionally.
    if let Some(handler) = (*c).c_handler {
        handler(argc - 1, argv.offset(1));
    }
    if connected != 0 {
        proxflag = 1;
    } else {
        proxflag = 0;
    }
    pswitch(0);
}
