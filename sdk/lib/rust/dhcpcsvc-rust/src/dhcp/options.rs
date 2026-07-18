use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const DHCP_OPTION_DATA: () = ();
pub static mut bad_options: i32 = 0;
pub static mut bad_options_max: i32 = 5;

// ---- parse_option_buffer ----
unsafe fn parse_option_buffer(packet: *mut _packet, buffer: *mut u8, length: i32) {
    let mut s = buffer;
    let end = buffer.offset(length as isize);
    while s < end && *s != (DHO_END) as u8 {
        let code = *s as i32;
        if code == (DHO_PAD) as i32 {
            s = s.offset(1);
            continue;
        }
        let len: i32;
        if s.offset(2) > end {
            len = 65536;
        } else {
            len = *s.offset(1) as i32;
        }
        if len == 65536 || s.offset(2 + len as isize) > end {
            bad_options += 1;
            warning(
                (b"option %s (%d) %s.\0".as_ptr()) as *const i8,
                dhcp_options[code as usize].name,
                len,
                b"larger than buffer\0".as_ptr(),
            );
            if bad_options == bad_options_max {
                (*packet).options_valid = 1;
                bad_options = 0;
                warning(
                    (b"Many bogus options seen in offers. Taking this offer in spite of bogus options - hope for the best!\0".as_ptr()) as *const i8,
                );
            } else {
                warning((b"rejecting bogus offer.\0".as_ptr()) as *const i8);
                (*packet).options_valid = 0;
            }
            return;
        }
        let opt = &mut (*packet).options[code as usize];
        if (*opt).data.is_null() {
            let t = calloc(1, (len + 1) as usize) as *mut u8;
            if t.is_null() {
                error(
                    (b"Can't allocate storage for option %s.\0".as_ptr()) as *const i8,
                    dhcp_options[code as usize].name,
                );
                return;
            }
            memcpy(t as *mut c_void, s.offset(2) as *const c_void, len as usize);
            *t.offset(len as isize) = 0;
            (*opt).len = len;
            (*opt).data = t;
        } else {
            let new_len = len + (*opt).len as i32 + 1;
            let t = calloc(1, new_len as usize) as *mut u8;
            if t.is_null() {
                error(
                    (b"Can't expand storage for option %s.\0".as_ptr()) as *const i8,
                    dhcp_options[code as usize].name,
                );
                return;
            }
            memcpy(t as *mut c_void, (*opt).data as *const c_void, (*opt).len as usize);
            memcpy(t.offset((*opt).len as isize) as *mut c_void, s.offset(2) as *const c_void, len as usize);
            (*opt).len += len;
            *t.offset((*opt).len as isize) = 0;
            free((*opt).data as *mut core::ffi::c_void);
            (*opt).data = t;
        }
        s = s.offset((2 + len) as isize);
    }
    (*packet).options_valid = 1;
}

// ---- store_options ----
unsafe fn store_options(
    buffer: *mut u8,
    buflen: i32,
    options: *mut *mut tree_cache,
    priority_list: *mut u8,
    priority_len: i32,
    first_cutoff: i32,
    second_cutoff: i32,
) -> i32 {
    let mut bufix: i32 = 0;
    let mut option_stored: [i32; 256] = [0; 256];
    core::ptr::write_bytes(
        option_stored.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<[i32; 256]>(),
    );

    for i in 0..priority_len {
        let code = *priority_list.offset(i as isize) as i32;
        let optstart: i32;
        let mut length: i32;

        let opt_ptr = *options.offset(code as isize);
        if opt_ptr.is_null() {
            continue;
        }

        if option_stored[code as usize] != 0 {
            continue;
        }
        option_stored[code as usize] = 1;

        length = (*opt_ptr).len;

        let mut ix: i32 = 0;
        optstart = bufix;
        while length > 0 {
            let mut incr: i32 = if length > 255 { 255 } else { length };

            if bufix < first_cutoff && bufix + incr > first_cutoff {
                incr = first_cutoff - bufix;
            } else if bufix < second_cutoff && bufix + incr > second_cutoff {
                incr = second_cutoff - bufix;
            }

            if bufix + 2 + incr > buflen {
                bufix = optstart;
                break;
            }

            *buffer.offset(bufix as isize) = code as u8;
            *buffer.offset(bufix as isize + 1) = incr as u8;
            core::ptr::copy_nonoverlapping(
                (*opt_ptr).value.offset(ix as isize),
                buffer.offset(bufix as isize + 2),
                incr as usize,
            );
            length -= incr;
            ix += incr;
            bufix += 2 + incr;
        }
    }

    bufix
}

