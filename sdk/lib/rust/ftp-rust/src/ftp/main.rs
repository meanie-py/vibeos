use crate::ftp::ftp::null;
use crate::ftp::ftp::pswitch;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static mut cout: SOCKET;
    static mut data: ::core::ffi::c_int;
    static mut cmdtab: [cmd; 0];
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut portnum: ::core::ffi::c_short = 0;

#[no_mangle]
pub static mut home: [::core::ffi::c_char; 128] = [0; 128];

#[no_mangle]
pub static mut globerr: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut autologin: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut trace: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut hash: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut verbose: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut connected: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut fromatty: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut interactive: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut debug: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut bell: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut doglob: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut proxy: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut passivemode: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut proxflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut sunique: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut runique: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut mcase: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut ntflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut mapflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut code: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut crflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut pasv: [::core::ffi::c_char; 64] = [0; 64];

#[no_mangle]
pub static mut altarg: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut ntin: [::core::ffi::c_char; 17] = [0; 17];

#[no_mangle]
pub static mut ntout: [::core::ffi::c_char; 17] = [0; 17];

#[no_mangle]
pub static mut mapin: [::core::ffi::c_char; 255] = [0; 255];

#[no_mangle]
pub static mut mapout: [::core::ffi::c_char; 255] = [0; 255];

#[no_mangle]
pub static mut typename: [::core::ffi::c_char; 32] = [0; 32];

#[no_mangle]
pub static mut structname: [::core::ffi::c_char; 32] = [0; 32];

#[no_mangle]
pub static mut stru: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut formname: [::core::ffi::c_char; 32] = [0; 32];

#[no_mangle]
pub static mut form: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut modename: [::core::ffi::c_char; 32] = [0; 32];

#[no_mangle]
pub static mut mode: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut bytename: [::core::ffi::c_char; 32] = [0; 32];

#[no_mangle]
pub static mut bytesize: ::core::ffi::c_int = 0;

#[no_mangle]
// No setjmp reads it any more (every jump is dead or a flag); zero-filled because jmp_buf is a
// different array per arch, so c2rust's `[0; 24]` is i386-only.
pub static mut toplevel: jmp_buf = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut line: [::core::ffi::c_char; 200] = [0; 200];

#[no_mangle]
pub static mut stringbase: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut argbuf: [::core::ffi::c_char; 200] = [0; 200];

#[no_mangle]
pub static mut argbase: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut margc: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut margv: [*const ::core::ffi::c_char; 20] =
    [::core::ptr::null::<::core::ffi::c_char>(); 20];

#[no_mangle]
pub static mut cpend: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut mflag: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut options: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut macnum: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut macros: [macel; 16] = [macel {
    mac_name: [0; 9],
    mac_start: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    mac_end: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
}; 16];

#[no_mangle]
pub static mut macbuf: [::core::ffi::c_char; 4096] = [0; 4096];

#[no_mangle]
pub static mut sendport: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

#[no_mangle]
pub static mut slrflag: ::core::ffi::c_int = 0;

pub const HELPINDENT: usize = ::core::mem::size_of::<[::core::ffi::c_char; 10]>();

pub static mut copyright: [u8; 94] = *b"@(#) Copyright (c) 1985, 1989 Regents of the University of California.\n All rights reserved.\n\0";

pub static mut sccsid: [u8; 44] = *b"@(#)main.c\tbased on 5.13 (Berkeley) 3/14/89\0";

// uid_t getuid(void); void intr(void); void lostpeer(void); char *getlogin(void);
// and static const char *slurpstring(); are prototypes, provided as imports.


