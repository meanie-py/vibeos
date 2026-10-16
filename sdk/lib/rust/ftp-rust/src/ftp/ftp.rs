use crate::ftp::main::r#type as type_0;
use crate::ftp::domacro::domacro;
use crate::ftp::fake::fgetcSocket;
use crate::ftp::fake::getpass;
use crate::ftp::fake::herror;
use crate::ftp::main::lostpeer;
use crate::ftp::main::makeargv;
use crate::ftp::cmds::setebcdic;
use crate::ftp::fake::{fputSocket, gettimeofday, sleep};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub s_w1: USHORT,
    pub s_w2: USHORT,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub S_un_b: C2RustUnnamed_1,
    pub S_un_w: C2RustUnnamed_0,
    pub S_addr: ULONG,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub s_b1: UCHAR,
    pub s_b2: UCHAR,
    pub s_b3: UCHAR,
    pub s_b4: UCHAR,
}

#[no_mangle]
// All-zero, as the C's implicit initializer (c2rust spelled winsock's in_addr union its own way).
pub static mut hisctladdr: sockaddr_in = unsafe { core::mem::zeroed() };

#[no_mangle]
// All-zero, as the C's implicit initializer (c2rust spelled winsock's in_addr union its own way).
pub static mut data_addr: sockaddr_in = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut data: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

#[no_mangle]
pub static mut abrtflag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ptflag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
pub static mut allbinary: ::core::ffi::c_int = 0;

#[no_mangle]
// All-zero, as the C's implicit initializer (c2rust spelled winsock's in_addr union its own way).
pub static mut myctladdr: sockaddr_in = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut restart_point: off_t = 0 as off_t;

#[no_mangle]
pub static mut cout: SOCKET = 0;

#[no_mangle]
pub static mut cin: SOCKET = 0;

#[no_mangle]
pub static mut hostname: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut reply_string: [::core::ffi::c_char; 512] = [0; 512];

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut sendabort: jmp_buf = unsafe { core::mem::zeroed() };

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut recvabort: jmp_buf = unsafe { core::mem::zeroed() };

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut ptabort: jmp_buf = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ptabflg: ::core::ffi::c_int = 0;

pub const L_SET: i32 = 0;

pub const L_INCR: i32 = 1;

pub type caddr_t = *mut core::ffi::c_void;

#[allow(non_upper_case_globals)]
pub static sccsid: [i8; 34] = unsafe { core::mem::transmute(*b"@(#)ftp.c\t5.28 (Berkeley) 4/20/89\0") };

pub const MAXHOSTNAMELEN: i32 = 64;

pub type Sig_t = Option<unsafe extern "C" fn(i32)>;

