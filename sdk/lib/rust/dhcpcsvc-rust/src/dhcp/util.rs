use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u8 = 0;

// ---- piaddr ----
pub unsafe fn piaddr(addr: iaddr) -> *mut i8 {
    let mut sa: sockaddr_in = core::mem::zeroed();
    core::ptr::copy_nonoverlapping(
        addr.iabuf.as_ptr(),
        (&mut sa.sin_addr as *mut _ as *mut i8) as *mut u8,
        core::mem::size_of_val(&sa.sin_addr),
    );
    inet_ntoa(sa.sin_addr)
}

// ---- note / warn / warning / debug / error ----
// These are C-variadic log helpers. Stable Rust CANNOT `va_start` a va_list, so the variadic ENTRY
// points (`note`, `warn`, …) live in `rust_varargs_shim.c` (the sanctioned varargs-rust trampoline —
// AGENTS.md "c_variadic"; prelude declares the entries so callers resolve). Each entry does the
// `va_start` C cannot express in Rust, then tail-calls its fixed-arity `*_v` twin below, which holds
// the ACTUAL translated logic (the `_vsnprintf` formatting into the local buffer + the DPRINT). `ap`
// is the caller's `va_list` (a `char*` on all three windows-gnu targets), forwarded to `_vsnprintf`.

// ---- note ----
#[no_mangle]
pub unsafe extern "C" fn note_v(format: *const c_char, ap: *mut c_void) -> i32 {
    let mut buf: [i8; 0x100] = [0i8; 0x100];
    let ret = _vsnprintf(buf.as_mut_ptr(), 0x100, format, ap as *mut c_char);
    // DPRINT!("NOTE: %s\n", buf);
    ret
}

// ---- debug ----
#[no_mangle]
pub unsafe extern "C" fn debug_v(format: *const c_char, ap: *mut c_void) -> i32 {
    let mut buf: [i8; 0x100] = [0i8; 0x100];
    let ret: i32;

    ret = _vsnprintf(buf.as_mut_ptr(), 0x100, format, ap as *mut c_char);

    DPRINT!(b"DEBUG: %s\n\0".as_ptr() as *const i8, buf.as_ptr());
    ret
}

// ---- warn ----
#[no_mangle]
pub unsafe extern "C" fn warn_v(format: *const c_char, ap: *mut c_void) -> i32 {
    let mut buf: [i8; 0x100] = [0; 0x100];
    let ret = _vsnprintf(buf.as_mut_ptr(), core::mem::size_of_val(&buf), format, ap as *mut c_char);
    ret
}

// ---- warning ----
#[no_mangle]
pub unsafe extern "C" fn warning_v(format: *const c_char, ap: *mut c_void) -> i32 {
    let mut buf: [i8; 0x100] = core::mem::zeroed();
    let ret = _vsnprintf(buf.as_mut_ptr(), 0x100usize, format, ap as *mut c_char);
    DPRINT!(b"WARNING: %s\n\0".as_ptr() as *const i8, buf.as_ptr());
    ret
}

