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