pub static mut r#type: i32 = 0;
#[no_mangle]
pub unsafe extern "C" fn lostpeer() {
    if connected != 0 {
        if cout != 0 {
            closesocket(cout);
            cout = 0;
        }
        if data >= 0 {
            let _ = shutdown(
                data as SOCKET,
                1 + 1,
            );
            let _ = close(data);
            data = -1;
        }
        connected = 0;
    }
    pswitch(1);
    if connected != 0 {
        if cout != 0 {
            closesocket(cout);
            cout = 0;
        }
        connected = 0;
    }
    proxflag = 0;
    pswitch(0);
}
#[no_mangle]
pub unsafe extern "C" fn cmdscanner(top: ::core::ffi::c_int) {
    let mut c: *mut cmd = ::core::ptr::null_mut::<cmd>();
    if top == 0 {
        putchar('\n' as i32);
    }
    loop {
        fflush(__iob_func().add(1));
        if fromatty != 0 {
            printf(b"ftp> \0".as_ptr() as *const ::core::ffi::c_char);
            fflush(__iob_func().add(1));
        }
        if gets((&raw mut line) as *mut ::core::ffi::c_char) == core::ptr::null_mut() {
            if feof(__iob_func().add(0)) != 0 || ferror(__iob_func().add(0)) != 0 {
                quit(0, core::ptr::null_mut());
            }
            break;
        }
        if line[0 as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            break;
        }
        makeargv();
        if margc == 0 {
            continue;
        }
        c = getcmd(margv[0 as usize]) as *mut cmd;
        if c == (-(1 as isize)) as *mut cmd {
            printf(b"?Ambiguous command\n\0".as_ptr() as *const ::core::ffi::c_char);
            continue;
        }
        if c == core::ptr::null_mut() {
            printf(b"?Invalid command\n\0".as_ptr() as *const ::core::ffi::c_char);
            continue;
        }
        if (*c).c_conn as ::core::ffi::c_int != 0 && connected == 0 {
            printf(b"Not connected.\n\0".as_ptr() as *const ::core::ffi::c_char);
            continue;
        }
        Some((*c).c_handler.unwrap_unchecked())
            .unwrap_unchecked()(
            margc,
            &raw mut margv as *mut *const ::core::ffi::c_char,
        );
        if bell != 0 && (*c).c_bell as ::core::ffi::c_int != 0 {
            putchar('\u{7}' as i32);
        }
        // Not a do-while condition: the C is `for (;;)` ending in this `break`, and its
        // `continue`s skip it too (scan_dowhile_continue baselines this site).
        if (*c).c_handler
            != Some(
                help as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            )
        {
            break;
        }
    }
    fflush(__iob_func().add(1));
}
#[no_mangle]
pub unsafe extern "C" fn getcmd(name: *const ::core::ffi::c_char) -> *mut cmd {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut q: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut c: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut found: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut nmatches: ::core::ffi::c_int = 0;
    let mut longest: ::core::ffi::c_int = 0;
    longest = 0;
    nmatches = 0;
    found = core::ptr::null_mut();
    c = core::ptr::addr_of_mut!(cmdtab) as *mut cmd;
    loop {
        p = (*c).c_name;
        if p.is_null() {
            break;
        }
        q = name;
        loop {
            let fresh23 = p;
            p = p.offset(1);
            if !(*q as ::core::ffi::c_int == *fresh23 as ::core::ffi::c_int) {
                break;
            }
            if *q as ::core::ffi::c_int == 0 {
                return c as *mut cmd;
            }
            q = q.offset(1);
        }
        if *q == 0 {
            if (q.offset_from(name) as ::core::ffi::c_int) > longest {
                longest = q.offset_from(name) as ::core::ffi::c_int;
                nmatches = 1;
                found = c;
            } else if (q.offset_from(name) as ::core::ffi::c_int) == longest {
                nmatches += 1;
            }
        }
        c = c.offset(1);
    }
    if nmatches > 1 {
        return (-(1)) as isize as *mut cmd;
    }
    return found as *mut cmd;
}
#[no_mangle]
pub unsafe extern "C" fn makeargv() {
    let mut argp: *mut *const i8;
    margc = 0;
    argp = core::ptr::addr_of_mut!(margv) as *mut *const i8;
    stringbase = core::ptr::addr_of_mut!(line) as *mut i8;
    argbase = core::ptr::addr_of_mut!(argbuf) as *mut i8;
    slrflag = 0;
    loop {
        let fresh15 = argp;
        argp = argp.add(1);
        *fresh15 = slurpstring();
        if (*fresh15).is_null() {
            break;
        }
        margc += 1;
    }
}

