use crate::ftp::ftp::login;
use crate::ftp::ftp::null;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut cfile: *mut FILE = ::core::ptr::null::<FILE>() as *mut FILE;

static mut tokval: [::core::ffi::c_char; 100] = [0; 100];

static mut toktab: [toktab; 8] = [
    toktab {
        tokstr: b"default\0" as *const u8 as *const ::core::ffi::c_char,
        tval: DEFAULT,
    },
    toktab {
        tokstr: b"login\0" as *const u8 as *const ::core::ffi::c_char,
        tval: LOGIN,
    },
    toktab {
        tokstr: b"password\0" as *const u8 as *const ::core::ffi::c_char,
        tval: PASSWD,
    },
    toktab {
        tokstr: b"passwd\0" as *const u8 as *const ::core::ffi::c_char,
        tval: PASSWD,
    },
    toktab {
        tokstr: b"account\0" as *const u8 as *const ::core::ffi::c_char,
        tval: ACCOUNT,
    },
    toktab {
        tokstr: b"machine\0" as *const u8 as *const ::core::ffi::c_char,
        tval: MACH,
    },
    toktab {
        tokstr: b"macdef\0" as *const u8 as *const ::core::ffi::c_char,
        tval: MACDEF,
    },
    toktab {
        tokstr: ::core::ptr::null::<::core::ffi::c_char>(),
        tval: 0 as ::core::ffi::c_int,
    },
];

// static char sccsid[] = "@(#)ruserpass.c\t5.1 (Berkeley) 3/1/89";  (guarded by #ifndef lint)
pub static sccsid: [u8; 38] = *b"@(#)ruserpass.c\t5.1 (Berkeley) 3/1/89\0";

// struct utmp *getutmp();  -- function prototype, not emitted here

pub const MAXHOSTNAMELEN: i32 = 64;

// #define DEFAULT 1  -- already resolved via ntdll_sys::nt::DEFAULT import

pub const LOGIN: i32 = 2;

pub const PASSWD: i32 = 3;

pub const ACCOUNT: i32 = 4;

pub const MACDEF: i32 = 5;

pub const ID: i32 = 10;

pub const MACH: i32 = 11;

#[repr(C)]
pub struct toktab {
    pub tokstr: *const i8,
    pub tval: i32,
}

unsafe impl Sync for toktab {}

// extern char *hostname;  -- defined in the ftp front end; declared here as a static