// ---- error ----
#[no_mangle]
pub unsafe extern "C" fn error_v(format: *const c_char, ap: *mut c_void) {
    let mut buf: [u8; 0x100] = [0; 0x100];
    _vsnprintf(
        buf.as_mut_ptr() as *mut i8,
        0x100i32 as usize,
        format as *const i8,
        ap as *mut c_char,
    );
    DPRINT1!("ERROR: %s\n\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
}

// ---- getShort ----
pub unsafe fn getShort(data: *mut u8) -> i16 {
    let val = core::ptr::read_unaligned(data as *const u16);
    ntohs(val) as i16
}

// ---- getUShort ----
pub unsafe fn getUShort(data: *const u8) -> u16 {
    let raw = core::ptr::read_unaligned(data as *const u16);
    ntohs(raw)
}

// ---- getLong ----
pub unsafe fn getLong(data: *mut u8) -> i32 {
    ntohl(*(data as *const u32)) as i32
}

// ---- getULong ----
pub unsafe fn getULong(data: *mut u8) -> u32 {
    ntohl(*(data as *const u32))
}

// ---- addr_eq ----
pub unsafe fn addr_eq(a: iaddr, b: iaddr) -> i32 {
    if a.len != b.len {
        return 0;
    }
    let len = a.len as usize;
    let a_ptr = &a.iabuf as *const u8;
    let b_ptr = &b.iabuf as *const u8;
    let a_slice = core::slice::from_raw_parts(a_ptr, len);
    let b_slice = core::slice::from_raw_parts(b_ptr, len);
    (a_slice == b_slice) as i32
}

// ---- dmalloc ----
pub unsafe fn dmalloc(size: i32, name: *mut i8) -> *mut core::ffi::c_void {
    malloc(size as usize)
}

// ---- broadcast_addr ----
unsafe fn broadcast_addr(addr: iaddr, mask: iaddr) -> iaddr {
    let bcast: iaddr = core::mem::zeroed();
    bcast
}

// ---- subnet_number ----
unsafe fn subnet_number(addr: iaddr, mask: iaddr) -> iaddr {
    let bcast: iaddr = core::mem::zeroed();
    return bcast;
}

// ---- read_client_conf ----
pub unsafe fn read_client_conf(ifi: *mut interface_info) -> i32 {
    let mut config: *mut client_config;
    let mut ComputerName: [i8; MAX_COMPUTERNAME_LENGTH as usize + 1] = [0i8; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut ComputerNameSize: u32 = (MAX_COMPUTERNAME_LENGTH + 1) as u32;
    let lpClassIdentifier: LPSTR = b"MSFT 5.0\0".as_ptr() as *mut i8;

    if !ifi.is_null() && !(*(*ifi).client).config.is_null() {
        config = (*(*ifi).client).config;
    } else {
        warn(b"util.c read_client_conf poorly implemented!\0".as_ptr() as *mut i8);
        return 0;
    }

    GetComputerName(ComputerName.as_mut_ptr(), &mut ComputerNameSize as *mut u32);
    debug(
        b"Hostname: %s, length: %lu\0".as_ptr() as *mut i8,
        ComputerName.as_mut_ptr() as *mut i8,
        ComputerNameSize,
    );

    let lpCompName: *mut i8 = HeapAlloc(GetProcessHeap(), 0, ((ComputerNameSize as usize + 1) as SIZE_T) as usize) as *mut i8;
    if !lpCompName.is_null() {
        memcpy(
            lpCompName as *mut c_void,
            ComputerName.as_ptr() as *const c_void,
            (ComputerNameSize as usize + 1) as usize,
        );
        (*config).send_options[DHO_HOST_NAME as usize].data = lpCompName as *mut u8;
        (*config).send_options[DHO_HOST_NAME as usize].len = (ComputerNameSize as u8) as i32;
        debug(
            b"Hostname: %s, length: %d\0".as_ptr() as *mut i8,
            (*config).send_options[DHO_HOST_NAME as usize].data as *mut i8,
            (*config).send_options[DHO_HOST_NAME as usize].len,
        );
    } else {
        error(
            (b"Failed to allocate heap for hostname\0".as_ptr() as *mut u8) as *const i8,
        );
    }

    (*config).send_options[DHO_DHCP_CLIENT_IDENTIFIER as usize].data = (*ifi).hw_address.haddr.as_mut_ptr();
    (*config).send_options[DHO_DHCP_CLIENT_IDENTIFIER as usize].len = (*ifi).hw_address.hlen as i32;

    (*config).send_options[DHO_DHCP_CLASS_IDENTIFIER as usize].data = lpClassIdentifier as *mut u8;
    (*config).send_options[DHO_DHCP_CLASS_IDENTIFIER as usize].len = (strlen(lpClassIdentifier) as u8) as i32;

    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_SUBNET_MASK) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_DOMAIN_NAME) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_ROUTERS) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_DOMAIN_NAME_SERVERS) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_NETBIOS_NAME_SERVERS) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_NETBIOS_NODE_TYPE) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_NETBIOS_SCOPE) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_ROUTER_DISCOVERY) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_STATIC_ROUTES) as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = 249i32 as u8;
    (*config).requested_option_count += 1;
    let idx = (*config).requested_option_count as usize;
    (*config).requested_options[idx] = (DHO_VENDOR_ENCAPSULATED_OPTIONS) as u8;
    (*config).requested_option_count += 1;

    warn(b"util.c read_client_conf poorly implemented!\0".as_ptr() as *mut i8);
    return 0;
}
