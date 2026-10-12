use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static mut lflag: core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn netfinger(mut name: *mut ::core::ffi::c_char) {
    let mut c: core::ffi::c_char = 0;
    let mut lastc: core::ffi::c_char = 0;
    let mut defaddr: in_addr = core::mem::zeroed::<in_addr>();
    let mut hp: *mut hostent = core::ptr::null_mut::<hostent>();
    let mut def: hostent = core::mem::zeroed::<hostent>();
    let mut sp: *mut servent = core::ptr::null_mut::<servent>();
    let mut sin: sockaddr_in = core::mem::zeroed::<sockaddr_in>();
    let mut s: SOCKET = 0;
    let mut alist: [*mut core::ffi::c_char; 1] =
        [core::ptr::null_mut::<core::ffi::c_char>(); 1];
    let mut host: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();

    /* If this is a local request */
    host = rindex!(name as *const u8, '@' as core::ffi::c_int) as *mut core::ffi::c_char;
    if host.is_null() {
        return;
    }

    *host = '\0' as core::ffi::c_char;
    host = host.add(1);
    if isdigit(*host as core::ffi::c_int) != 0 && {
        defaddr.S_un.S_addr = inet_addr(host) as ULONG;
        defaddr.S_un.S_addr
            != (-(1 as core::ffi::c_int)) as ULONG
    } {
        def.h_name = host;
        def.h_addr_list = core::ptr::addr_of_mut!(alist) as *mut *mut core::ffi::c_char;
        *def.h_addr_list.offset(0) = core::ptr::addr_of_mut!(defaddr) as *mut core::ffi::c_char;
        def.h_length = core::mem::size_of::<in_addr>() as core::ffi::c_short;
        def.h_addrtype = AF_INET as core::ffi::c_short;
        def.h_aliases = 0 as *mut *mut core::ffi::c_char;
        hp = core::ptr::addr_of_mut!(def);
    } else {
        hp = gethostbyname(host);
        if hp.is_null() {
            fprintf(
                (__iob_func()).offset(2) as *mut FILE,
                b"finger: unknown host: %s\n\0".as_ptr() as *const core::ffi::c_char,
                host,
            );
            return;
        }
    }

    sp = getservbyname(
        b"finger\0".as_ptr() as *const core::ffi::c_char,
        b"tcp\0".as_ptr() as *const core::ffi::c_char,
    );
    if sp.is_null() {
        fprintf(
            (__iob_func()).offset(2) as *mut FILE,
            b"finger: tcp/finger: unknown service\n\0".as_ptr() as *const core::ffi::c_char,
        );
        return;
    }
    sin.sin_family = ((*hp).h_addrtype as USHORT) as i16;
    bcopy!(
        (*(*hp).h_addr_list.offset(0) as *const core::ffi::c_char).cast(),
        (core::ptr::addr_of_mut!(sin.sin_addr) as *mut core::ffi::c_char).cast(),
        ((*hp).h_length as core::ffi::c_int) as usize,
    );
    sin.sin_port = (*sp).s_port as USHORT;
    s = socket(
        (*hp).h_addrtype as core::ffi::c_int,
        SOCK_STREAM as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    if s == INVALID_SOCKET as SOCKET {
        perror(b"finger: socket\0".as_ptr() as *const core::ffi::c_char);
        return;
    }

    /* have network connection; identify the host connected with */
    printf(
        b"[%s]\n\0".as_ptr() as *const core::ffi::c_char,
        (*hp).h_name,
    );
    if connect(
        s,
        core::ptr::addr_of_mut!(sin) as *mut sockaddr,
        core::mem::size_of::<sockaddr_in>() as core::ffi::c_int,
    ) < 0 as core::ffi::c_int
    {
        fprintf(
            (__iob_func()).offset(2) as *mut FILE,
            b"finger: connect rc = %d\0".as_ptr() as *const core::ffi::c_char,
            WSAGetLastError(),
        );
        closesocket(s);
        return;
    }

    /* -l flag for remote fingerd  */
    if lflag != 0 {
        send(
            s,
            b"/W \0".as_ptr() as *const core::ffi::c_char,
            3 as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
    }
    /* send the name followed by <CR><LF> */
    send(
        s,
        name,
        strlen(name) as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    send(
        s,
        b"\r\n\0".as_ptr() as *const core::ffi::c_char,
        2 as core::ffi::c_int,
        0 as core::ffi::c_int,
    );

    /*
     * Read from the remote system; once we're connected, we assume some
     * data.  If none arrives, we hang until the user interrupts.
     *
     * If we see a <CR> or a <CR> with the high bit set, treat it as
     * a newline; if followed by a newline character, only output one
     * newline.
     *
     * Otherwise, all high bits are stripped; if it isn't printable and
     * it isn't a space, we can simply set the 7th bit.  Every ASCII
     * character with bit 7 set is printable.
     */
    lastc = 0 as core::ffi::c_char;
    while recv(
        s,
        core::ptr::addr_of_mut!(c) as *mut core::ffi::c_char,
        1 as core::ffi::c_int,
        0 as core::ffi::c_int,
    ) == 1 as core::ffi::c_int
    {
        c = ((c as core::ffi::c_int) & (0x7f as core::ffi::c_int))
            as core::ffi::c_char;
        if c as core::ffi::c_int == 0x0d as core::ffi::c_int {
            if lastc as core::ffi::c_int == '\r' as core::ffi::c_int {
                /* ^M^M - skip dupes */
                continue;
            }
            c = '\n' as core::ffi::c_int as core::ffi::c_char;
            lastc = '\r' as core::ffi::c_int as core::ffi::c_char;
        } else {
            if isprint(c as core::ffi::c_int) == 0
                && isspace(c as core::ffi::c_int) == 0
            {
                c = ((c as core::ffi::c_int) | (0x40 as core::ffi::c_int))
                    as core::ffi::c_char;
            }
            if lastc as core::ffi::c_int != '\r' as core::ffi::c_int
                || c as core::ffi::c_int != '\n' as core::ffi::c_int
            {
                lastc = c;
            } else {
                lastc = '\n' as core::ffi::c_int as core::ffi::c_char;
                continue;
            }
        }
        putchar(c as core::ffi::c_int);
    }
    if lastc as core::ffi::c_int != '\n' as core::ffi::c_int {
        putchar('\n' as core::ffi::c_int);
    }
    putchar('\n' as core::ffi::c_int);
    closesocket(s);
}
