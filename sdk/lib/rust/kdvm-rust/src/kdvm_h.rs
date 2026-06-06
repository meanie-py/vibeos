//! Hand-ported header `drivers/base/kdvm/kdvm.h`: the kdvm marshalling
//! structs, KDP_STATUS enum, protocol constants and the `PFNDBGPRNT`
//! typedef. These live only in the header (no `.c`), so rustconv never
//! translated them. Ported faithfully here (`#pragma pack(push,1)` ranges
//! map to `#[repr(C, packed)]`).

use ntkrnl_sys::base::{CHAR, UCHAR, USHORT, ULONG, BOOLEAN};

// typedef ULONG (*PFNDBGPRNT)(const char *Format, ...);
pub type PFNDBGPRNT = Option<unsafe extern "C" fn(Format: *const CHAR, ...) -> ULONG>;

// #define KDRPC_PROTOCOL_VERSION 0x101
pub const KDRPC_PROTOCOL_VERSION: ULONG = 0x101;
// #define CONNECTION_TEST_ROUNDS 2
pub const CONNECTION_TEST_ROUNDS: ULONG = 2;
// #define KDVM_BUFFER_SIZE (131072 + 1024)
pub const KDVM_BUFFER_SIZE: ULONG = 131072 + 1024;
// #define KDRPC_TEST_BUFFER_SIZE 512
pub const KDRPC_TEST_BUFFER_SIZE: ULONG = 512;

// typedef enum { KDP_PACKET_RECEIVED = 0, KDP_PACKET_TIMEOUT = 1,
//                KDP_PACKET_RESEND = 2 } KDP_STATUS;
pub type KDP_STATUS = u32;
pub const KDP_PACKET_RECEIVED: KDP_STATUS = 0;
pub const KDP_PACKET_TIMEOUT: KDP_STATUS = 1;
pub const KDP_PACKET_RESEND: KDP_STATUS = 2;

#[repr(C)]
pub struct KDVM_MARSHAL_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
}

// #pragma pack(push,1)

#[repr(C, packed)]
pub struct KDVM_CMD_HEADER {
    pub Magic: [CHAR; 8],
    pub Command: UCHAR,
}

#[repr(C, packed)]
pub struct KDVM_RECEIVE_HEADER {
    pub Id: USHORT,
    pub Magic: [CHAR; 9],
}
pub type PKDVM_RECEIVE_HEADER = *mut KDVM_RECEIVE_HEADER;

#[repr(C, packed)]
pub struct KDVM_CONTEXT {
    pub RetryCount: ULONG,
    pub BreakInRequested: BOOLEAN,
    pub align: UCHAR,
}
pub type PKDVM_CONTEXT = *mut KDVM_CONTEXT;

// The unnamed bitfield struct in KDVM_SENDPACKET_INFO:
//   UCHAR KdDebuggerNotPresent : 1;
//   UCHAR RetryKdSendPacket : 1;
//   UCHAR KdDebuggerEnabledAvailable : 1;
// Modeled here as a single byte plus accessors so layout matches C exactly.
#[repr(C, packed)]
pub struct KDVM_SENDPACKET_INFO {
    pub Bits: UCHAR,
    pub KdDebuggerEnabled: BOOLEAN,
    pub Unused: USHORT,
}

impl KDVM_SENDPACKET_INFO {
    #[inline]
    pub fn KdDebuggerNotPresent(&self) -> UCHAR {
        self.Bits & 0x1
    }
    #[inline]
    pub fn set_KdDebuggerNotPresent(&mut self, val: UCHAR) {
        self.Bits = (self.Bits & !0x1) | (val & 0x1);
    }
    #[inline]
    pub fn RetryKdSendPacket(&self) -> UCHAR {
        (self.Bits >> 1) & 0x1
    }
    #[inline]
    pub fn set_RetryKdSendPacket(&mut self, val: UCHAR) {
        self.Bits = (self.Bits & !0x2) | ((val & 0x1) << 1);
    }
    #[inline]
    pub fn KdDebuggerEnabledAvailable(&self) -> UCHAR {
        (self.Bits >> 2) & 0x1
    }
    #[inline]
    pub fn set_KdDebuggerEnabledAvailable(&mut self, val: UCHAR) {
        self.Bits = (self.Bits & !0x4) | ((val & 0x1) << 2);
    }
}

#[repr(C, packed)]
pub struct KDVM_SEND_PKT_REQUEST {
    pub MessageHeader: KDVM_MARSHAL_STRING,
    pub MessageData: KDVM_MARSHAL_STRING,
    pub KdContext: KDVM_CONTEXT,
    pub PacketType: ULONG,
    pub HeaderSize: ULONG,
    pub DataSize: ULONG,
    pub Info: KDVM_SENDPACKET_INFO,
}
pub type PKDVM_SEND_PKT_REQUEST = *mut KDVM_SEND_PKT_REQUEST;

#[repr(C, packed)]
pub struct KDVM_SEND_PKT_RESULT {
    pub CommandType: UCHAR,
    pub KdContext: KDVM_CONTEXT,
    pub Info: KDVM_SENDPACKET_INFO,
}
pub type PKDVM_SEND_PKT_RESULT = *mut KDVM_SEND_PKT_RESULT;

#[repr(C, packed)]
pub struct KDVM_RECV_PKT_REQUEST {
    pub PacketType: ULONG,
    pub Info: KDVM_SENDPACKET_INFO,
    pub MessageHeader: KDVM_MARSHAL_STRING,
    pub MessageData: KDVM_MARSHAL_STRING,
    pub KdContext: KDVM_CONTEXT,
}

#[repr(C, packed)]
pub struct KDVM_RECV_PKT_RESULT {
    pub CommandType: UCHAR,
    pub MessageHeader: KDVM_MARSHAL_STRING,
    pub MessageData: KDVM_MARSHAL_STRING,
    pub KdContext: KDVM_CONTEXT,
    pub KdStatus: KDP_STATUS,
    pub FullSize: ULONG,
    pub HeaderSize: ULONG,
    pub DataSize: ULONG,
    pub Info: KDVM_SENDPACKET_INFO,
}
pub type PKDVM_RECV_PKT_RESULT = *mut KDVM_RECV_PKT_RESULT;
