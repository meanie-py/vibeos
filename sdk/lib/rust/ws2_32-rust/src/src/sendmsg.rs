//! `WSASendMsg` (Vista+): sendto with a WSAMSG.
//!
//! Follows Wine: the name/buffers/flags map straight onto `WSASendTo`. Ancillary (control) data
//! cannot be delivered by this transport; control messages that only select addressing details
//! (IP_PKTINFO and friends) are ignored as Wine does, but a UDP_SEND_MSG_SIZE message — which
//! changes how the payload is segmented on the wire — is refused with WSAEOPNOTSUPP rather than
//! silently sending the wrong datagrams.

use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;

const E_FAULT: u32 = 10014;
const E_OPNOTSUPP: u32 = 10045;
const IPPROTO_UDP_LEVEL: i32 = 17;
const UDP_SEND_MSG_SIZE: i32 = 2;

// WSAMSG layout measured (llvm-mingw): x86_64 size 56 {namelen 8, lpBuffers 16, dwBufferCount 24,
// Control 32, dwFlags 48}; i686 size 28 {4, 8, 12, 16, 24}.
const _: () = assert!(core::mem::offset_of!(WSAMSG, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<WSAMSG>() == 56);
    assert!(core::mem::offset_of!(WSAMSG, namelen) == 8);
    assert!(core::mem::offset_of!(WSAMSG, lpBuffers) == 16);
    assert!(core::mem::offset_of!(WSAMSG, dwBufferCount) == 24);
    assert!(core::mem::offset_of!(WSAMSG, Control) == 32);
    assert!(core::mem::offset_of!(WSAMSG, dwFlags) == 48);
};
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<WSAMSG>() == 28);
    assert!(core::mem::offset_of!(WSAMSG, namelen) == 4);
    assert!(core::mem::offset_of!(WSAMSG, lpBuffers) == 8);
    assert!(core::mem::offset_of!(WSAMSG, dwBufferCount) == 12);
    assert!(core::mem::offset_of!(WSAMSG, Control) == 16);
    assert!(core::mem::offset_of!(WSAMSG, dwFlags) == 24);
};

/// True if the control buffer holds a message this transport cannot honour.
unsafe fn control_unsupported(ctl: &WSABUF) -> bool {
    let hdr = core::mem::size_of::<WSACMSGHDR>();
    let align = core::mem::size_of::<usize>(); // MAX_NATURAL_ALIGNMENT
    let total = ctl.len as usize;
    let mut off = 0usize;
    while !ctl.buf.is_null() && off + hdr <= total {
        let h = (ctl.buf as *const u8).add(off) as *const WSACMSGHDR;
        let len = (*h).cmsg_len as usize;
        if len < hdr {
            break;
        }
        if (*h).cmsg_level == IPPROTO_UDP_LEVEL && (*h).cmsg_type == UDP_SEND_MSG_SIZE {
            return true;
        }
        off += (len + align - 1) & !(align - 1);
    }
    false
}

/// C: `int WSASendMsg(SOCKET Handle, LPWSAMSG lpMsg, DWORD dwFlags, LPDWORD lpNumberOfBytesSent,
/// LPWSAOVERLAPPED lpOverlapped, LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)`.
#[no_mangle]
pub unsafe extern "system" fn WSASendMsg(
    Handle: SOCKET,
    lpMsg: *mut WSAMSG,
    dwFlags: DWORD,
    lpNumberOfBytesSent: LPDWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    if lpMsg.is_null() {
        SetLastError(E_FAULT);
        return SOCKET_ERROR;
    }
    let msg = &*lpMsg;
    if msg.Control.len != 0 && control_unsupported(&msg.Control) {
        SetLastError(E_OPNOTSUPP);
        return SOCKET_ERROR;
    }
    crate::src::send::WSASendTo(
        Handle,
        msg.lpBuffers,
        msg.dwBufferCount,
        lpNumberOfBytesSent,
        dwFlags,
        msg.name as *const sockaddr,
        msg.namelen,
        lpOverlapped,
        lpCompletionRoutine,
    )
}