// ---- pretty_print_option ----
#[no_mangle]
pub unsafe extern "system" fn pretty_print_option(
    code: u32,
    data: *const u8,
    len: i32,
    emit_commas: i32,
    emit_quotes: i32,
) -> *mut u8 {
    'toobig: {
    static mut optbuf: [u8; 32768] = [0; 32768];
    let mut hunksize: i32 = 0;
    let mut numhunk: i32 = -1;
    let mut numelem: i32 = 0;
    let mut fmtbuf: [u8; 32] = [0u8; 32];
    let mut op = optbuf.as_mut_ptr();
    let mut opleft: isize = 32768;
    let mut dp = data;
    let mut foo: in_addr = core::mem::zeroed();
    let comma: u8;

    if code > 255 {
        error((b"pretty_print_option: bad code %d\0".as_ptr() as *const u8) as *const i8, code);
    }

    if emit_commas != 0 {
        comma = b',';
    } else {
        comma = b' ';
    }

    let format_ptr = dhcp_options[code as usize].format;
    let name_ptr = dhcp_options[code as usize].name;
    let mut i: i32 = 0;
    loop {
        if *format_ptr.add(i as usize) == 0 {
            break;
        }
        if numhunk == 0 {
            warning(
                (b"%s: Excess information in format string: %s\0".as_ptr() as *const u8) as *const i8,
                name_ptr,
                format_ptr.add(i as usize),
            );
            break;
        }
        numelem += 1;
        fmtbuf[i as usize] = (*format_ptr.add(i as usize)) as u8;
        match *format_ptr.add(i as usize) as u8 as char {
            'A' => {
                numelem -= 1;
                fmtbuf[i as usize] = 0;
                numhunk = 0;
            }
            'X' => {
                let mut k: i32 = 0;
                while k < len {
                    if isascii(*(data.add(k as usize)) as i32) == 0
                        || isprint(*(data.add(k as usize)) as i32) == 0
                    {
                        break;
                    }
                    k += 1;
                }
                if k == len {
                    fmtbuf[i as usize] = b't';
                    numhunk = -2;
                } else {
                    fmtbuf[i as usize] = b'x';
                    hunksize += 1;
                    // comma = ':' // comma is not mutated? Actually comma is set to ':' in C. We handle later.
                    numhunk = 0;
                }
                fmtbuf[(i + 1) as usize] = 0;
            }
            't' => {
                fmtbuf[i as usize] = b't';
                fmtbuf[(i + 1) as usize] = 0;
                numhunk = -2;
            }
            'I' | 'l' | 'L' => hunksize += 4,
            's' | 'S' => hunksize += 2,
            'b' | 'B' | 'f' => hunksize += 1,
            'e' => {}
            _ => {
                warning(
                    (b"%s: garbage in format string: %s\0".as_ptr() as *const u8) as *const i8,
                    name_ptr,
                    format_ptr.add(i as usize),
                );
            }
        }
        i += 1;
    }

    if hunksize > len {
        warning(
            (b"%s: expecting at least %d bytes; got %d\0".as_ptr() as *const u8) as *const i8,
            name_ptr,
            hunksize,
            len,
        );
        return b"<error>\0".as_ptr() as *mut u8;
    }

    if numhunk == -1 && hunksize < len {
        warning(
            (b"%s: %d extra bytes\0".as_ptr() as *const u8) as *const i8,
            name_ptr,
            len - hunksize,
        );
    }

    if numhunk == 0 {
        numhunk = len / hunksize;
    }

    if numhunk > 0 && numhunk * hunksize < len {
        warning(
            (b"%s: %d extra bytes at end of array\0".as_ptr() as *const u8) as *const i8,
            name_ptr,
            len - numhunk * hunksize,
        );
    }

    if numhunk < 0 {
        numhunk = 1;
    }

    let mut i: i32 = 0;
    while i < numhunk {
        let mut j: i32 = 0;
        while j < numelem {
            let opcount: i32;
            match fmtbuf[j as usize] as char {
                't' => {
                    if emit_quotes != 0 {
                        *op = b'"';
                        op = op.add(1);
                        opleft -= 1;
                    }
                    while dp < data.add(len as usize) {
                        if isascii(*dp as i32) == 0 || isprint(*dp as i32) == 0 {
                            if dp.add(1) != data.add(len as usize) || *dp != 0 {
                                let n = _snprintf(
                                    op as *mut i8,
                                    opleft as usize,
                                    b"\\%03o\0".as_ptr() as *const i8,
                                    *dp as i32,
                                );
                                op = op.add(n as usize);
                                opleft -= n as isize;
                            }
                        } else if *dp == b'"'
                            || *dp == b'\''
                            || *dp == b'$'
                            || *dp == b'`'
                            || *dp == b'\\'
                        {
                            *op = b'\\';
                            op = op.add(1);
                            *op = *dp;
                            op = op.add(1);
                            opleft -= 2;
                        } else {
                            *op = *dp;
                            op = op.add(1);
                            opleft -= 1;
                        }
                        dp = dp.add(1);
                    }
                    if emit_quotes != 0 {
                        *op = b'"';
                        op = op.add(1);
                        opleft -= 1;
                    }
                    *op = 0;
                }
                'I' => {
                    foo.S_un.S_addr = htonl(getULong(dp as *mut u8));
                    let s = inet_ntoa(foo);
                    let n = strncpy((op) as *mut i8, s, (opleft - 1) as usize);
                    *op.add((opleft - 1) as usize) = 0;
                    let opcount = strlen((op) as *const i8);
                    if opcount as isize >= opleft {
                        break 'toobig;
                    }
                    opleft -= opcount as isize;
                    op = op.add(opcount as usize);
                    dp = dp.add(4);
                }
                'l' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%ld\0".as_ptr() as *const i8,
                        getLong(dp as *mut u8) as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(4);
                }
                'L' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%ld\0".as_ptr() as *const i8,
                        getULong(dp as *mut u8) as u32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(4);
                }
                's' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%d\0".as_ptr() as *const i8,
                        getShort(dp as *mut u8) as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(2);
                }
                'S' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%d\0".as_ptr() as *const i8,
                        getUShort(dp as *mut u8) as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(2);
                }
                'b' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%d\0".as_ptr() as *const i8,
                        *(dp as *const i8) as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(1);
                }
                'B' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%d\0".as_ptr() as *const i8,
                        *dp as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(1);
                }
                'x' => {
                    let n = _snprintf(
                        op as *mut i8,
                        opleft as usize,
                        b"%x\0".as_ptr() as *const i8,
                        *dp as i32,
                    );
                    if n as isize >= opleft || n == -1 {
                        break 'toobig;
                    }
                    opleft -= n as isize;
                    op = op.add(n as usize);
                    dp = dp.add(1);
                }
                'f' => {
                    let src = if *dp != 0 {
                        b"true\0".as_ptr()
                    } else {
                        b"false\0".as_ptr()
                    };
                    let n = strncpy(op as *mut i8, src as *const i8, (opleft - 1) as usize);
                    *op.add((opleft - 1) as usize) = 0;
                    let opcount = n as usize; // mimicking the original cast
                    if opcount as isize >= opleft {
                        break 'toobig;
                    }
                    opleft -= opcount as isize;
                    op = op.add(opcount);
                    dp = dp.add(1);
                }
                _ => {
                    warning(
                        (b"Unexpected format code %c\0".as_ptr() as *const u8) as *const i8,
                        fmtbuf[j as usize] as i32,
                    );
                }
            }
            let slen = strlen((op) as *const i8);
            op = op.add(slen);
            opleft -= slen as isize;
            if opleft < 1 {
                break 'toobig;
            }
            if j + 1 < numelem && comma != b':' {
                *op = b' ';
                op = op.add(1);
                opleft -= 1;
            }
            j += 1;
        }
        if i + 1 < numhunk {
            *op = comma;
            op = op.add(1);
            opleft -= 1;
        }
        if opleft < 1 {
            break 'toobig;
        }
        i += 1;
    }
    return optbuf.as_mut_ptr();
    }
    // C `toobig:` handler (fall-through target of `goto toobig`)
    warning((b"dhcp option too large\0".as_ptr() as *const u8) as *const i8);
    return b"<error>\0".as_ptr() as *mut u8;
}

