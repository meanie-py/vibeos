use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// (C forward declarations `struct protocol;` etc. — the real layouts come from the bindgen
// `types` island via the prelude; empty Rust placeholders here would shadow them.)

pub static mut protocols: *mut protocol = core::ptr::null_mut();
pub static mut timeouts: *mut timeout = core::ptr::null_mut();
static mut free_timeouts: *mut timeout = core::ptr::null_mut();
#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut bootp_packet_handler: Option<unsafe extern "C" fn(*mut interface_info, *mut dhcp_packet, i32, u32, iaddr, *mut hardware)> = None;

// ---- dispatch ----
#[no_mangle]
pub unsafe extern "system" fn dispatch(hStopEvent: HANDLE) {
    let mut count: i32;
    let mut to_msec: i32;
    let mut howlong: time_t;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    let mut Events: [HANDLE; 3] = [
        hAdapterStateChangedEvent,
        hStopEvent,
        core::ptr::null_mut(),
    ];
    let mut EventCount: i32 = 2;

    ApiLock();

    loop {
        time(&mut cur_time as *mut time_t);

        if !timeouts.is_null() {
            if (*timeouts).when <= cur_time {
                let t = timeouts;
                timeouts = (*timeouts).next;
                ((*t).func.unwrap_unchecked())((*t).what);
                (*t).next = free_timeouts;
                free_timeouts = t;
                continue;
            }
            howlong = (*timeouts).when - cur_time;
            if howlong > (i32::MAX as time_t) / 1000 {
                howlong = (i32::MAX as time_t) / 1000;
            }
            to_msec = (howlong * 1000) as i32;
        } else {
            to_msec = INFINITE as i32;
        }

        if Events[2].is_null() && DhcpSocket != INVALID_SOCKET {
            Events[2] = WSACreateEvent();
            if !Events[2].is_null() {
                count = WSAEventSelect(DhcpSocket, Events[2], 0x21);
                if count != (NO_ERROR) as i32 {
                    WSACloseEvent(Events[2]);
                    Events[2] = core::ptr::null_mut();
                } else {
                    EventCount = 3;
                }
            }
        } else if !Events[2].is_null() && DhcpSocket == INVALID_SOCKET {
            WSACloseEvent(Events[2]);
            Events[2] = core::ptr::null_mut();
            EventCount = 2;
        }

        ApiUnlock();
        count = WaitForMultipleObjects(
            EventCount as u32,
            Events.as_ptr(),
            0,
            to_msec as u32,
        ) as i32;
        ApiLock();

        if count == WAIT_OBJECT_0 as i32 {
            continue;
        } else if count == WAIT_OBJECT_0 as i32 + 1 {
            DH_DbgPrint!(MID_TRACE, b"Dispatch thread stop event!\n\0".as_ptr() as *const i8);
            break;
        } else if count == WAIT_OBJECT_0 as i32 + 2 {
            WSAResetEvent(Events[2]);
        } else {
            continue;
        }

        let mut l = protocols;
        while !l.is_null() {
            let ip = (*l).local as *mut interface_info;
            if !ip.is_null()
                && ((*l).handler != Some(got_one as unsafe extern "C" fn(*mut protocol)) || (*ip).dead == 0)
            {
                DH_DbgPrint!(
                    MID_TRACE,
                    b"Handling %x\n\0".as_ptr() as *const i8,
                    l as u32,
                );
                ((*l).handler.unwrap_unchecked())(l);
            }
            l = (*l).next;
        }
    }

    WSACloseEvent(Events[2]);
    ApiUnlock();
    DH_DbgPrint!(MID_TRACE, b"Dispatch thread stopped!\n\0".as_ptr() as *const i8);
}

// ---- add_timeout ----
pub unsafe fn add_timeout(when: time_t, where_: unsafe extern "C" fn(*mut core::ffi::c_void), what: *mut core::ffi::c_void) {
    let mut t: *mut _timeout;
    let mut q: *mut _timeout;

    // Debug print (MID_TRACE constant not available; using 0)
    DH_DbgPrint!(0, b"Adding timeout %x %p %x\n\0".as_ptr() as *const i8, when, where_ as *const core::ffi::c_void, what);

    // See if this timeout supersedes an existing timeout
    t = core::ptr::null_mut();
    q = timeouts;
    while !q.is_null() {
        if (*q).func == Some(where_) && (*q).what == what {
            if !t.is_null() {
                (*t).next = (*q).next;
            } else {
                timeouts = (*q).next;
            }
            break;
        }
        t = q;
        q = (*q).next;
    }

    // If we didn't supersede a timeout, allocate a timeout structure now
    if q.is_null() {
        if !free_timeouts.is_null() {
            q = free_timeouts;
            free_timeouts = (*q).next;
            (*q).func = Some(where_);
            (*q).what = what;
        } else {
            q = (core::mem::size_of::<_timeout>() as *mut c_void) as *mut timeout;
            q = malloc((q) as usize) as *mut _timeout;
            if q.is_null() {
                error(b"Can't allocate timeout structure!\0".as_ptr() as *const i8);
                return;
            }
            (*q).func = Some(where_);
            (*q).what = what;
        }
    }

    (*q).when = when;

    // Now sort this timeout into the timeout list

    // Beginning of list?
    if timeouts.is_null() || (*timeouts).when > (*q).when {
        (*q).next = timeouts;
        timeouts = q;
        return;
    }

    // Middle of list?
    t = timeouts;
    while !(*t).next.is_null() {
        if (*(*t).next).when > (*q).when {
            (*q).next = (*t).next;
            (*t).next = q;
            return;
        }
        t = (*t).next;
    }

    // End of list
    (*t).next = q;
    (*q).next = core::ptr::null_mut();
}

