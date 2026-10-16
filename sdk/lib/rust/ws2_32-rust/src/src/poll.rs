//! `WSAPoll` (winsock2.h) built on `select()`.
//!
//! Windows' WSAPoll is a thin wrapper over the AFD poll ioctl; this port's AFD has no poll, so the
//! request is expressed as three fd_sets (read / write / except) and run through `select` — the
//! same way the pre-Vista emulations did. Mapping:
//!   * POLLRDNORM -> readfds; a read-ready TCP stream is then probed with `recv(MSG_PEEK)` to
//!     tell data (POLLRDNORM) from a graceful close (POLLHUP) or a reset (POLLHUP|POLLERR);
//!     listening sockets (accept ready) and datagram sockets report plain POLLRDNORM.
//!   * POLLWRNORM -> writefds (also fires when a non-blocking connect completes).
//!   * every valid socket is watched in exceptfds: that is a failed non-blocking connect ->
//!     POLLERR, or, if POLLRDBAND was requested and SIOCATMARK shows pending urgent data,
//!     POLLRDBAND.
//!   * invalid / negative sockets -> POLLNVAL (not counted, as Wine); if none is valid the call
//!     fails with WSAENOTSOCK.
//!
//! Known limits of the select emulation: a graceful close (FIN) is detected only when POLLRDNORM
//! was requested for that socket (otherwise merely watching for it would wake the call on
//! unrelated data); POLLPRI / POLLWRBAND are ignored (Winsock does not support them either).

use crate::prelude::*;
use crate::src::ioctl::ioctlsocket;
use crate::src::recv::recv;
use crate::src::select::select;
use crate::src::sockctrl::getsockopt;
use crate::types::*;
use core::ffi::c_void;

pub const POLLERR: i16 = 0x0001;
pub const POLLHUP: i16 = 0x0002;
pub const POLLNVAL: i16 = 0x0004;
pub const POLLWRNORM: i16 = 0x0010;
pub const POLLRDNORM: i16 = 0x0100;
pub const POLLRDBAND: i16 = 0x0200;

const E_NOTSOCK: u32 = 10038;
const E_INVAL: u32 = 10022;
const E_FAULT: u32 = 10014;
const E_NOBUFS: u32 = 10055;
const E_MSGSIZE: u32 = 10040;
const E_NOTCONN: u32 = 10057;
const E_CONNABORTED: u32 = 10053;
const E_CONNRESET: u32 = 10054;
const E_SHUTDOWN: u32 = 10058;
const E_TIMEDOUT: u32 = 10060;
const E_NETRESET: u32 = 10052;
const HEAP_ZERO_MEMORY: u32 = 0x8;
const SIOCATMARK: i32 = 0x4004_7307u32 as i32;

/// C: `WSAPOLLFD` (winsock2.h). Measured: 16 bytes on x86_64, 8 on i686.
#[repr(C)]
pub struct WSAPOLLFD {
    pub fd: SOCKET,
    pub events: i16,
    pub revents: i16,
}

const _: () = assert!(core::mem::offset_of!(WSAPOLLFD, fd) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<WSAPOLLFD>() == 16);
    assert!(core::mem::offset_of!(WSAPOLLFD, events) == 8);
    assert!(core::mem::offset_of!(WSAPOLLFD, revents) == 10);
};
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<WSAPOLLFD>() == 8);
    assert!(core::mem::offset_of!(WSAPOLLFD, events) == 4);
    assert!(core::mem::offset_of!(WSAPOLLFD, revents) == 6);
};

unsafe fn set_add(set: *mut fd_set, s: SOCKET) {
    let n = (*set).fd_count as usize;
    *(core::ptr::addr_of_mut!((*set).fd_array) as *mut SOCKET).add(n) = s;
    (*set).fd_count = (n + 1) as u32;
}

unsafe fn set_has(set: *const fd_set, s: SOCKET) -> bool {
    if set.is_null() {
        return false;
    }
    let arr = core::ptr::addr_of!((*set).fd_array) as *const SOCKET;
    for i in 0..(*set).fd_count as usize {
        if *arr.add(i) == s {
            return true;
        }
    }
    false
}

/// Alloc one zeroed fd_set able to hold `count` sockets.
unsafe fn alloc_set(count: usize) -> *mut fd_set {
    let bytes = core::mem::size_of::<fd_set>() + count * core::mem::size_of::<SOCKET>();
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, bytes as SIZE_T) as *mut fd_set
}

