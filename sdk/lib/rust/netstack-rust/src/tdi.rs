//! TDI (Transport Driver Interface) constants + helpers — the surface AFD
//! (winsock's kernel backer) drives. netstack.sys reimplements the transport
//! side of this contract; see tdidev.rs for the device/dispatch layer.
//!
//! Minor-function codes ride in the IO_STACK_LOCATION MinorFunction field of
//! IRP_MJ_INTERNAL_DEVICE_CONTROL requests (set by the TdiBuildXxx macros).
//! Definitions mirror sdk/include/ddk/tdikrnl.h.

// TDI minor function codes.
pub const TDI_ASSOCIATE_ADDRESS: u8 = 0x01;
pub const TDI_DISASSOCIATE_ADDRESS: u8 = 0x02;
pub const TDI_CONNECT: u8 = 0x03;
pub const TDI_LISTEN: u8 = 0x04;
pub const TDI_ACCEPT: u8 = 0x05;
pub const TDI_DISCONNECT: u8 = 0x06;
pub const TDI_SEND: u8 = 0x07;
pub const TDI_RECEIVE: u8 = 0x08;
pub const TDI_SEND_DATAGRAM: u8 = 0x09;
pub const TDI_RECEIVE_DATAGRAM: u8 = 0x0A;
pub const TDI_SET_EVENT_HANDLER: u8 = 0x0B;
pub const TDI_QUERY_INFORMATION: u8 = 0x0C;
pub const TDI_SET_INFORMATION: u8 = 0x0D;
pub const TDI_ACTION: u8 = 0x0E;

// TDI event types (TDI_SET_EVENT_HANDLER), the upcalls into AFD.
pub const TDI_EVENT_CONNECT: i32 = 0;
pub const TDI_EVENT_DISCONNECT: i32 = 1;
pub const TDI_EVENT_ERROR: i32 = 2;
pub const TDI_EVENT_RECEIVE: i32 = 3;
pub const TDI_EVENT_RECEIVE_DATAGRAM: i32 = 4;
pub const TDI_EVENT_RECEIVE_EXPEDITED: i32 = 5;
pub const TDI_EVENT_SEND_POSSIBLE: i32 = 6;

// EA names used on \Device\Tcp opens to select the file-object flavor.
pub const EA_TRANSPORT_ADDRESS: &[u8] = b"TransportAddress";
pub const EA_CONNECTION_CONTEXT: &[u8] = b"ConnectionContext";

// ReceiveFlags passed to a TDI_EVENT_RECEIVE indication (ndis/tdi.h).
pub const TDI_RECEIVE_NORMAL: u32 = 0x0020;
pub const TDI_RECEIVE_ENTIRE_MESSAGE: u32 = 0x0100;

/// PTDI_IND_RECEIVE (sdk/include/ddk/tdikrnl.h) — the upcall the transport makes
/// into AFD when stream data arrives on a connection. NTAPI = `extern "system"`
/// (cdecl amd64/arm64, stdcall i386). Returns STATUS_SUCCESS (took the data) or
/// STATUS_DATA_NOT_ACCEPTED (couldn't take it now — transport keeps it).
pub type PtdiIndReceive = unsafe extern "system" fn(
    TdiEventContext: *mut core::ffi::c_void,
    ConnectionContext: *mut core::ffi::c_void,
    ReceiveFlags: u32,
    BytesIndicated: u32,
    BytesAvailable: u32,
    BytesTaken: *mut u32,
    Tsdu: *mut core::ffi::c_void,
    IoRequestPacket: *mut ntkrnl_sys::ddk::PIRP,
) -> i32;

/// TDI address family for IPv4 (TDI_ADDRESS_TYPE_IP).
pub const TDI_ADDRESS_TYPE_IP: u16 = 2;

/// The three TDI file-object flavors AFD opens on a transport device.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TdiFileKind {
    /// No EA — a control channel (query-only).
    ControlChannel,
    /// "TransportAddress" EA — a bound local address (TA_IP_ADDRESS).
    Address,
    /// "ConnectionContext" EA — a connection endpoint.
    Connection,
}

// ---- TDI request structures (overlaid on IO_STACK_LOCATION.Parameters by the
// TdiBuildXxx macros) + address wire structs. repr(C) matches tdi.h/tdikrnl.h;
// the pointer padding after the leading ULONG matches the C LP64/ILP32 layout.

use core::ffi::c_void;