pub const HASHBYTES: i32 = 1024;
#[no_mangle]
pub unsafe extern "C" fn hookup(host: *const ::core::ffi::c_char, port: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut hp: *mut hostent = 0 as *mut hostent;
    let mut len: core::ffi::c_int = 0;
    let mut s: SOCKET = 0;
    static mut hostnamebuf: [core::ffi::c_char; 80] = [0; 80];

    bzero!(
        (core::ptr::addr_of_mut!(hisctladdr) as *mut core::ffi::c_char).cast(),
        (core::mem::size_of::<sockaddr_in>() as core::ffi::c_int) as usize,
    );
    hisctladdr.sin_addr.S_un.S_addr = inet_addr(host) as core::ffi::c_ulong;
    if hisctladdr.sin_addr.S_un.S_addr != -(1 as core::ffi::c_int) as core::ffi::c_ulong {
        hisctladdr.sin_family = (AF_INET as u16) as i16;
        strncpy(
            core::ptr::addr_of_mut!(hostnamebuf) as *mut core::ffi::c_char,
            host,
            core::mem::size_of::<[core::ffi::c_char; 80]>() as usize,
        );
    } else {
        hp = gethostbyname(host);
        if hp.is_null() {
            fprintf(
                (__iob_func() as *mut FILE).offset(2) as *mut FILE,
                b"ftp: %s: \0".as_ptr() as *const core::ffi::c_char,
                host,
            );
            herror(0 as *mut core::ffi::c_char);
            code = -(1 as core::ffi::c_int);
            return 0 as *mut core::ffi::c_char;
        }
        hisctladdr.sin_family = (*hp).h_addrtype;
        bcopy!(
            (*(*hp).h_addr_list.offset(0 as isize) as *const core::ffi::c_char).cast(),
            (core::ptr::addr_of_mut!(hisctladdr.sin_addr) as *mut core::ffi::c_char).cast(),
            (*hp).h_length as usize,
        );
        strncpy(
            core::ptr::addr_of_mut!(hostnamebuf) as *mut core::ffi::c_char,
            (*hp).h_name,
            core::mem::size_of::<[core::ffi::c_char; 80]>() as usize,
        );
    }
    hostname = core::ptr::addr_of_mut!(hostnamebuf) as *mut core::ffi::c_char;
    s = socket(
        hisctladdr.sin_family as core::ffi::c_int,
        SOCK_STREAM as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    if s == INVALID_SOCKET as SOCKET {
        perror(b"ftp: socket\0".as_ptr() as *const core::ffi::c_char);
        code = -(1 as core::ffi::c_int);
        return 0 as *mut core::ffi::c_char;
    }
    hisctladdr.sin_port = port as u16;
    'bad: {
        while connect(
            s,
            core::ptr::addr_of_mut!(hisctladdr) as *mut sockaddr,
            core::mem::size_of::<sockaddr_in>() as core::ffi::c_int,
        ) < 0 as core::ffi::c_int
        {
            if !hp.is_null() && !(*(*hp).h_addr_list.offset(1 as isize)).is_null() {
                let oerrno: core::ffi::c_int = *_errno();

                fprintf(
                    (__iob_func() as *mut FILE).offset(2) as *mut FILE,
                    b"ftp: connect to address %s: \0".as_ptr() as *const core::ffi::c_char,
                    inet_ntoa(hisctladdr.sin_addr),
                );
                *_errno() = oerrno;
                perror(0 as *mut core::ffi::c_char);
                (*hp).h_addr_list = (*hp).h_addr_list.offset(1);
                bcopy!(
                    (*(*hp).h_addr_list.offset(0 as isize) as *const core::ffi::c_char).cast(),
                    (core::ptr::addr_of_mut!(hisctladdr.sin_addr) as *mut core::ffi::c_char).cast(),
                    (*hp).h_length as usize,
                );
                fprintf(
                    (__iob_func() as *mut FILE).offset(1) as *mut FILE,
                    b"Trying %s...\n\0".as_ptr() as *const core::ffi::c_char,
                    inet_ntoa(hisctladdr.sin_addr),
                );
                fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
                close(s as core::ffi::c_int);
                s = socket(
                    hisctladdr.sin_family as core::ffi::c_int,
                    SOCK_STREAM as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                if s == INVALID_SOCKET as SOCKET {
                    perror(b"ftp: socket\0".as_ptr() as *const core::ffi::c_char);
                    code = -(1 as core::ffi::c_int);
                    return 0 as *mut core::ffi::c_char;
                }
                continue;
            }
            perror(b"ftp: connect\0".as_ptr() as *const core::ffi::c_char);
            code = -(1 as core::ffi::c_int);
            break 'bad;
        }
        len = core::mem::size_of::<sockaddr_in>() as core::ffi::c_int;
        if getsockname(
            s,
            core::ptr::addr_of_mut!(myctladdr) as *mut sockaddr,
            core::ptr::addr_of_mut!(len),
        ) < 0 as core::ffi::c_int
        {
            perror(b"ftp: getsockname\0".as_ptr() as *const core::ffi::c_char);
            code = -(1 as core::ffi::c_int);
            break 'bad;
        }
        cout = s;
        cin = cout;
        if verbose != 0 {
            printf(
                b"Connected to %s.\n\0".as_ptr() as *const core::ffi::c_char,
                hostname,
            );
            fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        }
        /* read startup message from server */
        if getreply(0 as core::ffi::c_int) > 2 as core::ffi::c_int {
            closesocket(cin);
            code = -(1 as core::ffi::c_int);
            break 'bad;
        }
        {
            let mut on: core::ffi::c_int = 1 as core::ffi::c_int;

            if setsockopt(
                s,
                SOL_SOCKET as core::ffi::c_int,
                SO_OOBINLINE as core::ffi::c_int,
                core::ptr::addr_of_mut!(on) as *const core::ffi::c_char,
                core::mem::size_of::<core::ffi::c_int>() as core::ffi::c_int,
            ) < 0 as core::ffi::c_int && debug != 0
            {
                perror(b"ftp: setsockopt\0".as_ptr() as *const core::ffi::c_char);
            }
        }

        return hostname;
    }
    // bad:
    close(s as core::ffi::c_int);
    return 0 as *mut core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn login(host: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut tmp: [::core::ffi::c_char; 80] = [0; 80];
    let mut puser: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut ppass: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut pacct: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut user: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut pass: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut acct: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut n: ::core::ffi::c_int = 0;
    let mut aflag: ::core::ffi::c_int = 0;

    acct = ::core::ptr::null();
    pass = acct;
    user = pass;
    n = ruserpass(host, &raw mut puser, &raw mut ppass, &raw mut pacct);
    if n < 0 {
        code = -1;
        return 0;
    }
    if 0 != n {
        user = puser;
        pass = ppass;
        acct = pacct;
    }
    while user.is_null() {
        let myname: *const ::core::ffi::c_char = b"none\0".as_ptr() as *const ::core::ffi::c_char;
        if !myname.is_null() {
            printf(
                b"Name (%s:%s): \0".as_ptr() as *const ::core::ffi::c_char,
                host,
                myname,
            );
        } else {
            printf(
                b"Name (%s): \0".as_ptr() as *const ::core::ffi::c_char,
                host,
            );
        }
        fflush(__iob_func().add(1));
        fgets(
            tmp.as_mut_ptr(),
            (::core::mem::size_of_val(&tmp) - 1 as usize) as ::core::ffi::c_int,
            __iob_func().add(0),
        );
        tmp[(strlen(tmp.as_ptr()) - 1 as usize) as usize] = 0;
        if *tmp.as_ptr() == 0 {
            user = myname;
        } else {
            user = tmp.as_ptr();
        }
    }
    n = command(b"USER %s\0".as_ptr() as *const ::core::ffi::c_char, user);
    if n == (CONTINUE as i32) {
        if pass.is_null() {
            pass = getpass(b"Password:\0".as_ptr() as *const ::core::ffi::c_char);
        }
        n = command(b"PASS %s\0".as_ptr() as *const ::core::ffi::c_char, pass);
        fflush(__iob_func().add(0));
    }
    if n == (CONTINUE as i32) {
        aflag += 1;
        acct = getpass(b"Account:\0".as_ptr() as *const ::core::ffi::c_char);
        n = command(b"ACCT %s\0".as_ptr() as *const ::core::ffi::c_char, acct);
    }
    if n != (COMPLETE as i32) {
        fprintf(
            __iob_func().add(2),
            b"Login failed.\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0;
    }
    if aflag == 0 && !acct.is_null() {
        command(b"ACCT %s\0".as_ptr() as *const ::core::ffi::c_char, acct);
    }
    if proxy != 0 {
        return 1;
    }
    n = 0;
    while n < macnum {
        if strcmp(
            b"init\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut (*(&raw mut macros as *mut macel).offset(n as isize)).mac_name
                as *mut ::core::ffi::c_char,
        ) == 0
        {
            strcpy(
                &raw mut line as *mut ::core::ffi::c_char,
                b"$init\0".as_ptr() as *const ::core::ffi::c_char,
            );
            makeargv();
            domacro(margc, &raw mut margv as *mut *const ::core::ffi::c_char);
            break;
        }
        n += 1;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn command_v(
    mut fmt: *const ::core::ffi::c_char,
    ap_list: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut ap: va_list = *(ap_list as *mut va_list);
    let mut r: ::core::ffi::c_int = 0;
    let mut oldintr: Option<unsafe extern "C" fn(::core::ffi::c_int)> = None;
    abrtflag = 0;
    if debug != 0 {
        printf(b"---> \0".as_ptr() as *const ::core::ffi::c_char);

        vfprintf(__iob_func().add(1), fmt, ap);

        printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        (fflush(__iob_func().add(1)));
    }
    if cout == 0 {
        perror(b"No control connection for command\0".as_ptr() as *const ::core::ffi::c_char);
        code = -1;
        return 0;
    }
    oldintr = signal(SIGINT as i32, Some(cmdabort as unsafe extern "C" fn(::core::ffi::c_int)));
    {
        let mut buffer: [::core::ffi::c_char; 1024] = [0; 1024];


        vsprintf(
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
            fmt,
            ap,
        );

        strcat(
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
            b"\r\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        fprintfSocket(
            cout as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
        );
    }
    cpend = 1;
    r = getreply(
        (strcmp(fmt, b"QUIT\0".as_ptr() as *const ::core::ffi::c_char) == 0)
            as ::core::ffi::c_int,
    );
    if abrtflag != 0
        && ::core::mem::transmute::<Option<unsafe extern "C" fn(::core::ffi::c_int)>, usize>(oldintr)
            != SIG_IGN as usize
    {
        (oldintr.unwrap_unchecked())((SIGINT as i32));
    }
    return r;
}


#[no_mangle]
pub unsafe extern "C" fn getreply(mut expecteof: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut c: c_int = 0;
    let mut n: c_int = 0;
    let mut dig: c_int = 0;
    let mut cp: *mut c_char = core::ptr::null_mut();
    let mut originalcode: c_int = 0;
    let mut continuation: c_int = 0;
    let mut oldintr: Option<unsafe extern "C" fn(c_int)>;
    let mut pflag: c_int = 0;
    let mut pt: *mut c_char = core::ptr::addr_of_mut!(pasv) as *mut c_char;

    oldintr = core::mem::transmute::<_, Option<unsafe extern "C" fn(c_int)>>(
        signal(
            (SIGINT as i32),
            Some(cmdabort as unsafe extern "C" fn(c_int)),
        ),
    );
    loop {
        code = 0;
        n = code;
        dig = n;
        cp = core::ptr::addr_of_mut!(reply_string) as *mut c_char;
        loop {
            c = fgetcSocket(cin as c_int);
            if !(c != '\n' as c_int) {
                break;
            }
            if c == (IAC as i32) {
                /* handle telnet commands */
                let t: c_int = fgetcSocket(cin as c_int);
                if t == (WILL as i32) || t == (WONT as i32) {
                    c = fgetcSocket(cin as c_int);
                    fprintfSocket(
                        cout as c_int,
                        b"%c%c%c\0".as_ptr() as *const c_char,
                        IAC,
                        DONT,
                        c,
                    );
                } else if t == (DO as i32) || t == (DONT as i32) {
                    c = fgetcSocket(cin as c_int);
                    fprintfSocket(
                        cout as c_int,
                        b"%c%c%c\0".as_ptr() as *const c_char,
                        IAC,
                        WONT,
                        c,
                    );
                } else {
                }
                continue;
            }
            dig += 1;
            if c == EOF {
                if expecteof != 0 {
//					(void) signal(SIGINT,oldintr);
                    code = 221;
                    return 0;
                }
                lostpeer();
                if verbose != 0 {
                    printf(
                        b"421 Service not available, remote server has closed connection\n\0"
                            .as_ptr() as *const c_char,
                    );
                    fflush(__iob_func().add(1));
                }
                code = 421;
                return 4;
            }
            if c != '\r' as c_int
                && (verbose > 0
                    || (verbose > -(1)
                        && n == '5' as c_int
                        && dig > 4))
            {
                if proxflag != 0
                    && ((dig == 1 || dig == 5)
                        && verbose == 0)
                {
                    printf(b"%s:\0".as_ptr() as *const c_char, hostname);
                }
                putchar(c);
                fflush(__iob_func().add(1));
            }
            if dig < 4 && isdigit(c) != 0 {
                code = code * 10 + (c - '0' as c_int);
            }
            if pflag == 0 && code == 227 {
                pflag = 1;
            }
            if dig > 4
                && pflag == 1
                && isdigit(c) != 0
            {
                pflag = 2;
            }
            if pflag == 2 {
                if c != '\r' as c_int && c != ')' as c_int {
                    let fresh = pt;
                    pt = pt.add(1);
                    *fresh = c as c_char;
                } else {
                    *pt = '\0' as c_char;
                    pflag = 3;
                }
            }
            if dig == 4 && c == '-' as c_int {
                if continuation != 0 {
                    code = 0;
                }
                continuation += 1;
            }
            if n == 0 {
                n = c;
            }
            if cp
                < (core::ptr::addr_of_mut!(reply_string) as *mut c_char).add(
                    core::mem::size_of::<[c_char; BUFSIZ as usize]>()
                        - 1 as usize,
                )
            {
                let fresh = cp;
                cp = cp.add(1);
                *fresh = c as c_char;
            }
        }
        if verbose > 0
            || (verbose > -(1) && n == '5' as c_int)
        {
            putchar(c);
            fflush(__iob_func().add(1));
        }
        if continuation != 0 && code != originalcode {
            if originalcode == 0 {
                originalcode = code;
            }
            continue;
        }
        *cp = '\0' as c_char;
        if n != '1' as c_int {
            cpend = 0;
        }
        signal((SIGINT as i32), oldintr);
        if code == 421 || originalcode == 421 {
            lostpeer();
        }
        if abrtflag != 0
            && oldintr != Some(cmdabort as unsafe extern "C" fn(c_int))
            && oldintr
                != core::mem::transmute::<isize, Option<unsafe extern "C" fn(c_int)>>(
                    SIG_IGN as isize,
                )
        {
            oldintr.unwrap_unchecked()((SIGINT as i32));
        }
        return n - '0' as c_int;
    }
}

unsafe extern "C" fn empty(mut mask: *mut fd_set, mut sec: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut t: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    t.tv_sec = sec as ::core::ffi::c_long as LONG;
    t.tv_usec = 0;
    return select(
        32,
        mask,
        (0) as *mut fd_set,
        (0) as *mut fd_set,
        &raw mut t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn initconn() -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut a: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut result: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut tmpno: ::core::ffi::c_int = 0;
    let mut on: ::core::ffi::c_int = 1;
    let mut a0: ::core::ffi::c_int = 0;
    let mut a1: ::core::ffi::c_int = 0;
    let mut a2: ::core::ffi::c_int = 0;
    let mut a3: ::core::ffi::c_int = 0;
    let mut p0: ::core::ffi::c_int = 0;
    let mut p1: ::core::ffi::c_int = 0;
    let mut at: i32 = 0;
    loop {
        match at {
            0 => {
                if passivemode != 0 {
                    data = socket((AF_INET as i32), (SOCK_STREAM as i32), 0) as ::core::ffi::c_int;
                    if data < 0 {
                        perror(b"ftp: socket\0".as_ptr() as *const ::core::ffi::c_char);
                        return 1;
                    }
                    if (options & (SO_DEBUG as i32)) != 0
                        && setsockopt(
                            data as SOCKET,
                            (SOL_SOCKET as i32),
                            (SO_DEBUG as i32),
                            ::core::ptr::addr_of_mut!(on) as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
                        ) < 0
                    {
                        perror(b"ftp: setsockopt (ignored)\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    if command(b"PASV\0".as_ptr() as *const ::core::ffi::c_char) != (COMPLETE as i32) {
                        printf(b"Passive mode refused.\n\0".as_ptr() as *const ::core::ffi::c_char);
                        at = 2;
                        continue;
                    }

                    if sscanf(
                        ::core::ptr::addr_of_mut!(pasv) as *mut ::core::ffi::c_char,
                        b"%d,%d,%d,%d,%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::addr_of_mut!(a0),
                        ::core::ptr::addr_of_mut!(a1),
                        ::core::ptr::addr_of_mut!(a2),
                        ::core::ptr::addr_of_mut!(a3),
                        ::core::ptr::addr_of_mut!(p0),
                        ::core::ptr::addr_of_mut!(p1),
                    ) != 6
                    {
                        printf(b"Passive mode address scan failure. Shouldn't happen!\n\0".as_ptr() as *const ::core::ffi::c_char);
                        at = 2;
                        continue;
                    }

                    bzero!(::core::ptr::addr_of_mut!(data_addr) as *mut ::core::ffi::c_void, ::core::mem::size_of::<sockaddr_in>());
                    data_addr.sin_family = (AF_INET as u_short) as i16;
                    a = ::core::ptr::addr_of_mut!(data_addr.sin_addr.S_un.S_addr) as *mut ::core::ffi::c_char;
                    *a.offset(0 as isize) = (a0 & 0xff) as ::core::ffi::c_char;
                    *a.offset(1 as isize) = (a1 & 0xff) as ::core::ffi::c_char;
                    *a.offset(2 as isize) = (a2 & 0xff) as ::core::ffi::c_char;
                    *a.offset(3 as isize) = (a3 & 0xff) as ::core::ffi::c_char;
                    p = ::core::ptr::addr_of_mut!(data_addr.sin_port) as *mut ::core::ffi::c_char;
                    *p.offset(0 as isize) = (p0 & 0xff) as ::core::ffi::c_char;
                    *p.offset(1 as isize) = (p1 & 0xff) as ::core::ffi::c_char;

                    if connect(
                        data as SOCKET,
                        ::core::ptr::addr_of_mut!(data_addr) as *mut sockaddr,
                        ::core::mem::size_of::<sockaddr_in>() as ::core::ffi::c_int,
                    ) < 0
                    {
                        perror(b"ftp: connect\0".as_ptr() as *const ::core::ffi::c_char);
                        at = 2;
                        continue;
                    }
                    return 0;
                }
                at = 1;
                continue;
            }
            1 => {
                // noport:
                data_addr = myctladdr;
                if sendport != 0 {
                    data_addr.sin_port = 0 as u_short; // let system pick one
                }
                if data != -1 {
                    close((data as SOCKET) as i32);
                }
                data = socket((AF_INET as i32), (SOCK_STREAM as i32), 0) as ::core::ffi::c_int;
                if data < 0 {
                    perror(b"ftp: socket\0".as_ptr() as *const ::core::ffi::c_char);
                    if tmpno != 0 {
                        sendport = 1;
                    }
                    return 1;
                }
                if sendport == 0 {
                    if setsockopt(
                        data as SOCKET,
                        (SOL_SOCKET as i32),
                        (SO_REUSEADDR as i32),
                        ::core::ptr::addr_of_mut!(on) as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
                    ) < 0
                    {
                        perror(b"ftp: setsockopt (reuse address)\0".as_ptr() as *const ::core::ffi::c_char);
                        at = 2;
                        continue;
                    }
                }
                if bind(
                    data as SOCKET,
                    ::core::ptr::addr_of_mut!(data_addr) as *mut sockaddr,
                    ::core::mem::size_of::<sockaddr_in>() as ::core::ffi::c_int,
                ) < 0
                {
                    perror(b"ftp: bind\0".as_ptr() as *const ::core::ffi::c_char);
                    at = 2;
                    continue;
                }
                if (options & (SO_DEBUG as i32)) != 0
                    && setsockopt(
                        data as SOCKET,
                        (SOL_SOCKET as i32),
                        (SO_DEBUG as i32),
                        ::core::ptr::addr_of_mut!(on) as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
                    ) < 0
                {
                    perror(b"ftp: setsockopt (ignored)\0".as_ptr() as *const ::core::ffi::c_char);
                }
                len = ::core::mem::size_of::<sockaddr_in>() as ::core::ffi::c_int;
                if getsockname(
                    data as SOCKET,
                    ::core::ptr::addr_of_mut!(data_addr) as *mut sockaddr,
                    ::core::ptr::addr_of_mut!(len),
                ) < 0
                {
                    perror(b"ftp: getsockname\0".as_ptr() as *const ::core::ffi::c_char);
                    at = 2;
                    continue;
                }
                if listen(data as SOCKET, 1) < 0 {
                    perror(b"ftp: listen\0".as_ptr() as *const ::core::ffi::c_char);
                }
                if sendport != 0 {
                    a = ::core::ptr::addr_of_mut!(data_addr.sin_addr) as *mut ::core::ffi::c_char;
                    p = ::core::ptr::addr_of_mut!(data_addr.sin_port) as *mut ::core::ffi::c_char;
                    // #define UC(b) (((int)b)&0xff)
                    result = command(
                        b"PORT %d,%d,%d,%d,%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                        UC!(*a.offset(0 as isize) as ::core::ffi::c_int),
                        UC!(*a.offset(1 as isize) as ::core::ffi::c_int),
                        UC!(*a.offset(2 as isize) as ::core::ffi::c_int),
                        UC!(*a.offset(3 as isize) as ::core::ffi::c_int),
                        UC!(*p.offset(0 as isize) as ::core::ffi::c_int),
                        UC!(*p.offset(1 as isize) as ::core::ffi::c_int),
                    );
                    if result == (ERROR as i32) && sendport == -1 {
                        sendport = 0;
                        tmpno = 1;
                        at = 1;
                        continue; // goto noport
                    }
                    return (result != (COMPLETE as i32)) as ::core::ffi::c_int;
                }
                if tmpno != 0 {
                    sendport = 1;
                }
                return 0;
            }
            2 => {
                // bad:
                fflush(__iob_func().add(1));
                close((data as SOCKET) as i32);
                data = -1;
                if tmpno != 0 {
                    sendport = 1;
                }
                return 1;
            }
            _ => break,
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn dataconn(mode_local: *const core::ffi::c_char) -> core::ffi::c_int {
    let mut from: sockaddr_in = core::mem::zeroed();
    let mut s: core::ffi::c_int = 0;
    let mut fromlen: core::ffi::c_int = core::mem::size_of::<sockaddr_in>() as core::ffi::c_int;

    if passivemode != 0 {
        return data;
    }

    s = accept(data as SOCKET, &raw mut from as *mut sockaddr, &raw mut fromlen) as core::ffi::c_int;
    if s < 0 {
        perror(b"ftp: accept\0".as_ptr() as *const core::ffi::c_char);
        closesocket(data as SOCKET);
        data = -1;
        return 0;
    }
    if closesocket(data as SOCKET) != 0 {
        let mut iret: core::ffi::c_int = WSAGetLastError();
        fprintf(
            (__iob_func() as *mut FILE).offset(1) as *mut FILE,
            b"Error closing socket(%d)\n\0".as_ptr() as *const core::ffi::c_char,
            iret,
        );
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
    }

    data = s;
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn ptransfer(
    mut direction: *const ::core::ffi::c_char,
    mut bytes: ::core::ffi::c_long,
    mut t0: *mut timeval,
    mut t1: *mut timeval,
) {
    let mut td: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut s: ::core::ffi::c_double = 0.;
    let mut bs: ::core::ffi::c_double = 0.;
    if verbose != 0 {
        tvsub(&raw mut td, t1, t0);
        s = td.tv_sec as ::core::ffi::c_double
            + td.tv_usec as ::core::ffi::c_double / 1000000.0f64;
        bs = bytes as ::core::ffi::c_double / nz!(s);
        printf(
            b"%ld bytes %s in %.1f seconds (%.0f Kbytes/s)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            bytes,
            direction,
            s,
            bs / 1024.0f64,
        );
        fflush(
            (__iob_func() as *mut FILE).offset(1 as ::core::ffi::c_int as isize) as *mut FILE,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn tvsub(tdiff: *mut timeval, t1: *mut timeval, t0: *mut timeval) {
    (*tdiff).tv_sec = (*t1).tv_sec - (*t0).tv_sec;
    (*tdiff).tv_usec = (*t1).tv_usec - (*t0).tv_usec;
    if (*tdiff).tv_usec < 0 as LONG {
        (*tdiff).tv_sec -= 1;
        (*tdiff).tv_usec += 1000000 as LONG;
    }
}
#[no_mangle]
pub unsafe extern "C" fn psabort(mut flag: i32) {
    abrtflag += 1;
}
#[no_mangle]
pub unsafe extern "C" fn pswitch(mut flag: ::core::ffi::c_int) {
    #[repr(C)]
    struct comvars {
        connect: ::core::ffi::c_int,
        name: [::core::ffi::c_char; (MAXHOSTNAMELEN as _)],
        mctl: sockaddr_in,
        hctl: sockaddr_in,
        in_0: SOCKET,
        out: SOCKET,
        tpe: ::core::ffi::c_int,
        cpnd: ::core::ffi::c_int,
        sunqe: ::core::ffi::c_int,
        runqe: ::core::ffi::c_int,
        mcse: ::core::ffi::c_int,
        ntflg: ::core::ffi::c_int,
        nti: [::core::ffi::c_char; 17],
        nto: [::core::ffi::c_char; 17],
        mapflg: ::core::ffi::c_int,
        mi: [::core::ffi::c_char; (MAXPATHLEN as _)],
        mo: [::core::ffi::c_char; (MAXPATHLEN as _)],
    }
    static mut proxstruct: comvars = unsafe { core::mem::zeroed() };
    static mut tmpstruct: comvars = unsafe { core::mem::zeroed() };
    let mut oldintr: Sig_t = None;
    let mut ip: *mut comvars = core::ptr::null_mut();
    let mut op: *mut comvars = core::ptr::null_mut();

    abrtflag = 0 as ::core::ffi::c_int;
    oldintr = signal(
        SIGINT as ::core::ffi::c_int,
        Some(psabort as unsafe extern "C" fn(::core::ffi::c_int) -> ()),
    ) as Sig_t;
    if flag != 0 {
        if proxy != 0 {
            return;
        }
        ip = core::ptr::addr_of_mut!(tmpstruct);
        op = core::ptr::addr_of_mut!(proxstruct);
        proxy += 1;
    } else {
        if proxy == 0 {
            return;
        }
        ip = core::ptr::addr_of_mut!(proxstruct);
        op = core::ptr::addr_of_mut!(tmpstruct);
        proxy = 0 as ::core::ffi::c_int;
    }
    (*ip).connect = connected;
    connected = (*op).connect;
    if !hostname.is_null() {
        strncpy(
            core::ptr::addr_of_mut!((*ip).name) as *mut ::core::ffi::c_char,
            hostname,
            (core::mem::size_of::<[::core::ffi::c_char; (MAXHOSTNAMELEN as _)]>() as size_t)
                .wrapping_sub(1 as size_t),
        );
        (*ip).name[strlen(
            core::ptr::addr_of_mut!((*ip).name) as *mut ::core::ffi::c_char,
        ) as usize] = '\0' as i32 as ::core::ffi::c_char;
    } else {
        (*ip).name[(0) as usize] = 0 as ::core::ffi::c_char;
    }
    hostname = core::ptr::addr_of_mut!((*op).name) as *mut ::core::ffi::c_char;
    (*ip).hctl = hisctladdr;
    hisctladdr = (*op).hctl;
    (*ip).mctl = myctladdr;
    myctladdr = (*op).mctl;
    (*ip).in_0 = cin;
    cin = (*op).in_0;
    (*ip).out = cout;
    cout = (*op).out;
    (*ip).tpe = type_0;
    type_0 = (*op).tpe;
    if type_0 == 0 {
        type_0 = 1 as ::core::ffi::c_int;
    }
    (*ip).cpnd = cpend;
    cpend = (*op).cpnd;
    (*ip).sunqe = sunique;
    sunique = (*op).sunqe;
    (*ip).runqe = runique;
    runique = (*op).runqe;
    (*ip).mcse = mcase;
    mcase = (*op).mcse;
    (*ip).ntflg = ntflag;
    ntflag = (*op).ntflg;
    strncpy(
        core::ptr::addr_of_mut!((*ip).nti) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(ntin) as *mut ::core::ffi::c_char,
        16 as size_t,
    );
    (*ip).nti[strlen(
        core::ptr::addr_of_mut!((*ip).nti) as *mut ::core::ffi::c_char,
    ) as usize] = '\0' as i32 as ::core::ffi::c_char;
    strcpy(
        core::ptr::addr_of_mut!(ntin) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!((*op).nti) as *mut ::core::ffi::c_char,
    );
    strncpy(
        core::ptr::addr_of_mut!((*ip).nto) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(ntout) as *mut ::core::ffi::c_char,
        16 as size_t,
    );
    (*ip).nto[strlen(
        core::ptr::addr_of_mut!((*ip).nto) as *mut ::core::ffi::c_char,
    ) as usize] = '\0' as i32 as ::core::ffi::c_char;
    strcpy(
        core::ptr::addr_of_mut!(ntout) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!((*op).nto) as *mut ::core::ffi::c_char,
    );
    (*ip).mapflg = mapflag;
    mapflag = (*op).mapflg;
    strncpy(
        core::ptr::addr_of_mut!((*ip).mi) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(mapin) as *mut ::core::ffi::c_char,
        (MAXPATHLEN - 1) as size_t,
    );
    (*ip).mi[strlen(
        core::ptr::addr_of_mut!((*ip).mi) as *mut ::core::ffi::c_char,
    ) as usize] = '\0' as i32 as ::core::ffi::c_char;
    strcpy(
        core::ptr::addr_of_mut!(mapin) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!((*op).mi) as *mut ::core::ffi::c_char,
    );
    strncpy(
        core::ptr::addr_of_mut!((*ip).mo) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(mapout) as *mut ::core::ffi::c_char,
        (MAXPATHLEN - 1) as size_t,
    );
    (*ip).mo[strlen(
        core::ptr::addr_of_mut!((*ip).mo) as *mut ::core::ffi::c_char,
    ) as usize] = '\0' as i32 as ::core::ffi::c_char;
    strcpy(
        core::ptr::addr_of_mut!(mapout) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!((*op).mo) as *mut ::core::ffi::c_char,
    );
    if abrtflag != 0 {
        abrtflag = 0 as ::core::ffi::c_int;
        (oldintr.unwrap_unchecked())(1 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn reset(argc: ::core::ffi::c_int, argv: *mut *const ::core::ffi::c_char) {
    let mut mask: fd_set = core::mem::zeroed();
    let mut nfnd: ::core::ffi::c_int = 1;

    FD_ZERO!(core::ptr::addr_of_mut!(mask));
    while nfnd > 0 {
        //		FD_SET(fileno(cin), &mask); // Chris
        nfnd = empty(core::ptr::addr_of_mut!(mask), 0);
        if nfnd < 0 {
            perror(b"reset\0".as_ptr() as *const i8);
            code = -1;
            lostpeer();
        } else if nfnd != 0 {
            let _ = getreply(0);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn null() -> i32 {
    0
}

// Hand-ported from base/applications/network/ftp/ftp.c -- sendrequest, recvrequest,
// proxtrans, cmdabort -- for pasting into sdk/lib/rust/ftp-rust/src/ftp/ftp.rs.
//
// uses:
//   ftp.rs (same module): abrtflag, ptflag, ptabflg, data, cin, cout, restart_point,
//     allbinary, HASHBYTES, L_SET, L_INCR, getreply, empty, initconn, dataconn,
//     ptransfer, pswitch, null, type_0 (= crate::ftp::main::r#type, already imported)
//   main.rs: verbose, debug, hash, proxy, connected, cpend, code, crflag, runique, pasv,
//     lostpeer (already imported in ftp.rs)
//   cmds.rs: setascii, setbinary, settenex (via prelude), setebcdic (NEW `use` below)
//   fake.rs: fgetcSocket (already imported), fputSocket, sleep, gettimeofday (NEW `use` below)
//   prelude/island: command, fprintfSocket (C-variadic), printf, fprintf, fflush, perror,
//     putchar, fopen, fclose, fileno, getc, strcmp, _errno, __iob_func, closesocket,
//     WSAGetLastError, send, recv, stat, off_t, timeval, fd_set, SOCKET, FD_SETSIZE,
//     IAC, IP, DM, MSG_OOB, TYPE_A/I/E/L, PRELIM, COMPLETE, CONTINUE, ERROR
//
// PRELUDE ADDITIONS REQUIRED (not declared anywhere in ftp-rust today; ftp.rs already
// relies on the first four unqualified). C prototypes from sdk/include/crt + psdk:
//   type FILE (stdio.h)            jmp_buf already used by ftp.rs statics
//   const EOF: c_int = -1;         (stdio.h:24)
//   const BUFSIZ: c_int = 512;     (stdio.h:20)  -- `char buf[BUFSIZ]` in sendrequest
//   fn close(fd: c_int) -> c_int;                                  (io.h:776, oldnames)
//   fn read(fd: c_int, buf: *mut c_void, n: c_uint) -> c_int;      (io.h:859)
//   fn write(fd: c_int, buf: *const c_void, n: c_uint) -> c_int;   (io.h:898)
//   fn lseek(fd: c_int, off: c_long, origin: c_int) -> c_long;     (io.h:836)
//   fn fseek(f: *mut FILE, off: c_long, origin: c_int) -> c_int;   (stdio.h:371)
//   fn putc(ch: c_int, f: *mut FILE) -> c_int;                     (stdio.h:493)
//   fn _popen(cmd: *const c_char, mode: *const c_char) -> *mut FILE; (stdio.h:477)
//   fn _pclose(f: *mut FILE) -> c_int;                             (stdio.h:470)
//   #[link_name = "_fstat"] fn fstat(fd: c_int, st: *mut stat) -> c_int;
//       -- sys/stat.h:341 `fstat` is a __CRT_INLINE forwarding to _fstat((struct _stat*)st)
//   extern "system" fn __WSAFDIsSet(s: SOCKET, set: *mut fd_set) -> c_int;
//       -- winsock.h:65 (PASCAL); FD_ISSET expands to it
// NOTE ftp.rs:716 and :804 call `close(data as SOCKET)`; with close(c_int) those two
// existing sites need `close(data)` (data is already c_int) -- C passes the int.


// errno.h:52 / sys/stat.h:133,137,311,314 -- island does not carry them.

/// Replaces the C's `longjmp(ptabort, 1)` out of `cmdabort` (see cmdabort / proxtrans).
/// Set by `cmdabort` while `ptflag != 0`; consumed by `proxtrans` at reply boundaries.
pub static PTABORT: AtomicBool = AtomicBool::new(false);

#[inline(always)]
unsafe fn ftp_stdin() -> *mut FILE {
    (__iob_func() as *mut FILE).offset(0)
}
#[inline(always)]
unsafe fn ftp_stdout() -> *mut FILE {
    (__iob_func() as *mut FILE).offset(1)
}
#[inline(always)]
unsafe fn ftp_stderr() -> *mut FILE {
    (__iob_func() as *mut FILE).offset(2)
}

/// The repeated `switch (oldtype) { case 0: break; case TYPE_A: setascii(0, NULL); ... }`
/// of proxtrans (also its `switch (tmptype)`, which simply has no case 0 -- same effect).
unsafe fn ftp_settype_code(t: c_int) {
    if t == TYPE_A as c_int {
        setascii(0, core::ptr::null_mut());
    } else if t == TYPE_I as c_int {
        setbinary(0, core::ptr::null_mut());
    } else if t == TYPE_E as c_int {
        setebcdic();
    } else if t == TYPE_L as c_int {
        settenex(0, core::ptr::null_mut());
    }
}

/// recvrequest's repeated `if (oldtype) { if (!debug) verbose = 0; switch (oldtype)
/// { TYPE_I / TYPE_E / TYPE_L } verbose = oldverbose; }` -- note: no TYPE_A case there.
unsafe fn ftp_recv_restore_type(oldtype: c_int, oldverbose: c_int) {
    if oldtype != 0 {
        if debug == 0 {
            verbose = 0;
        }
        if oldtype == TYPE_I as c_int {
            setbinary(0, core::ptr::null_mut());
        } else if oldtype == TYPE_E as c_int {
            setebcdic();
        } else if oldtype == TYPE_L as c_int {
            settenex(0, core::ptr::null_mut());
        }
        verbose = oldverbose;
    }
}

/// proxtrans's repeated RFC959 abort block:
/// `if (cpend) { IAC IP; send IAC DM OOB; "ABOR"; FD_ZERO; empty(); getreply x2 }`
/// (the caller tests cpend). The `FD_SET(fileno(cin), &mask)` is commented out in the C,
/// so `empty` selects on an empty set -- preserved.
unsafe fn ftp_proxtrans_abor(mask: *mut fd_set) {
    let mut msg: [c_char; 2] = [0; 2];
    let nfnd: c_int;

    fprintfSocket(
        cout as c_int,
        b"%c%c\0".as_ptr() as *const c_char,
        IAC as c_int,
        IP as c_int,
    );
    msg[0] = IAC as c_char;
    msg[1] = DM as c_char;
    if send(cout, msg.as_ptr(), 2, MSG_OOB as c_int) != 2 {
        perror(b"abort\0".as_ptr() as *const c_char);
    }
    fprintfSocket(cout as c_int, b"ABOR\r\n\0".as_ptr() as *const c_char);
    (*mask).fd_count = 0; // FD_ZERO (winsock.h:90)
    nfnd = empty(mask, 10);
    if nfnd <= 0 {
        if nfnd < 0 {
            perror(b"abort\0".as_ptr() as *const c_char);
        }
        if ptabflg != 0 {
            code = -1;
        }
        lostpeer();
    }
    let _ = getreply(0);
    let _ = getreply(0);
}

// hand-ported: SIGINT handler whose C body longjmp()s across threads (UB on Windows);
// replaced by the PTABORT flag. The type `unsafe extern "C" fn(c_int)` (no return) is
// exactly what ftp.rs's command_v / getreply cast `cmdabort` to.
//
// DIVERGENCE: the C did `if (ptflag) longjmp(ptabort, 1);` to land in proxtrans's
// `if (setjmp(ptabort)) goto abort;`. The Windows CRT runs SIGINT (Ctrl-C) handlers on a
// separate console-control thread, so that longjmp restores the MAIN thread's saved
// context onto ANOTHER thread's stack -- undefined behaviour (the C is demonstrably
// broken here). Instead we raise PTABORT; proxtrans polls it after each reply/pswitch
// between `ptflag++` and `ptflag = 0` and takes its `abort:` path. Consequence: an
// interrupt during a proxy transfer takes effect at the next reply boundary rather than
// mid-recv. When cmdabort is invoked synchronously (command_v calling `oldintr(SIGINT)`),
// the same flag path applies.
unsafe extern "C" fn cmdabort(sig: c_int) {
    printf(b"\n\0".as_ptr() as *const c_char);
    fflush(ftp_stdout());
    // Possibly another thread: touch the C-visible globals without caching assumptions.
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!(abrtflag),
        core::ptr::read_volatile(core::ptr::addr_of!(abrtflag)) + 1,
    );
    if core::ptr::read_volatile(core::ptr::addr_of!(ptflag)) != 0 {
        PTABORT.store(true, Ordering::Release);
    }
}

/// proxtrans's stand-in for "did cmdabort longjmp to ptabort?" (consumes the flag).
#[inline(always)]
fn ftp_ptabort_taken() -> bool {
    PTABORT.swap(false, Ordering::Acquire)
}

// hand-ported: C uses setjmp(sendabort) twice -- both dead (abortsend is #if 0 and every
// signal(SIGINT, abortsend) is commented out) -- plus `goto abort` into a shared tail;
// lowered to a labeled block.
#[no_mangle]
pub unsafe extern "C" fn sendrequest(cmd: *const ::core::ffi::c_char, local: *const ::core::ffi::c_char, remote: *const ::core::ffi::c_char, printnames: ::core::ffi::c_int) {
    let mut fin: *mut FILE = core::ptr::null_mut();
    let mut dout: c_int = 0;
    let mut closefunc: Option<unsafe extern "C" fn(*mut FILE) -> c_int>;
    let mut oldintr: usize;
    let mut oldintp: usize;
    let mut buf: [c_char; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut bufp: *mut c_char;
    let mut bytes: c_long = 0;
    let mut hashbytes: c_long = HASHBYTES as c_long;
    let mut c: c_int = 0;
    let mut d: c_int = 0;
    let mut st: stat = core::mem::zeroed();
    let mut start: timeval = core::mem::zeroed();
    let mut stop: timeval = core::mem::zeroed();
    let mut mode_local: *const c_char;

    if verbose != 0 && printnames != 0 {
        if !local.is_null() && *local != b'-' as c_char {
            printf(b"local: %s \0".as_ptr() as *const c_char, local);
        }
        if !remote.is_null() {
            printf(b"remote: %s\n\0".as_ptr() as *const c_char, remote);
        }
        fflush(ftp_stdout());
    }
    if proxy != 0 {
        proxtrans(cmd, local, remote);
        return;
    }
    closefunc = None;
    oldintr = 0;
    oldintp = 0;
    mode_local = b"w\0".as_ptr() as *const c_char;
    // dropped: `if (setjmp(sendabort)) {...}` -- setjmp can only return 0 (no longjmp source).
    let _ = oldintr;
    null(); // oldintr = signal(SIGINT, abortsend);
    if strcmp(local, b"-\0".as_ptr() as *const c_char) == 0 {
        fin = ftp_stdin();
    } else if *local == b'|' as c_char {
        null(); // oldintp = signal(SIGPIPE,SIG_IGN);
        fin = _popen(local.add(1), b"r\0".as_ptr() as *const c_char);
        if fin.is_null() {
            perror(local.add(1));
            null(); // signal(SIGINT, oldintr);
            null(); // signal(SIGPIPE, oldintp);
            code = -1;
            return;
        }
        closefunc = Some(_pclose as unsafe extern "C" fn(*mut FILE) -> c_int);
    } else {
        fin = fopen(local, b"r\0".as_ptr() as *const c_char);
        if fin.is_null() {
            perror(local);
            null(); // signal(SIGINT, oldintr);
            code = -1;
            return;
        }
        closefunc = Some(fclose as unsafe extern "C" fn(*mut FILE) -> c_int);
        if fstat(fileno(fin), core::ptr::addr_of_mut!(st)) < 0
            || (st.st_mode as c_int & (S_IFMT as i32)) != (S_IFREG as i32)
        {
            fprintf(
                ftp_stdout(),
                b"%s: not a plain file.\n\0".as_ptr() as *const c_char,
                local,
            );
            fflush(ftp_stdout());
            null(); // signal(SIGINT, oldintr);
            fclose(fin);
            code = -1;
            return;
        }
    }
    if initconn() != 0 {
        null(); // signal(SIGINT, oldintr);
        if oldintp != 0 {
            null(); // signal(SIGPIPE, oldintp);
        }
        code = -1;
        if let Some(f) = closefunc {
            f(fin);
        }
        return;
    }
    'abort: {
        // dropped: `if (setjmp(sendabort)) goto abort;` -- never jumps; `abort:` is still
        // reached by the explicit `goto abort` below (break 'abort).

        if restart_point != 0
            && (strcmp(cmd, b"STOR\0".as_ptr() as *const c_char) == 0
                || strcmp(cmd, b"APPE\0".as_ptr() as *const c_char) == 0)
        {
            if fseek(fin, restart_point as c_long, 0) < 0 {
                perror(local);
                restart_point = 0;
                if let Some(f) = closefunc {
                    f(fin);
                }
                return;
            }
            if command(
                b"REST %ld\0".as_ptr() as *const c_char,
                restart_point as c_long,
            ) != CONTINUE as c_int
            {
                restart_point = 0;
                if let Some(f) = closefunc {
                    f(fin);
                }
                return;
            }
            restart_point = 0;
            mode_local = b"r+w\0".as_ptr() as *const c_char;
        }
        if !remote.is_null() {
            if command(b"%s %s\0".as_ptr() as *const c_char, cmd, remote) != PRELIM as c_int {
                null(); // signal(SIGINT, oldintr);
                if oldintp != 0 {
                    null(); // signal(SIGPIPE, oldintp);
                }
                if let Some(f) = closefunc {
                    f(fin);
                }
                return;
            }
        } else if command(b"%s\0".as_ptr() as *const c_char, cmd) != PRELIM as c_int {
            null(); // signal(SIGINT, oldintr);
            if oldintp != 0 {
                null(); // signal(SIGPIPE, oldintp);
            }
            if let Some(f) = closefunc {
                f(fin);
            }
            return;
        }
        dout = dataconn(mode_local);
        if dout == 0 {
            break 'abort;
        }
        gettimeofday(&raw mut start, core::ptr::null_mut());
        null(); // oldintp = signal(SIGPIPE, SIG_IGN);
        if type_0 == TYPE_I as c_int || type_0 == TYPE_L as c_int {
            d = 0;
            *_errno() = 0;
            loop {
                c = read(
                    fileno(fin),
                    buf.as_mut_ptr() as *mut c_void,
                    core::mem::size_of_val(&buf) as c_uint,
                );
                if c <= 0 {
                    break;
                }
                bytes += c as c_long;
                bufp = buf.as_mut_ptr();
                while c > 0 {
                    d = send(dout as SOCKET, bufp, c, 0);
                    if d <= 0 {
                        break;
                    }
                    c -= d;
                    bufp = bufp.offset(d as isize);
                }
                if hash != 0 {
                    while bytes >= hashbytes {
                        putchar(b'#' as c_int);
                        hashbytes += HASHBYTES as c_long;
                    }
                    fflush(ftp_stdout());
                }
            }
            if hash != 0 && bytes > 0 {
                if bytes < HASHBYTES as c_long {
                    putchar(b'#' as c_int);
                }
                putchar(b'\n' as c_int);
                fflush(ftp_stdout());
            }
            if c < 0 {
                perror(local);
            }
            if d <= 0 {
                if d == 0 {
                    fprintf(
                        ftp_stderr(),
                        b"netout: write returned 0?\n\0".as_ptr() as *const c_char,
                    );
                } else if *_errno() != (EPIPE as i32) {
                    perror(b"netout\0".as_ptr() as *const c_char);
                }
                bytes = -1;
            }
        } else if type_0 == TYPE_A as c_int {
            // shadows the outer buf, as the C block does
            let mut buf: [c_char; 1024] = [0; 1024];
            const BUFSIZE: c_int = 1024; // `static int bufsize = 1024;` -- never written
            let mut ipos: c_int = 0;

            loop {
                c = getc(fin);
                if c == EOF {
                    break;
                }
                if c == b'\n' as c_int {
                    while hash != 0 && bytes >= hashbytes {
                        putchar(b'#' as c_int);
                        fflush(ftp_stdout());
                        hashbytes += HASHBYTES as c_long;
                    }
                    // Szurgot: the '\r' insertion is commented out in the C on Win32.
                }
                if ipos >= BUFSIZE {
                    fputSocket(dout, buf.as_mut_ptr(), ipos);
                    if hash == 0 {
                        putchar(b'.' as c_int);
                    }
                    ipos = 0;
                }
                buf[ipos as usize] = c as c_char;
                ipos += 1;
                bytes += 1;
            }
            if ipos != 0 {
                fputSocket(dout, buf.as_mut_ptr(), ipos);
                ipos = 0;
            }
            if hash != 0 {
                if bytes < hashbytes {
                    putchar(b'#' as c_int);
                }
                putchar(b'\n' as c_int);
                fflush(ftp_stdout());
            } else {
                putchar(b'.' as c_int);
                putchar(b'\n' as c_int);
                fflush(ftp_stdout());
            }
            if ferror(fin) != 0 {
                perror(local);
            }
        }
        gettimeofday(&raw mut stop, core::ptr::null_mut());
        if let Some(f) = closefunc {
            f(fin);
        }
        if closesocket(dout as SOCKET) != 0 {
            let iret: c_int = WSAGetLastError();
            fprintf(
                ftp_stdout(),
                b"Error closing socket(%d)\n\0".as_ptr() as *const c_char,
                iret,
            );
            fflush(ftp_stdout());
        }
        let _ = getreply(0);
        null(); // signal(SIGINT, oldintr);
        if oldintp != 0 {
            null(); // signal(SIGPIPE, oldintp);
        }
        if bytes > 0 {
            ptransfer(
                b"sent\0".as_ptr() as *const c_char,
                bytes,
                &raw mut start,
                &raw mut stop,
            );
        }
        return;
    }
    // abort:
    gettimeofday(&raw mut stop, core::ptr::null_mut());
    null(); // signal(SIGINT, oldintr);
    if oldintp != 0 {
        null(); // signal(SIGPIPE, oldintp);
    }
    if cpend == 0 {
        code = -1;
        // DIVERGENCE: C calls `(*closefunc)(fin)` unconditionally here; closefunc is NULL
        // when fin is stdin ("-"), which would call through a null pointer. Guarded.
        if let Some(f) = closefunc {
            f(fin);
        }
        return;
    }
    if data >= 0 {
        close(data);
        data = -1;
    }
    if dout != 0 && closesocket(dout as SOCKET) != 0 {
        let iret: c_int = WSAGetLastError();
        fprintf(
            ftp_stdout(),
            b"Error closing socket(%d)\n\0".as_ptr() as *const c_char,
            iret,
        );
        fflush(ftp_stdout());
    }
    let _ = getreply(0);
    code = -1;
    if let Some(f) = closefunc {
        if !fin.is_null() {
            f(fin);
        }
    }
    if bytes > 0 {
        ptransfer(
            b"sent\0".as_ptr() as *const c_char,
            bytes,
            &raw mut start,
            &raw mut stop,
        );
    }
}

// hand-ported: C uses two dead setjmp(recvabort) sites, a shared `abort:` tail reached by
// `goto abort`, and `goto done` / `goto contin2` / `goto break2` in the TYPE_A path;
// lowered to labeled blocks/loops. The `#ifndef _WIN32` access()/gunique block is
// compiled out of this _WIN32 build and is omitted.
#[no_mangle]
pub unsafe extern "C" fn recvrequest(cmd: *const ::core::ffi::c_char, local: *const ::core::ffi::c_char, remote: *const ::core::ffi::c_char, mode_local: *const ::core::ffi::c_char, printnames: ::core::ffi::c_int) {
    let mut fout: *mut FILE = ftp_stdout();
    let mut din: c_int = 0;
    let mut closefunc: Option<unsafe extern "C" fn(*mut FILE) -> c_int>;
    let mut oldintr: usize;
    let mut oldintp: usize;
    let mut oldverbose: c_int = 0;
    let mut oldtype: c_int = 0;
    let is_retr: c_int;
    let tcrflag: c_int;
    let mut nfnd: c_int;
    let mut bare_lfs: c_int = 0;
    let mut msg: c_char;
    let mut buf: [c_char; 1024] = [0; 1024];
    const BUFSIZE: c_int = 1024; // `static int bufsize = 1024;` -- never written
    let mut bytes: c_long = 0;
    let mut hashbytes: c_long = HASHBYTES as c_long;
    let mut mask: fd_set = core::mem::zeroed();
    let mut c: c_int;
    let mut start: timeval = core::mem::zeroed();
    let mut stop: timeval = core::mem::zeroed();

    is_retr = (strcmp(cmd, b"RETR\0".as_ptr() as *const c_char) == 0) as c_int;
    if is_retr != 0 && verbose != 0 && printnames != 0 {
        if !local.is_null() && *local != b'-' as c_char {
            printf(b"local: %s \0".as_ptr() as *const c_char, local);
        }
        if !remote.is_null() {
            printf(b"remote: %s\n\0".as_ptr() as *const c_char, remote);
        }
        fflush(ftp_stdout());
    }
    if proxy != 0 && is_retr != 0 {
        proxtrans(cmd, local, remote);
        return;
    }
    closefunc = None;
    oldintr = 0;
    oldintp = 0;
    let _ = oldintr;
    tcrflag = (crflag == 0 && is_retr != 0) as c_int;
    // dropped: `if (setjmp(recvabort)) {...}` -- setjmp can only return 0 (no longjmp source).
    null(); // oldintr = signal(SIGINT, abortrecv);
    if strcmp(local, b"-\0".as_ptr() as *const c_char) != 0 && *local != b'|' as c_char {
        // #ifndef _WIN32 access()/gunique() block -- compiled out on _WIN32.
    }
    if initconn() != 0 {
        null(); // signal(SIGINT, oldintr);
        code = -1;
        return;
    }
    'abort: {
        // dropped: `if (setjmp(recvabort)) goto abort;` -- never jumps; `abort:` is still
        // reached by the explicit `goto abort` sites below (break 'abort).
        if is_retr == 0 {
            if type_0 != TYPE_A as c_int && (allbinary == 0 || type_0 != TYPE_I as c_int) {
                oldtype = type_0;
                oldverbose = verbose;
                if debug == 0 {
                    verbose = 0;
                }
                setascii(0, core::ptr::null_mut());
                verbose = oldverbose;
            }
        } else if restart_point != 0 {
            if command(
                b"REST %ld\0".as_ptr() as *const c_char,
                restart_point as c_long,
            ) != CONTINUE as c_int
            {
                return;
            }
        }
        if !remote.is_null() {
            if command(b"%s %s\0".as_ptr() as *const c_char, cmd, remote) != PRELIM as c_int {
                null(); // signal(SIGINT, oldintr);
                ftp_recv_restore_type(oldtype, oldverbose);
                return;
            }
        } else if command(b"%s\0".as_ptr() as *const c_char, cmd) != PRELIM as c_int {
            null(); // signal(SIGINT, oldintr);
            ftp_recv_restore_type(oldtype, oldverbose);
            return;
        }
        din = dataconn(b"r\0".as_ptr() as *const c_char);
        if din == 0 {
            break 'abort;
        }
        if strcmp(local, b"-\0".as_ptr() as *const c_char) == 0 {
            fout = ftp_stdout();
        } else if *local == b'|' as c_char {
            null(); // oldintp = signal(SIGPIPE, SIG_IGN);
            fout = _popen(local.add(1), b"w\0".as_ptr() as *const c_char);
            if fout.is_null() {
                perror(local.add(1));
                break 'abort;
            }
            closefunc = Some(_pclose as unsafe extern "C" fn(*mut FILE) -> c_int);
        } else {
            fout = fopen(local, mode_local);
            if fout.is_null() {
                perror(local);
                break 'abort;
            }
            closefunc = Some(fclose as unsafe extern "C" fn(*mut FILE) -> c_int);
        }
        gettimeofday(&raw mut start, core::ptr::null_mut());
        if type_0 == TYPE_I as c_int || type_0 == TYPE_L as c_int {
            if restart_point != 0
                && lseek(fileno(fout), restart_point as c_long, L_SET) < 0
            {
                perror(local);
                if let Some(f) = closefunc {
                    f(fout);
                }
                return;
            }
            *_errno() = 0;
            loop {
                c = recv(din as SOCKET, buf.as_mut_ptr(), BUFSIZE, 0);
                if c <= 0 {
                    break;
                }
                write(fileno(fout), buf.as_ptr() as *const c_void, c as c_uint);
                bytes += c as c_long;
                if hash != 0 {
                    while bytes >= hashbytes {
                        putchar(b'#' as c_int);
                        hashbytes += HASHBYTES as c_long;
                    }
                    fflush(ftp_stdout());
                }
            }
            if hash != 0 && bytes > 0 {
                if bytes < HASHBYTES as c_long {
                    putchar(b'#' as c_int);
                }
                putchar(b'\n' as c_int);
                fflush(ftp_stdout());
            }
        } else if type_0 == TYPE_A as c_int {
            if restart_point != 0 {
                // `goto done` from two sites, plus fall-in when the final fseek fails.
                let reached_done: bool = 'done: {
                    let mut i: c_int;
                    let n: c_int;
                    let mut c: c_int; // the C block's own `register int i, n, c`

                    if fseek(fout, 0, L_SET) < 0 {
                        break 'done true;
                    }
                    n = restart_point as c_int;
                    i = 0;
                    loop {
                        // while (i++ < n)
                        let prev = i;
                        i += 1;
                        if prev >= n {
                            break;
                        }
                        c = getc(fout);
                        if c == EOF {
                            break 'done true;
                        }
                        if c == b'\n' as c_int {
                            i += 1;
                        }
                    }
                    fseek(fout, 0, L_INCR) < 0
                };
                if reached_done {
                    // done:
                    perror(local);
                    if let Some(f) = closefunc {
                        f(fout);
                    }
                    return;
                }
            }
            'break2: loop {
                c = fgetcSocket(din);
                if c == EOF {
                    break;
                }
                'contin2: {
                    if c == b'\n' as c_int {
                        bare_lfs += 1;
                    }
                    while c == b'\r' as c_int {
                        while hash != 0 && bytes >= hashbytes {
                            putchar(b'#' as c_int);
                            fflush(ftp_stdout());
                            hashbytes += HASHBYTES as c_long;
                        }
                        bytes += 1;
                        c = fgetcSocket(din);
                        if c != b'\n' as c_int || tcrflag != 0 {
                            if ferror(fout) != 0 {
                                break 'break2;
                            }
                            putc(b'\r' as c_int, fout);
                            if c == 0 {
                                bytes += 1;
                                break 'contin2;
                            }
                            if c == EOF {
                                break 'contin2;
                            }
                        }
                    }
                    putc(c, fout);
                    bytes += 1;
                }
                // contin2: ;
            }
            // break2:
            if bare_lfs != 0 {
                printf(
                    b"WARNING! %d bare linefeeds received in ASCII mode\n\0".as_ptr()
                        as *const c_char,
                    bare_lfs,
                );
                printf(b"File may not have transferred correctly.\n\0".as_ptr() as *const c_char);
                fflush(ftp_stdout());
            }
            if hash != 0 {
                if bytes < hashbytes {
                    putchar(b'#' as c_int);
                }
                putchar(b'\n' as c_int);
                fflush(ftp_stdout());
            }
            if ferror(fout) != 0 {
                perror(local);
            }
        }
        if let Some(f) = closefunc {
            f(fout);
        }
        null(); // signal(SIGINT, oldintr);
        if oldintp != 0 {
            null(); // signal(SIGPIPE, oldintp);
        }
        gettimeofday(&raw mut stop, core::ptr::null_mut());
        if closesocket(din as SOCKET) != 0 {
            let iret: c_int = WSAGetLastError();
            fprintf(
                ftp_stdout(),
                b"Error closing socket(%d)\n\0".as_ptr() as *const c_char,
                iret,
            );
            fflush(ftp_stdout());
        }
        let _ = getreply(0);
        if bytes > 0 && is_retr != 0 {
            ptransfer(
                b"received\0".as_ptr() as *const c_char,
                bytes,
                &raw mut start,
                &raw mut stop,
            );
        }
        ftp_recv_restore_type(oldtype, oldverbose);
        return;
    }
    // abort:  (abort using RFC959 recommended IP,SYNC sequence)
    gettimeofday(&raw mut stop, core::ptr::null_mut());
    if oldintp != 0 {
        null(); // signal(SIGPIPE, oldintr);
    }
    null(); // signal(SIGINT, SIG_IGN);
    ftp_recv_restore_type(oldtype, oldverbose);
    if cpend == 0 {
        code = -1;
        null(); // signal(SIGINT, oldintr);
        return;
    }
    fprintfSocket(
        cout as c_int,
        b"%c%c\0".as_ptr() as *const c_char,
        IAC as c_int,
        IP as c_int,
    );
    msg = IAC as c_char;
    // send IAC in urgent mode instead of DM because UNIX places oob mark
    // after urgent byte rather than before as now is protocol
    if send(cout, &raw const msg, 1, MSG_OOB as c_int) != 1 {
        perror(b"abort\0".as_ptr() as *const c_char);
    }
    fprintfSocket(
        cout as c_int,
        b"%cABOR\r\n\0".as_ptr() as *const c_char,
        DM as c_int,
    );
    mask.fd_count = 0; // FD_ZERO (winsock.h:90)
    FD_SET!(cin, &raw mut mask);
    if din != 0 {
        FD_SET!(din as SOCKET, &raw mut mask);
    }
    nfnd = empty(&raw mut mask, 10);
    if nfnd <= 0 {
        if nfnd < 0 {
            perror(b"abort\0".as_ptr() as *const c_char);
        }
        code = -1;
        lostpeer();
    }
    if din != 0 && __WSAFDIsSet(din as SOCKET, &raw mut mask) != 0 {
        while recv(din as SOCKET, buf.as_mut_ptr(), BUFSIZE, 0) > 0 {}
    }
    // ERROR here is wingdi.h's `#define ERROR 0` (the island's ERROR = 0): this _WIN32 build
    // never sees BSD arpa/ftp.h's ERROR 5. Preserved as built.
    if getreply(0) == ERROR as c_int && code == 552 {
        // needed for nic style abort
        if data >= 0 {
            close(data);
            data = -1;
        }
        let _ = getreply(0);
    }
    let _ = getreply(0);
    code = -1;
    if data >= 0 {
        close(data);
        data = -1;
    }
    if let Some(f) = closefunc {
        if !fout.is_null() {
            f(fout);
        }
    }
    if din != 0 && closesocket(din as SOCKET) != 0 {
        let iret: c_int = WSAGetLastError();
        fprintf(
            ftp_stdout(),
            b"Error closing socket(%d)\n\0".as_ptr() as *const c_char,
            iret,
        );
        fflush(ftp_stdout());
    }
    if bytes > 0 {
        ptransfer(
            b"received\0".as_ptr() as *const c_char,
            bytes,
            &raw mut start,
            &raw mut stop,
        );
    }
    null(); // signal(SIGINT, oldintr);
}

// hand-ported: the C's `if (setjmp(ptabort)) goto abort;` is the target of the one LIVE
// longjmp in ftp (from cmdabort, across threads -- UB on Windows). Replaced by polling
// PTABORT; see cmdabort for the divergence. Labels lowered to a labeled block.
#[no_mangle]
pub unsafe extern "C" fn proxtrans(cmd: *const c_char, local: *const c_char, remote: *const c_char) {
    let tmptype: c_int;
    let mut oldtype: c_int = 0;
    let mut secndflag: c_int = 0;
    let mut nfnd: c_int;
    let cmd2: *const c_char;
    let mut mask: fd_set = core::mem::zeroed();

    if strcmp(cmd, b"RETR\0".as_ptr() as *const c_char) != 0 {
        cmd2 = b"RETR\0".as_ptr() as *const c_char;
    } else {
        cmd2 = if runique != 0 {
            b"STOU\0".as_ptr() as *const c_char
        } else {
            b"STOR\0".as_ptr() as *const c_char
        };
    }
    if command(b"PASV\0".as_ptr() as *const c_char) != COMPLETE as c_int {
        printf(b"proxy server does not support third part transfers.\n\0".as_ptr() as *const c_char);
        fflush(ftp_stdout());
        return;
    }
    tmptype = type_0;
    pswitch(0);
    if connected == 0 {
        printf(b"No primary connection\n\0".as_ptr() as *const c_char);
        fflush(ftp_stdout());
        pswitch(1);
        code = -1;
        return;
    }
    if type_0 != tmptype {
        oldtype = type_0;
        ftp_settype_code(tmptype); // switch (tmptype) -- no case 0
    }
    if command(
        b"PORT %s\0".as_ptr() as *const c_char,
        core::ptr::addr_of_mut!(pasv) as *mut c_char,
    ) != COMPLETE as c_int
    {
        ftp_settype_code(oldtype);
        pswitch(1);
        return;
    }
    'abort: {
        // C: `if (setjmp(ptabort)) goto abort;` -- arm the flag instead of saving a context.
        PTABORT.store(false, Ordering::Release);
        null(); // oldintr = signal(SIGINT, abortpt);
        if command(b"%s %s\0".as_ptr() as *const c_char, cmd, remote) != PRELIM as c_int {
            null(); // signal(SIGINT, oldintr);
            ftp_settype_code(oldtype);
            pswitch(1);
            return;
        }
        sleep(2);
        pswitch(1);
        secndflag += 1;
        if command(b"%s %s\0".as_ptr() as *const c_char, cmd2, local) != PRELIM as c_int {
            break 'abort;
        }
        ptflag += 1;
        // From here to `ptflag = 0` the C's cmdabort could longjmp to `abort:`. We check
        // PTABORT at each reply/pswitch boundary instead (see cmdabort).
        let _ = getreply(0);
        if ftp_ptabort_taken() {
            break 'abort;
        }
        pswitch(0);
        if ftp_ptabort_taken() {
            break 'abort;
        }
        let _ = getreply(0);
        if ftp_ptabort_taken() {
            break 'abort;
        }
        null(); // signal(SIGINT, oldintr);
        ftp_settype_code(oldtype); // issues TYPE commands: cmdabort is live here too
        if ftp_ptabort_taken() {
            break 'abort;
        }
        pswitch(1);
        if ftp_ptabort_taken() {
            break 'abort;
        }
        ptflag = 0;
        printf(b"local: %s remote: %s\n\0".as_ptr() as *const c_char, local, remote);
        fflush(ftp_stdout());
        return;
    }
    // abort:
    null(); // signal(SIGINT, SIG_IGN);
    ptflag = 0;
    if strcmp(cmd, b"RETR\0".as_ptr() as *const c_char) != 0 && proxy == 0 {
        pswitch(1);
    } else if strcmp(cmd, b"RETR\0".as_ptr() as *const c_char) == 0 && proxy != 0 {
        pswitch(0);
    }
    if cpend == 0 && secndflag == 0 {
        // only here if cmd = "STOR" (proxy=1)
        if command(b"%s %s\0".as_ptr() as *const c_char, cmd2, local) != PRELIM as c_int {
            pswitch(0);
            ftp_settype_code(oldtype);
            if cpend != 0 {
                ftp_proxtrans_abor(&raw mut mask);
            }
        }
        pswitch(1);
        if ptabflg != 0 {
            code = -1;
        }
        null(); // signal(SIGINT, oldintr);
        return;
    }
    if cpend != 0 {
        ftp_proxtrans_abor(&raw mut mask);
    }
    pswitch((proxy == 0) as c_int);
    if cpend == 0 && secndflag == 0 {
        // only if cmd = "RETR" (proxy=1)
        if command(b"%s %s\0".as_ptr() as *const c_char, cmd2, local) != PRELIM as c_int {
            pswitch(0);
            ftp_settype_code(oldtype);
            if cpend != 0 {
                ftp_proxtrans_abor(&raw mut mask);
            }
            pswitch(1);
            if ptabflg != 0 {
                code = -1;
            }
            null(); // signal(SIGINT, oldintr);
            return;
        }
    }
    if cpend != 0 {
        ftp_proxtrans_abor(&raw mut mask);
    }
    pswitch((proxy == 0) as c_int);
    if cpend != 0 {
        mask.fd_count = 0; // FD_ZERO; FD_SET(fileno(cin)) is commented out in the C
        nfnd = empty(&raw mut mask, 10);
        if nfnd <= 0 {
            if nfnd < 0 {
                perror(b"abort\0".as_ptr() as *const c_char);
            }
            if ptabflg != 0 {
                code = -1;
            }
            lostpeer();
        }
        let _ = getreply(0);
        let _ = getreply(0);
    }
    if proxy != 0 {
        pswitch(0);
    }
    ftp_settype_code(oldtype);
    pswitch(1);
    if ptabflg != 0 {
        code = -1;
    }
    null(); // signal(SIGINT, oldintr);
}