/// Classifies a read-ready socket: returns the revents bits for it.
unsafe fn classify_readable(s: SOCKET) -> i16 {
    let mut ty: i32 = 0;
    let mut len: i32 = 4;
    if getsockopt(s, SOL_SOCKET as i32, SO_TYPE as i32, &mut ty as *mut i32 as *mut i8, &mut len) != 0
        || ty != SOCK_STREAM as i32
    {
        return POLLRDNORM;
    }
    let mut b: i8 = 0;
    let r = recv(s, &mut b, 1, MSG_PEEK as i32);
    if r > 0 {
        return POLLRDNORM;
    }
    if r == 0 {
        return POLLHUP; // graceful close
    }
    match GetLastError() {
        E_CONNRESET | E_CONNABORTED | E_SHUTDOWN | E_NETRESET | E_TIMEDOUT => POLLHUP | POLLERR,
        // WSAEMSGSIZE (datagram larger than the peek), WSAENOTCONN (listening socket: a pending
        // accept) and anything else unexpected: the socket is "readable" for the caller.
        E_MSGSIZE | E_NOTCONN => POLLRDNORM,
        _ => POLLRDNORM,
    }
}

/// C: `int WSAPoll(LPWSAPOLLFD fdArray, ULONG fds, INT timeout)`. `timeout` in ms; negative waits
/// forever. Returns the number of entries with non-zero revents, 0 on timeout, SOCKET_ERROR on
/// failure.
#[no_mangle]
pub unsafe extern "system" fn WSAPoll(fdArray: *mut WSAPOLLFD, fds: u32, timeout: i32) -> i32 {
    if fds == 0 {
        SetLastError(E_INVAL);
        return SOCKET_ERROR;
    }
    if fdArray.is_null() {
        SetLastError(E_FAULT);
        return SOCKET_ERROR;
    }
    let n = fds as usize;
    let rs = alloc_set(n);
    let ws = alloc_set(n);
    let es = alloc_set(n);
    if rs.is_null() || ws.is_null() || es.is_null() {
        for p in [rs, ws, es] {
            if !p.is_null() {
                HeapFree(GetProcessHeap(), 0, p as PVOID);
            }
        }
        SetLastError(E_NOBUFS);
        return SOCKET_ERROR;
    }
    let free_all = |code: u32| -> i32 {
        HeapFree(GetProcessHeap(), 0, rs as PVOID);
        HeapFree(GetProcessHeap(), 0, ws as PVOID);
        HeapFree(GetProcessHeap(), 0, es as PVOID);
        SetLastError(code);
        SOCKET_ERROR
    };

    let mut valid = 0usize;
    for i in 0..n {
        let f = &mut *fdArray.add(i);
        if (f.fd as isize) < 0 {
            f.revents = POLLNVAL;
            continue;
        }
        let mut ty: i32 = 0;
        let mut len: i32 = 4;
        if getsockopt(f.fd, SOL_SOCKET as i32, SO_TYPE as i32, &mut ty as *mut i32 as *mut i8, &mut len) != 0 {
            let e = GetLastError();
            if e == E_NOTSOCK {
                f.revents = POLLNVAL;
                continue;
            }
            return free_all(e); // e.g. WSANOTINITIALISED
        }
        f.revents = 0;
        valid += 1;
        if f.events & POLLRDNORM != 0 {
            set_add(rs, f.fd);
        }
        if f.events & POLLWRNORM != 0 {
            set_add(ws, f.fd);
        }
        set_add(es, f.fd);
    }
    if valid == 0 {
        return free_all(E_NOTSOCK);
    }

    let tv = timeval {
        tv_sec: if timeout > 0 { timeout / 1000 } else { 0 },
        tv_usec: if timeout > 0 { (timeout % 1000) * 1000 } else { 0 },
    };
    let tvp: *const timeval = if timeout < 0 { core::ptr::null() } else { &tv };
    let rp = if (*rs).fd_count != 0 { rs } else { core::ptr::null_mut() };
    let wp = if (*ws).fd_count != 0 { ws } else { core::ptr::null_mut() };
    let r = select(0, rp, wp, es, tvp);
    if r == SOCKET_ERROR {
        let e = GetLastError();
        return free_all(e);
    }

    let mut count = 0i32;
    if r > 0 {
        for i in 0..n {
            let f = &mut *fdArray.add(i);
            if f.revents == POLLNVAL || (f.fd as isize) < 0 {
                continue;
            }
            let mut rev: i16 = 0;
            if f.events & POLLWRNORM != 0 && set_has(wp, f.fd) {
                rev |= POLLWRNORM;
            }
            if f.events & POLLRDNORM != 0 && set_has(rp, f.fd) {
                rev |= classify_readable(f.fd);
            }
            if set_has(es, f.fd) {
                let mut at_mark: u32 = 1;
                if f.events & POLLRDBAND != 0
                    && ioctlsocket(f.fd, SIOCATMARK, &mut at_mark) == 0
                    && at_mark == 0
                {
                    rev |= POLLRDBAND; // urgent data pending
                } else {
                    rev |= POLLERR; // failed non-blocking connect / socket error
                }
            }
            f.revents = rev;
            if rev != 0 {
                count += 1;
            }
        }
    }
    HeapFree(GetProcessHeap(), 0, rs as PVOID);
    HeapFree(GetProcessHeap(), 0, ws as PVOID);
    HeapFree(GetProcessHeap(), 0, es as PVOID);
    SetLastError(0);
    count
}