// ---- cancel_timeout ----
pub unsafe fn cancel_timeout(where_: unsafe extern "C" fn(*mut core::ffi::c_void), what: *mut core::ffi::c_void) {
    let mut t: *mut timeout = core::ptr::null_mut();
    let mut q: *mut timeout = timeouts;
    while !q.is_null() {
        if (*q).func == Some(where_) && (*q).what == what {
            break;
        }
        t = q;
        q = (*q).next;
    }
    if !q.is_null() {
        if !t.is_null() {
            (*t).next = (*q).next;
        } else {
            timeouts = (*q).next;
        }
        (*q).next = free_timeouts;
        free_timeouts = q;
    }
}

// ---- add_protocol ----
pub unsafe fn add_protocol(name: *mut i8, fd: i32, handler: unsafe extern "C" fn(*mut protocol), local: *mut core::ffi::c_void) {
    let p = malloc(core::mem::size_of::<protocol>()) as *mut protocol;
    if p.is_null() {
        error(b"can't allocate protocol struct for %s\0".as_ptr() as *const i8, name);
    }
    (*p).fd = fd;
    (*p).handler = Some(handler);
    (*p).local = local;
    (*p).next = protocols;
    protocols = p;
}

// ---- remove_protocol ----
pub unsafe fn remove_protocol(proto: *mut protocol) {
    let mut p: *mut protocol;
    let mut next: *mut protocol;
    let mut prev: *mut protocol;
    let ip: *mut interface_info = ((*proto).local) as *mut interface_info;
    let mut t: *mut timeout;
    let mut q: *mut timeout;
    let mut u: *mut timeout;

    t = core::ptr::null_mut();
    q = timeouts;
    while q != core::ptr::null_mut() {
        if (*q).what == (ip) as *mut c_void {
            if t != core::ptr::null_mut() {
                (*t).next = (*q).next;
            } else {
                timeouts = (*q).next;
            }
            u = (*q).next;
            (*q).next = free_timeouts;
            free_timeouts = q;
        } else {
            u = (*q).next;
            t = q;
        }
        q = u;
    }

    prev = core::ptr::null_mut();
    p = protocols;
    while p != core::ptr::null_mut() {
        next = (*p).next;
        if p == proto {
            if prev != core::ptr::null_mut() {
                (*prev).next = (*p).next;
            } else {
                protocols = (*p).next;
            }
            free(p as *mut core::ffi::c_void);
        } else {
            prev = p;
        }
        p = next;
    }
}

// ---- find_protocol_by_adapter ----
pub unsafe fn find_protocol_by_adapter(info: *mut interface_info) -> *mut protocol {
    let mut p = protocols;
    while !p.is_null() {
        if (*p).local as *mut core::ffi::c_void == info as *mut core::ffi::c_void {
            return p;
        }
        p = (*p).next;
    }
    core::ptr::null_mut()
}

// ---- interface_link_status ----
unsafe fn interface_link_status(ifname: *const i8) -> i32 {
    1
}

// ---- got_one ----
pub unsafe extern "C" fn got_one(l: *mut protocol) {
    let mut from: sockaddr_in = core::mem::zeroed();
    let mut hfrom: hardware = core::mem::zeroed();
    let mut ifrom: iaddr = core::mem::zeroed();
    let mut packbuf: [u8; 4095] = [0; 4095];
    let ip: *mut interface_info = ((*l).local) as *mut interface_info;
    let adapter: *mut DHCP_ADAPTER;

    let result = receive_packet(
        ip,
        packbuf.as_mut_ptr(),
        (core::mem::size_of::<[u8; 4095]>() as u32) as usize,
        &mut from,
        &mut hfrom,
    );
    if result == -1 {
        warning(
            b"receive_packet failed on %s: %d\0".as_ptr() as *const i8,
            (*ip).name.as_ptr(),
            WSAGetLastError(),
        );
        (*ip).errors += 1;
        if (*ip).errors > 20 {
            warning(
                b"Interface %s no longer appears valid.\0".as_ptr() as *const i8,
                (*ip).name.as_ptr(),
            );
            (*ip).dead = 1;
            closesocket(((*l).fd) as usize);
            remove_protocol(l);
            let adapter = AdapterFindInfo(ip);
            if !adapter.is_null() {
                RemoveEntryList(&mut (*adapter).ListEntry);
                free(adapter as *mut core::ffi::c_void);
            }
        }
        return;
    }
    if result == 0 {
        return;
    }

    if let Some(handler) = bootp_packet_handler {
        ifrom.len = 4;
        core::ptr::copy_nonoverlapping(
            &from.sin_addr as *const _ as *const u8,
            ifrom.iabuf.as_mut_ptr(),
            4,
        );

        let packet: *mut dhcp_packet = packbuf.as_mut_ptr() as *mut dhcp_packet;
        let adapter = AdapterFindByHardwareAddress((*packet).chaddr.as_mut_ptr(), (*packet).hlen);
        if adapter.is_null() {
            warning(
                b"Discarding packet with a non-matching target physical address\n\0".as_ptr()
                    as *const i8,
            );
            return;
        }

        handler(
            &mut (*adapter).DhclientInfo,
            packet,
            result as i32,
            (from.sin_port) as u32,
            ifrom,
            &mut hfrom,
        );
    }
}