// ---- parse_options ----
unsafe fn parse_options(packet: *mut _packet) {
    memset((*packet).options.as_mut_ptr() as *mut _, 0, core::mem::size_of_val(&(*packet).options));
    if memcmp((*(*packet).raw).options.as_ptr() as *const _, DHCP_OPTIONS_COOKIE.as_ptr() as *const _, 4) != 0 {
        (*packet).options_valid = 0;
        return;
    }
    parse_option_buffer(
        packet,
        core::ptr::addr_of_mut!((*(*packet).raw).options[4]) as *mut u8,
        ((*packet).packet_length - (DHCP_FIXED_NON_UDP) as i32 - 4) as i32,
    );
    if (*packet).options_valid != 0 && !(*packet).options[DHO_DHCP_OPTION_OVERLOAD as usize].data.is_null() {
        if (*((*packet).options[DHO_DHCP_OPTION_OVERLOAD as usize].data)) & 1 != 0 {
            parse_option_buffer(
                packet,
                core::ptr::addr_of_mut!((*(*packet).raw).file) as *mut u8,
                core::mem::size_of_val(&(*(*packet).raw).file) as i32,
            );
        }
        if (*((*packet).options[DHO_DHCP_OPTION_OVERLOAD as usize].data)) & 2 != 0 {
            parse_option_buffer(
                packet,
                core::ptr::addr_of_mut!((*(*packet).raw).sname) as *mut u8,
                core::mem::size_of_val(&(*(*packet).raw).sname) as i32,
            );
        }
    }
}