// static int token(void);  -- function prototype, not emitted here
#[no_mangle]
pub unsafe extern "C" fn ruserpass(host: *const ::core::ffi::c_char, aname: *mut *mut ::core::ffi::c_char, apass: *mut *mut ::core::ffi::c_char, aacct: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut hdir: *const ::core::ffi::c_char = core::ptr::null();
    let mut mydomain: *const ::core::ffi::c_char = core::ptr::null();
    let mut buf: [::core::ffi::c_char; (BUFSIZ) as _] = [0; (BUFSIZ) as _];
    let mut tmp: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut myname: [::core::ffi::c_char; (MAXHOSTNAMELEN as _)] = [0; (MAXHOSTNAMELEN as _)];
    let mut t: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut usedefault: ::core::ffi::c_int = 0;
    let mut stb: stat = core::mem::zeroed();

    'bad: {
        'done: {
            hdir = getenv(b"HOME\0" as *const u8 as *const ::core::ffi::c_char);
            if hdir.is_null() {
                hdir = b".\0" as *const u8 as *const ::core::ffi::c_char;
            }
            sprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"%s/.netrc\0" as *const u8 as *const ::core::ffi::c_char,
                hdir,
            );
            cfile = fopen(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"r\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut FILE;
            if cfile.is_null() {
                if *_errno() != (ENOENT as i32) {
                    perror(&raw mut buf as *mut ::core::ffi::c_char);
                }
                return 0;
            }

            if gethostname(
                &raw mut myname as *mut ::core::ffi::c_char,
                core::mem::size_of::<[::core::ffi::c_char; (MAXHOSTNAMELEN as _)]>() as ::core::ffi::c_int,
            ) < 0
            {
                myname[0 as usize] = '\0' as ::core::ffi::c_char;
            }
            mydomain = index(
                &raw mut myname as *mut ::core::ffi::c_char,
                '.' as ::core::ffi::c_int,
            );
            if mydomain.is_null() {
                mydomain = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }

            'next: loop {
                t = token();
                if t == 0 {
                    break 'done;
                }
                match t {
                    DEFAULT => {
                        usedefault = 1;
                    }
                    MACH => {}
                    _ => {
                        continue;
                    }
                }

                if usedefault == 0 {
                    if token() != ID {
                        continue;
                    }
                    'mtch: {
                        if strcasecmp(host, &raw mut tokval as *mut ::core::ffi::c_char)
                            == 0
                        {
                            break 'mtch;
                        }
                        if strcasecmp(hostname, &raw mut tokval as *mut ::core::ffi::c_char)
                            == 0
                        {
                            break 'mtch;
                        }
                        tmp = index(hostname, '.' as ::core::ffi::c_int);
                        if !tmp.is_null()
                            && strcasecmp(tmp, mydomain) == 0
                            && strncasecmp(
                                hostname,
                                &raw mut tokval as *mut ::core::ffi::c_char,
                                tmp.offset_from(hostname) as usize,
                            ) == 0
                            && tokval[tmp.offset_from(hostname) as usize] == '\0' as ::core::ffi::c_char
                        {
                            break 'mtch;
                        }
                        tmp = index(host, '.' as ::core::ffi::c_int);
                        if !tmp.is_null()
                            && strcasecmp(tmp, mydomain) == 0
                            && strncasecmp(
                                host,
                                &raw mut tokval as *mut ::core::ffi::c_char,
                                tmp.offset_from(host) as usize,
                            ) == 0
                            && tokval[tmp.offset_from(host) as usize] == '\0' as ::core::ffi::c_char
                        {
                            break 'mtch;
                        }
                        continue 'next;
                    }
                }

                // match:
                loop {
                    t = token();
                    if !(t != 0 && t != MACH && t != DEFAULT) {
                        break 'done;
                    }
                    match t {
                        LOGIN => {
                            if token() != 0 {
                                if (*aname as usize) == 0 as usize {
                                    *aname = malloc(
                                        (strlen(&raw mut tokval as *mut ::core::ffi::c_char)
                                            as ::core::ffi::c_uint)
                                            .wrapping_add(1 as ::core::ffi::c_uint)
                                            as usize,
                                    ) as *mut ::core::ffi::c_char;
                                    strcpy(*aname, &raw mut tokval as *mut ::core::ffi::c_char);
                                } else {
                                    if strcmp(*aname, &raw mut tokval as *mut ::core::ffi::c_char)
                                        != 0
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                        PASSWD => {
                            if strcmp(
                                *aname,
                                b"anonymous\0" as *const u8 as *const ::core::ffi::c_char,
                            ) != 0
                                && fstat(fileno(cfile), &raw mut stb)
                                    >= 0
                                && ((stb.st_mode as ::core::ffi::c_int) & 0o77)
                                    != 0
                            {
                                fprintf(
                                    __iob_func().add(2),
                                    b"Error - .netrc file not correct mode.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                fprintf(
                                    __iob_func().add(2),
                                    b"Remove password or correct mode.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            if token() != 0
                                && (*apass as usize) == 0 as usize
                            {
                                *apass = malloc(
                                    (strlen(&raw mut tokval as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(1 as ::core::ffi::c_uint)
                                        as usize,
                                ) as *mut ::core::ffi::c_char;
                                strcpy(*apass, &raw mut tokval as *mut ::core::ffi::c_char);
                            }
                        }
                        ACCOUNT => {
                            if fstat(fileno(cfile), &raw mut stb)
                                >= 0
                                && ((stb.st_mode as ::core::ffi::c_int) & 0o77)
                                    != 0
                            {
                                fprintf(
                                    __iob_func().add(2),
                                    b"Error - .netrc file not correct mode.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                fprintf(
                                    __iob_func().add(2),
                                    b"Remove account or correct mode.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            if token() != 0
                                && (*aacct as usize) == 0 as usize
                            {
                                *aacct = malloc(
                                    (strlen(&raw mut tokval as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(1 as ::core::ffi::c_uint)
                                        as usize,
                                ) as *mut ::core::ffi::c_char;
                                strcpy(*aacct, &raw mut tokval as *mut ::core::ffi::c_char);
                            }
                        }
                        MACDEF => {
                            if proxy != 0 {
                                fclose(cfile);
                                return 0;
                            }
                            loop {
                                c = getc(cfile);
                                if !(c != EOF
                                    && (c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int))
                                {
                                    break;
                                }
                            }
                            if c == EOF || c == '\n' as ::core::ffi::c_int {
                                printf(
                                    b"Missing macdef name argument.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            if macnum == 16 {
                                printf(
                                    b"Limit of 16 macros have already been defined\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            tmp = &raw mut macros[macnum as usize].mac_name as *mut ::core::ffi::c_char;
                            *tmp = c as ::core::ffi::c_char;
                            tmp = tmp.offset(1);
                            i = 0;
                            loop {
                                if !(i < 8) {
                                    break;
                                }
                                c = getc(cfile);
                                if !(c != EOF) {
                                    break;
                                }
                                if !(isspace(c) == 0) {
                                    break;
                                }
                                *tmp = c as ::core::ffi::c_char;
                                tmp = tmp.offset(1);
                                i += 1;
                            }
                            if c == EOF {
                                printf(
                                    b"Macro definition missing null line terminator.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            *tmp = '\0' as ::core::ffi::c_char;
                            if c != '\n' as ::core::ffi::c_int {
                                loop {
                                    c = getc(cfile);
                                    if !(c != EOF && c != '\n' as ::core::ffi::c_int) {
                                        break;
                                    }
                                }
                            }
                            if c == EOF {
                                printf(
                                    b"Macro definition missing null line terminator.\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                            if macnum == 0 {
                                macros[macnum as usize].mac_start =
                                    &raw mut macbuf as *mut ::core::ffi::c_char;
                            } else {
                                macros[macnum as usize].mac_start = macros
                                    [(macnum - 1) as usize]
                                    .mac_end
                                    .offset(1 as isize);
                            }
                            tmp = macros[macnum as usize].mac_start;
                            while tmp
                                != (&raw mut macbuf as *mut ::core::ffi::c_char)
                                    .offset(4096 as isize)
                            {
                                c = getc(cfile);
                                if c == EOF {
                                    printf(
                                        b"Macro definition missing null line terminator.\n\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    break 'bad;
                                }
                                *tmp = c as ::core::ffi::c_char;
                                if *tmp as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                                    if *tmp.offset(-(1 as isize)) as ::core::ffi::c_int
                                        == '\0' as ::core::ffi::c_int
                                    {
                                        let fresh = macnum;
                                        macnum += 1;
                                        macros[fresh as usize].mac_end =
                                            tmp.offset(-(1 as isize));
                                        break;
                                    } else {
                                        *tmp = '\0' as ::core::ffi::c_char;
                                    }
                                }
                                tmp = tmp.offset(1);
                            }
                            if tmp
                                == (&raw mut macbuf as *mut ::core::ffi::c_char)
                                    .offset(4096 as isize)
                            {
                                printf(
                                    b"4K macro buffer exceeded\n\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                break 'bad;
                            }
                        }
                        _ => {
                            fprintf(
                                __iob_func().add(2),
                                b"Unknown .netrc keyword %s\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                &raw mut tokval as *mut ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        // done:
        fclose(cfile);
        return 0;
    }
    // bad:
    fclose(cfile);
    return -(1);
}

unsafe extern "C" fn token() -> ::core::ffi::c_int {
    let mut cp: *mut ::core::ffi::c_char;
    let mut c: ::core::ffi::c_int;
    let mut t: *mut toktab;
    if feof(cfile) != 0 {
        return 0;
    }
    loop {
        c = getc(cfile);
        if !(c != EOF
            && (c == '\r' as i32
                || c == '\n' as i32
                || c == '\t' as i32
                || c == ' ' as i32
                || c == ',' as i32))
        {
            break;
        }
    }
    if c == EOF {
        return 0;
    }
    cp = ::core::ptr::addr_of_mut!(tokval) as *mut ::core::ffi::c_char;
    if c == '"' as i32 {
        loop {
            c = getc(cfile);
            if !(c != EOF && c != '"' as i32) {
                break;
            }
            if c == '\\' as i32 {
                c = getc(cfile);
            }
            let fresh18 = cp;
            cp = cp.offset(1);
            *fresh18 = c as ::core::ffi::c_char;
        }
    } else {
        let fresh19 = cp;
        cp = cp.offset(1);
        *fresh19 = c as ::core::ffi::c_char;
        loop {
            c = getc(cfile);
            if !(c != EOF
                && c != '\n' as i32
                && c != '\t' as i32
                && c != ' ' as i32
                && c != ',' as i32
                && c != '\r' as i32)
            {
                break;
            }
            if c == '\\' as i32 {
                c = getc(cfile);
            }
            let fresh20 = cp;
            cp = cp.offset(1);
            *fresh20 = c as ::core::ffi::c_char;
        }
    }
    *cp = 0 as ::core::ffi::c_char;
    if tokval[0 as usize] as ::core::ffi::c_int == 0 {
        return 0;
    }
    t = ::core::ptr::addr_of_mut!(toktab) as *mut toktab;
    while !(*t).tokstr.is_null() {
        if strcmp(
            (*t).tokstr,
            ::core::ptr::addr_of!(tokval) as *const ::core::ffi::c_char,
        ) == 0
        {
            return (*t).tval;
        }
        t = t.offset(1);
    }
    return ID;
}
