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

/// PTDI_IND_RECEIVE_DATAGRAM (sdk/include/ddk/tdikrnl.h) — the upcall the transport
/// makes into AFD when a connectionless datagram (UDP) arrives on an ADDRESS object.
/// Unlike the stream receive indication this carries the SOURCE address (a
/// TRANSPORT_ADDRESS / TA_IP_ADDRESS) so AFD/recvfrom can report the sender.
pub type PtdiIndReceiveDatagram = unsafe extern "system" fn(
    TdiEventContext: *mut core::ffi::c_void,
    SourceAddressLength: i32,
    SourceAddress: *mut core::ffi::c_void,
    OptionsLength: i32,
    Options: *mut core::ffi::c_void,
    ReceiveDatagramFlags: u32,
    BytesIndicated: u32,
    BytesAvailable: u32,
    BytesTaken: *mut u32,
    Tsdu: *mut core::ffi::c_void,
    IoRequestPacket: *mut ntkrnl_sys::ddk::PIRP,
) -> i32;

/// TDI address family for IPv4 (TDI_ADDRESS_TYPE_IP).
pub const TDI_ADDRESS_TYPE_IP: u16 = 2;

/// TDI_ADDRESS_LENGTH_IP — sizeof(TDI_ADDRESS_IP) = sin_port(2) + in_addr(4) +
/// sin_zero(8), per tdi.h (the alignment padding is NOT counted in this field).
pub const TDI_ADDRESS_LENGTH_IP: u16 = 14;

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

/// TDI_REQUEST_KERNEL_SENDDG — TDI_SEND_DATAGRAM parameter overlay. The
/// destination endpoint is carried in SendDatagramInformation->RemoteAddress
/// (a TRANSPORT_ADDRESS / TA_IP_ADDRESS), exactly like CONNECT's remote.
#[repr(C)]
pub struct TDI_REQUEST_KERNEL_SENDDG {
    pub SendLength: u32,
    pub SendDatagramInformation: *mut TDI_CONNECTION_INFORMATION,
}

/// TDI_REQUEST_KERNEL_RECEIVEDG — TDI_RECEIVE_DATAGRAM parameter overlay.
#[repr(C)]
pub struct TDI_REQUEST_KERNEL_RECEIVEDG {
    pub ReceiveLength: u32,
    pub ReceiveDatagramInformation: *mut TDI_CONNECTION_INFORMATION,
    pub ReturnDatagramInformation: *mut TDI_CONNECTION_INFORMATION,
    pub ReceiveFlags: u32,
}

// The C TDI_ADDRESS_IP lives under `#include <pshpack1.h>` (sdk/include/psdk/tdi.h)
// — it is BYTE-PACKED, so in_addr sits at offset 2 (right after sin_port), NOT 4,
// and sizeof == TDI_ADDRESS_LENGTH_IP (14). Real AFD/msafd hand us this packed
// layout; a non-packed `#[repr(C)]` here would read in_addr 2 bytes late and
// mangle the destination (e.g. 10.0.2.100 -> 2.100.0.0), so the connect/datagram
// would target the wrong host.
#[repr(C, packed)]
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

/// A complete TA_IP_ADDRESS on the wire: TRANSPORT_ADDRESS (count) + one
/// TA_ADDRESS (len/type) + the BYTE-PACKED TDI_ADDRESS_IP. We hand this to AFD as
/// the SourceAddress of a datagram indication and use it to build CONNECT
/// addresses; AFD's TDI structs are packed, so in_addr lands at offset 10
/// (count@0, addr_len@4, addr_type@6, sin_port@8, in_addr@10, sin_zero@14),
/// giving sizeof == 22. Must stay packed and in lock-step with parse_ta below.
#[repr(C, packed)]
pub struct TaIpFull {
    pub count: i32,
    pub addr_len: u16,
    pub addr_type: u16,
    pub addr: TDI_ADDRESS_IP,
}

impl TaIpFull {
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        TaIpFull {
            count: 1,
            addr_len: TDI_ADDRESS_LENGTH_IP,
            addr_type: TDI_ADDRESS_TYPE_IP,
            addr: TDI_ADDRESS_IP {
                sin_port: port.to_be(),
                in_addr: u32::from_ne_bytes(ip), // memory bytes == network octets
                sin_zero: [0; 8],
            },
        }
    }
}

/// Parse the first IPv4 endpoint out of a raw TRANSPORT_ADDRESS / TA_IP_ADDRESS
/// (count + TA_ADDRESS + TDI_ADDRESS_IP). Returns (octets, port).
pub unsafe fn parse_ta(ta: *const u8) -> Option<([u8; 4], u16)> {
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
    // The TDI_ADDRESS_IP that follows the TA_ADDRESS header is BYTE-PACKED:
    // sin_port @0, in_addr @2 (NOT @4 — no alignment padding). Read it by byte
    // offset so the layout can't drift, and so we never take a reference to a
    // misaligned packed field.
    let ip = (addr as *const u8).add(core::mem::size_of::<TA_ADDRESS>());
    let port = u16::from_be(core::ptr::read_unaligned(ip as *const u16));
    // in_addr's 4 bytes (at packed offset 2) are the network-order octets a.b.c.d.
    let octets = [*ip.add(2), *ip.add(3), *ip.add(4), *ip.add(5)];
    Some((octets, port))
}

/// Parse the first IPv4 endpoint out of a TDI_CONNECTION_INFORMATION's
/// RemoteAddress (used by CONNECT and SEND_DATAGRAM). Returns (octets, port).
pub unsafe fn parse_remote_ip(
    conn_info: *const TDI_CONNECTION_INFORMATION,
) -> Option<([u8; 4], u16)> {
    if conn_info.is_null() {
        return None;
    }
    parse_ta((*conn_info).RemoteAddress as *const u8)
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