unsafe extern "C" fn slurpstring() -> *const core::ffi::c_char {
    let mut got_one: core::ffi::c_int = 0;
    let mut sb: *mut core::ffi::c_char = stringbase;
    let mut ap: *mut core::ffi::c_char = argbase;
    let mut tmp: *mut core::ffi::c_char = argbase;

    if *sb == b'!' as core::ffi::c_char || *sb == b'$' as core::ffi::c_char {
        match slrflag {
            0 => {
                slrflag += 1;
                stringbase = stringbase.add(1);
                return if *sb == b'!' as core::ffi::c_char {
                    b"!\0".as_ptr() as *const core::ffi::c_char
                } else {
                    b"$\0".as_ptr() as *const core::ffi::c_char
                };
            }
            1 => {
                slrflag += 1;
                altarg = stringbase;
            }
            _ => {}
        }
    }

    // The C's goto labels, as the states of this loop. Undefined, each name in a `match`
    // pattern would bind everything, so the first arm would swallow every state.
    const S0: core::ffi::c_int = 0;
    const S1: core::ffi::c_int = 1;
    const S2: core::ffi::c_int = 2;
    const S3: core::ffi::c_int = 3;
    const OUT1: core::ffi::c_int = 4;
    let mut at: core::ffi::c_int = S0;
    loop {
        match at {
            S0 => {
                match *sb as core::ffi::c_int {
                    0 => {
                        at = OUT1;
                    }
                    32 | 9 => {
                        sb = sb.add(1);
                        at = S0;
                    }
                    _ => {
                        match slrflag {
                            0 => {
                                slrflag += 1;
                            }
                            1 => {
                                slrflag += 1;
                                altarg = sb;
                            }
                            _ => {}
                        }
                        at = S1;
                    }
                }
            }
            S1 => {
                match *sb as core::ffi::c_int {
                    32 | 9 | 0 => {
                        at = OUT1;
                    }
                    92 => {
                        sb = sb.add(1);
                        at = S2;
                    }
                    34 => {
                        sb = sb.add(1);
                        at = S3;
                    }
                    _ => {
                        *ap = *sb;
                        ap = ap.add(1);
                        sb = sb.add(1);
                        got_one = 1;
                        at = S1;
                    }
                }
            }
            S2 => {
                match *sb as core::ffi::c_int {
                    0 => {
                        at = OUT1;
                    }
                    _ => {
                        *ap = *sb;
                        ap = ap.add(1);
                        sb = sb.add(1);
                        got_one = 1;
                        at = S1;
                    }
                }
            }
            S3 => {
                match *sb as core::ffi::c_int {
                    0 => {
                        at = OUT1;
                    }
                    34 => {
                        sb = sb.add(1);
                        at = S1;
                    }
                    _ => {
                        *ap = *sb;
                        ap = ap.add(1);
                        sb = sb.add(1);
                        got_one = 1;
                        at = S3;
                    }
                }
            }
            OUT1 => {
                break;
            }
            _ => {
                break;
            }
        }
    }

    if got_one != 0 {
        *ap = 0 as core::ffi::c_char;
        ap = ap.add(1);
    }
    argbase = ap;
    stringbase = sb;
    if got_one != 0 {
        return tmp as *const core::ffi::c_char;
    }
    match slrflag {
        0 => {
            slrflag += 1;
        }
        1 => {
            slrflag += 1;
            altarg = 0 as *mut core::ffi::c_char;
        }
        _ => {}
    }
    return 0 as *const core::ffi::c_char;
}

// uses: crate::ftp::main::{doglob, interactive, autologin, options, debug, verbose, trace,
//                          fromatty, typename, r#type, formname, form, modename, mode,
//                          structname, stru, bytename, bytesize, cpend, proxy, passivemode,
//                          crflag, home, portnum, getcmd, cmdscanner, cmdtab (extern [cmd; 0])}
//       crate::ftp::cmdtab::NCMDS
//       crate::ftp::cmds::setpeer
//       crate::prelude::{exit, fprintf, printf, putchar, fflush, strcpy, strcat, strlen, getenv,
//                        __iob_func, FILE, cmd, servent, WSADATA, WORD, MAKEWORD, SO_DEBUG,
//                        TYPE_A, FORM_N, MODE_S, STRU_F}
// needs (NOT yet declared anywhere in ftp-rust; msvcrt / ws2_32):
//       extern "C"      { fn _set_fmode(mode: c_int) -> c_int; }   // msvcrt.spec:905, all arches
//       extern "system" { fn WSAStartup(wVersionRequested: WORD, lpWSAData: LPWSADATA) -> c_int;
//                         fn getservbyname(name: *const c_char, proto: *const c_char) -> *mut servent; }