// ---- cons_options ----
pub unsafe fn cons_options(
    inpacket: *mut packet,
    outpacket: *mut dhcp_packet,
    mut mms: i32,
    options: *mut *mut tree_cache,
) -> i32 {
    let mut priority_list: [u8; 300] = [0; 300];
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut priority_len: i32;
    let mut main_buffer_size: i32;
    let mut mainbufix: i32;
    let mut option_size: i32;
    let mut length: i32;

    // If client provided max DHCP message size, use it; otherwise default.
    if mms == 0
        && !inpacket.is_null()
        && !(*inpacket).options[DHO_DHCP_MAX_MESSAGE_SIZE as usize].data.is_null()
        && (*inpacket).options[DHO_DHCP_MAX_MESSAGE_SIZE as usize].len >= 2
    {
        mms = (((getUShort((((*inpacket).options[DHO_DHCP_MAX_MESSAGE_SIZE as usize].data as *const core::ffi::c_void) as i32) as *const u8) as i32) as *const u8)) as i32;
    }

    if mms != 0 {
        main_buffer_size = mms - (DHCP_FIXED_LEN) as i32;
    } else {
        main_buffer_size = (576 - DHCP_FIXED_LEN) as i32;
    }

    if main_buffer_size > 4096 {
        main_buffer_size = 4096;
    }

    // Preload mandatory options.
    priority_len = 0;
    priority_list[priority_len as usize] = (DHO_DHCP_MESSAGE_TYPE) as u8;
    priority_len += 1;
    priority_list[priority_len as usize] = (DHO_DHCP_SERVER_IDENTIFIER) as u8;
    priority_len += 1;
    priority_list[priority_len as usize] = (DHO_DHCP_LEASE_TIME) as u8;
    priority_len += 1;
    priority_list[priority_len as usize] = (DHO_DHCP_MESSAGE) as u8;
    priority_len += 1;

    // Prioritize based on client request list or default.
    if !inpacket.is_null()
        && !(*inpacket).options[DHO_DHCP_PARAMETER_REQUEST_LIST as usize].data.is_null()
    {
        let mut prlen: i32 = (*inpacket).options[DHO_DHCP_PARAMETER_REQUEST_LIST as usize].len as i32;
        if (prlen + priority_len) > 300 {
            prlen = 300 - priority_len;
        }
        memcpy(
            (&mut priority_list[priority_len as usize]) as *mut u8 as *mut core::ffi::c_void,
            (*inpacket).options[DHO_DHCP_PARAMETER_REQUEST_LIST as usize].data as *const core::ffi::c_void,
            prlen as usize,
        );
        priority_len += prlen;
    } else {
        memcpy(
            (&mut priority_list[priority_len as usize]) as *mut u8 as *mut core::ffi::c_void,
            dhcp_option_default_priority_list.as_ptr() as *const core::ffi::c_void,
            sizeof_dhcp_option_default_priority_list as usize,
        );
        priority_len += sizeof_dhcp_option_default_priority_list as i32;
    }

    // Store options into buffer.
    option_size = store_options(
        buffer.as_mut_ptr(),
        main_buffer_size - 7,
        options,
        priority_list.as_mut_ptr(),
        priority_len,
        main_buffer_size,
        main_buffer_size,
    );

    // Write cookie.
    memcpy(
        (*outpacket).options.as_mut_ptr() as *mut core::ffi::c_void,
        DHCP_OPTIONS_COOKIE.as_ptr() as *const core::ffi::c_void,
        4,
    );
    mainbufix = 4;

    // Try to fit everything in the packet's option buffer.
    if option_size <= main_buffer_size - mainbufix {
        memcpy(
            &mut (*outpacket).options[mainbufix as usize] as *mut u8 as *mut core::ffi::c_void,
            buffer.as_mut_ptr() as *const core::ffi::c_void,
            option_size as usize,
        );
        mainbufix += option_size;
        if mainbufix < main_buffer_size {
            (*outpacket).options[mainbufix as usize] = (DHO_END) as u8;
            mainbufix += 1;
        }
        length = DHCP_FIXED_NON_UDP as i32 + mainbufix;
    } else {
        // Overload needed.
        (*outpacket).options[mainbufix as usize] = (DHO_DHCP_OPTION_OVERLOAD) as u8;
        mainbufix += 1;
        (*outpacket).options[mainbufix as usize] = 1;
        mainbufix += 1;
        if option_size > (main_buffer_size - mainbufix + (DHCP_FILE_LEN) as i32) {
            (*outpacket).options[mainbufix as usize] = 3;
        } else {
            (*outpacket).options[mainbufix as usize] = 1;
        }
        memcpy(
            &mut (*outpacket).options[mainbufix as usize] as *mut u8 as *mut core::ffi::c_void,
            buffer.as_mut_ptr() as *const core::ffi::c_void,
            (main_buffer_size - mainbufix) as usize,
        );
        length = DHCP_FIXED_NON_UDP as i32 + mainbufix;
    }

    length
}

