use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut NICHOST: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"whois.internic.net\0")
};

#[no_mangle]
pub static mut host: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;

#[no_mangle]
pub static mut optset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const WIN32_NO_STATUS: () = ();

pub const _INC_WINDOWS: () = ();

#[no_mangle]
pub unsafe extern "C" fn getwhoisserver(argc: core::ffi::c_int, argv: *mut *mut core::ffi::c_char) {
	let mut i: core::ffi::c_int = 1;

    while i < argc {
		if strcmp(*argv.offset(i as isize), b"-h\0".as_ptr() as *const core::ffi::c_char) == 0
		{
			if i + 2 < argc
			{
				host = *argv.offset((i + 1) as isize);
				optset = i + 1;
			}
			else
			{
				optset = argc;
			}
			return;
		}
		i += 1;
	}
	host = (NICHOST).as_mut_ptr();
	optset = 1;
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: i32, argv: *mut *mut i8) -> i32 {
    let mut argc = argc;
    let mut argv = argv;
    let mut ch: i8 = 0;
    let mut sin: sockaddr_in = core::mem::zeroed();
    let mut hp: *mut hostent = core::ptr::null_mut();
    let mut sp: *mut servent = core::ptr::null_mut();
    let mut s: SOCKET = core::mem::zeroed();

    let mut wVersionRequested: WORD = core::mem::zeroed();
    let mut wsaData: WSADATA = core::mem::zeroed();
    let mut err: i32 = 0;

    getwhoisserver(argc, argv);
    argc -= optset;
    argv = argv.offset(optset as isize);

    if host.is_null() || argc == 0 {
        usage();
    }

    /* Start winsock */
    wVersionRequested = MAKEWORD!(1, 1);
    err = WSAStartup(wVersionRequested, core::ptr::addr_of_mut!(wsaData));
    if err != 0 {
        /* Tell the user that we couldn't find a usable */
        /* WinSock DLL.                                 */
        perror(b"whois: WSAStartup failed\0".as_ptr() as *const i8);
        cleanup(1);
    }

    hp = gethostbyname(host);
    if hp.is_null() {
        let _ = fprintf(__iob_func().add(2), b"whois: %s: \0".as_ptr() as *const i8, host);
        cleanup(1);
    }
    host = (*hp).h_name;

    s = socket((*hp).h_addrtype as _, (SOCK_STREAM as i32), 0);
    if s == INVALID_SOCKET {
        perror(b"whois: socket\0".as_ptr() as *const i8);
        cleanup(1);
    }

    memset((core::ptr::addr_of_mut!(sin) as *mut i8).cast(), 0, core::mem::size_of::<sockaddr_in>());
    sin.sin_family = (*hp).h_addrtype as _;
    if bind(s, core::ptr::addr_of_mut!(sin) as *mut sockaddr, core::mem::size_of::<sockaddr_in>() as i32) < 0 {
        perror(b"whois: bind\0".as_ptr() as *const i8);
        cleanup(1);
    }

    memcpy((core::ptr::addr_of_mut!(sin.sin_addr) as *mut i8).cast(), (*(*hp).h_addr_list).cast(), (*hp).h_length as _);
    sp = getservbyname(b"nicname\0".as_ptr() as *const i8, b"tcp\0".as_ptr() as *const i8);
    if sp.is_null() {
        let _ = fprintf(__iob_func().add(2), b"whois: nicname/tcp: unknown service\n\0".as_ptr() as *const i8);
        cleanup(1);
    }

    sin.sin_port = ((*sp).s_port) as u16;

    /* have network connection; identify the host connected with */
    let _ = printf(b"[%s]\n\0".as_ptr() as *const i8, (*hp).h_name);

    if connect(s, core::ptr::addr_of_mut!(sin) as *mut sockaddr, core::mem::size_of::<sockaddr_in>() as i32) < 0 {
        fprintf(__iob_func().add(2), b"whois: connect error = %d\n\0".as_ptr() as *const i8, WSAGetLastError());
        cleanup(1);
    }

    /* WinSock doesn't allow using a socket as a file descriptor. */
    /* Have to use send() and recv().  whois will drop connection. */

    /* For each request */
    loop {
        let old = argc;
        argc -= 1;
        if !(old > 1) {
            break;
        }
        /* Send the request */
        send(s, *argv, (strlen(*argv)) as i32, 0);
        let _ = send(s, b" \0".as_ptr() as *const i8, 1, 0);
        argv = argv.add(1);
    }
    /* Send the last request */
    send(s, *argv, (strlen(*argv)) as i32, 0);
    let _ = send(s, b"\r\n\0".as_ptr() as *const i8, 2, 0);

    /* Receive anything and print it */
    while recv(s, core::ptr::addr_of_mut!(ch), 1, 0) == 1 {
        let _ = putchar(ch as i32);
    }

    cleanup(0);
    return 0;
}

unsafe extern "C" fn usage() {
    let _ = fprintf(__iob_func().add(2), b"usage: whois [-h hostname] name ...\n\0".as_ptr() as *const i8);
    cleanup(1);
}

unsafe extern "C" fn cleanup(iExitCode: i32) {
    WSACleanup();
    exit(iExitCode);
}