#[repr(C)]
pub struct TDI_CONNECTION_INFORMATION {
    pub UserDataLength: i32,
    pub UserData: *mut c_void,
    pub OptionsLength: i32,
    pub Options: *mut c_void,
    pub RemoteAddressLength: i32,
    pub RemoteAddress: *mut c_void,
}

/// TDI_REQUEST_KERNEL — CONNECT / DISCONNECT / LISTEN parameter overlay.
#[repr(C)]
pub struct TDI_REQUEST_KERNEL {
    pub RequestFlags: u32,
    pub RequestConnectionInformation: *mut TDI_CONNECTION_INFORMATION,
    pub ReturnConnectionInformation: *mut TDI_CONNECTION_INFORMATION,
    pub RequestSpecific: *mut c_void,
}

#[repr(C)]
pub struct TDI_REQUEST_KERNEL_SET_EVENT {
    pub EventType: u32,
    pub EventHandler: *mut c_void,
    pub EventContext: *mut c_void,
}

#[repr(C)]
pub struct TDI_REQUEST_KERNEL_SEND {
    pub SendLength: u32,
    pub SendFlags: u32,
}

#[repr(C)]
pub struct TDI_REQUEST_KERNEL_RECEIVE {
    pub ReceiveLength: u32,
    pub ReceiveFlags: u32,
}

#[repr(C)]
pub struct TDI_REQUEST_KERNEL_ASSOCIATE {
    pub AddressHandle: *mut c_void, // HANDLE
}

#[repr(C)]
pub struct TDI_ADDRESS_IP {
    pub sin_port: u16, // network byte order
    pub in_addr: u32,  // network byte order (octets a.b.c.d in memory)
    pub sin_zero: [u8; 8],
}

#[repr(C)]
pub struct TA_ADDRESS {
    pub AddressLength: u16,
    pub AddressType: u16,
    // followed by `AddressLength` bytes of address (TDI_ADDRESS_IP for IP)
}

#[repr(C)]
pub struct TRANSPORT_ADDRESS {
    pub TAAddressCount: i32,
    // followed by TA_ADDRESS[TAAddressCount]
}

/// Parse the first IPv4 endpoint out of a TDI_CONNECTION_INFORMATION's
/// RemoteAddress (a TRANSPORT_ADDRESS / TA_IP_ADDRESS). Returns (octets, port).
pub unsafe fn parse_remote_ip(
    conn_info: *const TDI_CONNECTION_INFORMATION,
) -> Option<([u8; 4], u16)> {
    if conn_info.is_null() {
        return None;
    }
    let ta = (*conn_info).RemoteAddress as *const u8;
    if ta.is_null() {
        return None;
    }
    let count = *(ta as *const i32);
    if count < 1 {
        return None;
    }
    let addr = ta.add(core::mem::size_of::<i32>()) as *const TA_ADDRESS;
    if (*addr).AddressType != TDI_ADDRESS_TYPE_IP {
        return None;
    }
    let ip = (addr as *const u8).add(core::mem::size_of::<TA_ADDRESS>()) as *const TDI_ADDRESS_IP;
    let port = u16::from_be((*ip).sin_port);
    // in_addr's 4 memory bytes are the network-order octets a.b.c.d.
    let raw = &(*ip).in_addr as *const u32 as *const u8;
    let octets = [*raw, *raw.add(1), *raw.add(2), *raw.add(3)];
    Some((octets, port))
}

/// Human-readable name for a TDI minor function (logging).
pub fn minor_name(m: u8) -> &'static str {
    match m {
        TDI_ASSOCIATE_ADDRESS => "ASSOCIATE_ADDRESS",
        TDI_DISASSOCIATE_ADDRESS => "DISASSOCIATE_ADDRESS",
        TDI_CONNECT => "CONNECT",
        TDI_LISTEN => "LISTEN",
        TDI_ACCEPT => "ACCEPT",
        TDI_DISCONNECT => "DISCONNECT",
        TDI_SEND => "SEND",
        TDI_RECEIVE => "RECEIVE",
        TDI_SEND_DATAGRAM => "SEND_DATAGRAM",
        TDI_RECEIVE_DATAGRAM => "RECEIVE_DATAGRAM",
        TDI_SET_EVENT_HANDLER => "SET_EVENT_HANDLER",
        TDI_QUERY_INFORMATION => "QUERY_INFORMATION",
        TDI_SET_INFORMATION => "SET_INFORMATION",
        TDI_ACTION => "ACTION",
        _ => "UNKNOWN",
    }
}