// ---- do_packet ----
pub unsafe extern "C" fn do_packet(
    interface: *mut interface_info,
    packet: *mut dhcp_packet,
    len: i32,
    from_port: u32,
    from: iaddr,
    hfrom: *mut hardware,
) {
    let mut tp: _packet = core::mem::zeroed();

    if (*packet).hlen > core::mem::size_of_val(&(*packet).chaddr) as u8 {
        note(b"Discarding packet with invalid hlen.\0".as_ptr() as *const i8);
        return;
    }

    tp.raw = packet;
    tp.packet_length = (len as u32) as i32;
    tp.client_port = (from_port) as i32;
    tp.client_addr = from;
    tp.interface = interface;
    tp.haddr = hfrom;

    parse_options(&mut tp as *mut _packet);

    if tp.options_valid != 0 && !tp.options[DHO_DHCP_MESSAGE_TYPE as usize].data.is_null() {
        tp.packet_type = (*tp.options[DHO_DHCP_MESSAGE_TYPE as usize].data) as i32;
    }

    if tp.packet_type != 0 {
        dhcp(&mut tp as *mut _packet);
    } else {
        bootp(&mut tp as *mut _packet);
    }

    for i in 0..256 {
        if tp.options[i].len != 0 && !tp.options[i].data.is_null() {
            free(tp.options[i].data as *mut core::ffi::c_void);
        }
    }
}