// hand-ported: both setjmp(toplevel) sites are dead (intr is never installed), so the C's
// `top = setjmp(toplevel) == 0` is the constant 1 and `if (setjmp(toplevel)) exit(0)` never fires.
#[no_mangle]
pub unsafe extern "C" fn main(mut argc: ::core::ffi::c_int, argv: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    // The C's argv is `const char *argv[]`.
    let mut argv: *mut *const ::core::ffi::c_char = argv as *mut *const ::core::ffi::c_char;
    let mut cp: *const ::core::ffi::c_char;
    let mut top: ::core::ffi::c_int;
    let err: ::core::ffi::c_int;
    let w_ver_req: WORD;
    let mut wsa_data: WSADATA = ::core::mem::zeroed();
    let sp: *mut servent;

    // `_fmode = O_BINARY;` -- _fmode is a msvcrt DATA export, which an extern Rust static cannot
    // import without dllimport; _set_fmode stores the same MSVCRT__fmode (O_BINARY = 0x8000).
    _set_fmode(0x8000);

    w_ver_req = MAKEWORD!(1, 1);

    err = WSAStartup(w_ver_req, &mut wsa_data);
    if err != 0 {
        fprintf(
            (__iob_func() as *mut FILE).offset(2) as *mut FILE,
            b"Could not initialize Windows socket interface.\0".as_ptr() as *const ::core::ffi::c_char,
        );
        exit(1);
    }

    sp = getservbyname(
        b"ftp\0".as_ptr() as *const ::core::ffi::c_char,
        b"tcp\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if sp.is_null() {
        fprintf(
            (__iob_func() as *mut FILE).offset(2) as *mut FILE,
            b"ftp: ftp/tcp: unknown service\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        exit(1);
    }

    portnum = (*sp).s_port;

    doglob = 1;
    interactive = 1;
    autologin = 1;
    argc -= 1;
    argv = argv.offset(1);
    while argc > 0 && **argv as ::core::ffi::c_int == '-' as i32 {
        cp = (*argv).offset(1);
        while *cp as ::core::ffi::c_int != 0 {
            match *cp as u8 {
                b'd' => {
                    options |= SO_DEBUG as ::core::ffi::c_int;
                    debug += 1;
                }
                b'v' => {
                    verbose += 1;
                }
                b't' => {
                    trace += 1;
                }
                b'i' => {
                    interactive = 0;
                }
                b'n' => {
                    autologin = 0;
                }
                b'g' => {
                    doglob = 0;
                }
                _ => {
                    fprintf(
                        (__iob_func() as *mut FILE).offset(1) as *mut FILE,
                        b"ftp: %c: unknown option\n\0".as_ptr() as *const ::core::ffi::c_char,
                        *cp as ::core::ffi::c_int,
                    );
                    exit(1);
                }
            }
            cp = cp.offset(1);
        }
        argc -= 1;
        argv = argv.offset(1);
    }
    fromatty = 1; // Strengthen this test
    // Set up defaults for FTP.
    strcpy(addr_of_mut!(typename) as *mut ::core::ffi::c_char, b"ascii\0".as_ptr() as *const ::core::ffi::c_char);
    r#type = TYPE_A as ::core::ffi::c_int;
    strcpy(addr_of_mut!(formname) as *mut ::core::ffi::c_char, b"non-print\0".as_ptr() as *const ::core::ffi::c_char);
    form = FORM_N as ::core::ffi::c_int;
    strcpy(addr_of_mut!(modename) as *mut ::core::ffi::c_char, b"stream\0".as_ptr() as *const ::core::ffi::c_char);
    mode = MODE_S as ::core::ffi::c_int;
    strcpy(addr_of_mut!(structname) as *mut ::core::ffi::c_char, b"file\0".as_ptr() as *const ::core::ffi::c_char);
    stru = STRU_F as ::core::ffi::c_int;
    strcpy(addr_of_mut!(bytename) as *mut ::core::ffi::c_char, b"8\0".as_ptr() as *const ::core::ffi::c_char);
    bytesize = 8;
    if fromatty != 0 {
        verbose += 1;
    }
    cpend = 0; /* no pending replies */
    proxy = 0; /* proxy not active */
    passivemode = 1; /* passive mode *is* active */
    crflag = 1; /* strip c.r. on ascii gets */
    // Set up the home directory in case we're globbing.
    cp = getenv(b"SystemDrive\0".as_ptr() as *const ::core::ffi::c_char);
    if !cp.is_null() && *cp as ::core::ffi::c_int != 0 {
        strcpy(addr_of_mut!(home) as *mut ::core::ffi::c_char, cp);
        strcat(addr_of_mut!(home) as *mut ::core::ffi::c_char, b"/\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        strcpy(addr_of_mut!(home) as *mut ::core::ffi::c_char, b"C:/\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if argc > 0 {
        // `if (setjmp(toplevel)) exit(0);` dropped: intr is never installed (signal() is
        // commented out), so nothing longjmps to toplevel and the exit(0) arm is unreachable.
        setpeer(argc + 1, argv.offset(-1));
    }
    // `top = setjmp(toplevel) == 0;` -- setjmp only ever returns 0 here (see above), so top = 1.
    top = 1;
    loop {
        cmdscanner(top);
        top = 1;
    }
}

// hand-ported: column layout walks cmdtab by raw pointer against NCMDS and compares getcmd's
// `(struct cmd *)-1` sentinel; HELPINDENT = sizeof("directory") = 10 (main.c:529).
#[no_mangle]
pub unsafe extern "C" fn help(mut argc: ::core::ffi::c_int, mut argv: *mut *const ::core::ffi::c_char) {
    let base: *mut cmd = addr_of_mut!(cmdtab) as *mut cmd;
    let end: *mut cmd = base.offset(NCMDS as isize);
    let mut c: *mut cmd;

    if argc == 1 {
        let mut i: ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int;
        let mut w: ::core::ffi::c_int;
        let mut k: ::core::ffi::c_int;
        let mut columns: ::core::ffi::c_int;
        let mut width: ::core::ffi::c_int = 0;
        let lines: ::core::ffi::c_int;

        printf(b"Commands may be abbreviated.  Commands are:\n\n\0".as_ptr() as *const ::core::ffi::c_char);
        c = base;
        while c < end {
            let len: ::core::ffi::c_int = strlen((*c).c_name) as ::core::ffi::c_int;
            if len > width {
                width = len;
            }
            c = c.offset(1);
        }
        width = (width + 8) & !7;
        columns = 80 / width;
        if columns == 0 {
            columns = 1;
        }
        lines = (NCMDS + columns - 1) / columns;
        i = 0;
        while i < lines {
            j = 0;
            while j < columns {
                c = base.offset((j * lines + i) as isize);
                if !(*c).c_name.is_null() && (proxy == 0 || (*c).c_proxy as ::core::ffi::c_int != 0) {
                    printf(b"%s\0".as_ptr() as *const ::core::ffi::c_char, (*c).c_name);
                } else if !(*c).c_name.is_null() {
                    k = 0;
                    while k < strlen((*c).c_name) as ::core::ffi::c_int {
                        putchar(' ' as i32);
                        k += 1;
                    }
                }
                if c.offset(lines as isize) >= end {
                    printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                    break;
                }
                w = strlen((*c).c_name) as ::core::ffi::c_int;
                while w < width {
                    w = (w + 8) & !7;
                    putchar('\t' as i32);
                }
                j += 1;
            }
            i += 1;
        }
        fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
        return;
    }
    loop {
        argc -= 1;
        if argc <= 0 {
            break;
        }
        argv = argv.offset(1);
        let arg: *const ::core::ffi::c_char = *argv;
        c = getcmd(arg);
        if c as isize == -1 {
            printf(b"?Ambiguous help command %s\n\0".as_ptr() as *const ::core::ffi::c_char, arg);
        } else if c.is_null() {
            printf(b"?Invalid help command %s\n\0".as_ptr() as *const ::core::ffi::c_char, arg);
        } else {
            printf(
                b"%-*s\t%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                HELPINDENT as ::core::ffi::c_int,
                (*c).c_name,
                (*c).c_help,
            );
        }
    }
    fflush((__iob_func() as *mut FILE).offset(1) as *mut FILE);
}
