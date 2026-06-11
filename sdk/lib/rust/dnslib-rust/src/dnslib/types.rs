// Shared type definitions for dnslib-rust, matching sdk/lib/dnslib/inc/dnslib.h
// and sdk/include/psdk/windns.h / winsock.h / ws2atm.h layouts.
// Types already in ntdll-sys (IN_ADDR, IN6_ADDR, PIN_ADDR, PIN6_ADDR) are NOT duplicated.
#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;
use ntdll_sys::nt::*;

// ---- From winsock headers: types not in ntdll-sys ----

pub type IP4_ADDRESS = u32; // typedef DWORD IP4_ADDRESS;
pub type IP6_ADDRESS = [u32; 4]; // DWORD IP6Dword[4]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SOCKADDR_IN {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: IN_ADDR,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SOCKADDR_IN6 {
    pub sin6_family: u16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: IN6_ADDR,
    pub sin6_scope_id: u32,
}

// ATM_ADDR_SIZE = 20
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ATM_ADDRESS {
    pub AddressType: u32,
    pub NumofDigits: u32,
    pub Addr: [u8; 20],
}

// ---- From windns.h: DNS_ADDR_ARRAY (public Windows type) ----
#[repr(C)]
pub struct DnsAddrArray {
    pub MaxCount: u32,
    pub AddrCount: u32,
    pub Tag: u32,
    pub Family: u16,
    pub WordReserved: u16,
    pub Flags: u32,
    pub MatchFlag: u32,
    pub Reserved1: u32,
    pub Reserved2: u32,
}

// ---- From dnslib.h ----

#[repr(C)]
pub struct DNS_FAMILY_INFO {
    pub AddrType: u16,
    pub DnsType: u16,
    pub AddressSize: u32,
    pub SockaddrSize: u32,
    pub AddressOffset: u32,
}

// DNS_ADDRESS: union of {AddressFamily+Port+AtmAddress}, SOCKADDR_IN, SOCKADDR_IN6.
// Union starts at offset 0; max size = 32 bytes (inner struct with ATM_ADDRESS).
// AddressFamily overlaps sin_family; Port overlaps sin_port.
// We embed Ip4Address (SOCKADDR_IN) at offset 0, pad to union max, then post-union fields.
// Access Ip6Address / AtmAddress via pointer casts.
#[repr(C)]
pub struct DNS_ADDRESS {
    pub Ip4Address: SOCKADDR_IN,  // 16 bytes at offset 0
    pub _pad: [u8; 16],           // pad to union max = 32 bytes
    pub AddressLength: u32,
    pub Sub: u32,
    pub Flag: u32,
}
pub type PDNS_ADDRESS = *mut DNS_ADDRESS;

#[repr(C)]
pub struct DNS_ARRAY {
    pub AllocatedAddresses: u32,
    pub UsedAddresses: u32,
    pub Unknown: [u32; 6],
    pub Addresses: [DNS_ADDRESS; 1], // flexible, accessed beyond [0]
}
pub type PDNS_ARRAY = *mut DNS_ARRAY;

#[repr(C)]
pub struct DNS_BLOB {
    pub Name: *mut u16, // LPWSTR
    pub DnsAddrArray: PDNS_ARRAY,
    pub Hostent: *mut HOSTENT,
    pub AliasCount: u32,
    pub Unknown: u32,
    pub Aliases: [*mut u16; 8], // LPWSTR[8]
}
pub type PDNS_BLOB = *mut DNS_BLOB;

#[repr(C)]
pub struct HOSTENT {
    pub h_name: *mut i8,
    pub h_aliases: *mut *mut i8,
    pub h_addrtype: i16,
    pub h_length: i16,
    pub h_addr_list: *mut *mut i8,
}
pub type PHOSTENT = *mut HOSTENT;

#[repr(C)]
pub struct FLATBUFF {
    pub Buffer: *mut c_void,
    pub BufferEnd: *mut c_void,
    pub BufferPos: usize,
    pub BufferSize: usize,
    pub BufferFreeSize: usize,
}
pub type PFLATBUFF = *mut FLATBUFF;

// ---- From windns.h: DNS_RECORD ----

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DNS_RECORD_FLAGS {
    pub _bitfield: u32,
}

impl DNS_RECORD_FLAGS {
    pub fn Section(&self) -> u32 {
        self._bitfield & 0x3
    }
}

#[repr(C)]
pub union DNS_RECORD_FLAGS_U {
    pub DW: u32,
    pub S: DNS_RECORD_FLAGS,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DNS_A_DATA {
    pub IpAddress: IP4_ADDRESS,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DNS_AAAA_DATA {
    pub Ip6Address: IP6_ADDRESS,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DNS_ATMA_DATA {
    pub AddressType: u8,
    pub Address: [u8; 20], // DNS_ATMA_MAX_ADDR_LENGTH = 20
}

#[repr(C)]
pub union DNS_RECORD_DATA_U {
    pub A: DNS_A_DATA,
    pub AAAA: DNS_AAAA_DATA,
    pub Atma: DNS_ATMA_DATA,
}

#[repr(C)]
pub struct DNS_RECORD {
    pub pNext: *mut DNS_RECORD,
    pub pName: *mut u16,
    pub wType: u16,
    pub wDataLength: u16,
    pub Flags: DNS_RECORD_FLAGS_U,
    pub dwTtl: u32,
    pub dwReserved: u32,
    pub Data: DNS_RECORD_DATA_U,
}
