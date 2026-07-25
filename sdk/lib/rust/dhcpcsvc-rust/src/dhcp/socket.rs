use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut ServerSocket: SOCKET = unsafe { core::mem::zeroed() };

// ---- SocketInit ----
unsafe fn SocketInit() {
    ServerSocket = socket(AF_INET, SOCK_DGRAM, 0);
}

// ---- send_packet ----
#[no_mangle]
pub unsafe extern "system" fn send_packet(
    ip: *mut interface_info,
    p: *mut dhcp_packet,
    size: usize,
    addr: in_addr,
    broadcast: *mut sockaddr_in,
    hardware: *mut hardware,
) -> isize {
    if size > i32::MAX as usize {
        return WSAEMSGSIZE as isize;
    }

    let result = sendto(
        ((*ip).wfdesc) as usize,
        p as *const i8,
        size as i32,
        0i32,
        broadcast as *const sockaddr,
        core::mem::size_of::<sockaddr_in>() as i32,
    );

    if result < 0 {
        note(b"send_packet: %x\0".as_ptr() as *const i8, result);
        if result == (WSAENETUNREACH) as i32 {
            note(
                b"send_packet: please consult README file%s\0".as_ptr() as *const i8,
                b" regarding broadcast address.\0".as_ptr() as *const i8,
            );
        }
    }

    result as isize
}

// ---- receive_packet ----
pub unsafe extern "C" fn receive_packet(
    ip: *mut interface_info,
    packet_data: *mut u8,
    packet_len: usize,
    dest: *mut sockaddr_in,
    hardware: *mut hardware,
) -> i32 {
    let mut recv_addr_size: i32 = core::mem::size_of::<sockaddr_in>() as i32;
    let result: i32;

    if packet_len > (i32::MAX as usize) {
        return WSAEMSGSIZE;
    }

    result = recvfrom(
        ((*ip).rfdesc) as usize,
        packet_data as *mut i8,
        packet_len as i32,
        0,
        dest as *mut sockaddr,
        &mut recv_addr_size,
    );
    result
}
