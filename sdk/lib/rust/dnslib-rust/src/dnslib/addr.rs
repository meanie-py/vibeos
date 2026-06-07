use ntdll_sys::nt::*;
pub use crate::dnslib::types::*;

// Address family constants from winsock
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 23;
pub const AF_ATM: u16 = 16;

// DNS type constants from windns.h
pub const DNS_TYPE_A: u16 = 0x0001;
pub const DNS_TYPE_AAAA: u16 = 0x001c; // 28
pub const DNS_TYPE_ATMA: u16 = 0x0022; // 34

// DNS_ATMA_MAX_ADDR_LENGTH from windns.h
pub const DNS_ATMA_MAX_ADDR_LENGTH: u32 = 20;

// ---- AddrFamilyTable ----

pub static AddrFamilyTable: [DNS_FAMILY_INFO; 3] = [
    DNS_FAMILY_INFO {
        AddrType: AF_INET,
        DnsType: DNS_TYPE_A,
        AddressSize: core::mem::size_of::<IP4_ADDRESS>() as u32,
        SockaddrSize: core::mem::size_of::<SOCKADDR_IN>() as u32,
        AddressOffset: core::mem::offset_of!(SOCKADDR_IN, sin_addr) as u32,
    },
    DNS_FAMILY_INFO {
        AddrType: AF_INET6,
        DnsType: DNS_TYPE_AAAA,
        AddressSize: core::mem::size_of::<IP6_ADDRESS>() as u32,
        SockaddrSize: core::mem::size_of::<SOCKADDR_IN6>() as u32,
        AddressOffset: core::mem::offset_of!(SOCKADDR_IN6, sin6_addr) as u32,
    },
    DNS_FAMILY_INFO {
        AddrType: AF_ATM,
        DnsType: DNS_TYPE_ATMA,
        AddressSize: core::mem::size_of::<ATM_ADDRESS>() as u32,
        SockaddrSize: 0, // ATM has no SOCKADDR analog we use
        AddressOffset: 0,
    },
];

#[no_mangle]
pub unsafe extern "system" fn FamilyInfo_GetForFamily(AddressFamily: u16) -> *mut DNS_FAMILY_INFO {
    match AddressFamily {
        AF_INET => &raw const AddrFamilyTable[0] as *mut DNS_FAMILY_INFO,
        AF_INET6 => &raw const AddrFamilyTable[1] as *mut DNS_FAMILY_INFO,
        AF_ATM => &raw const AddrFamilyTable[2] as *mut DNS_FAMILY_INFO,
        _ => core::ptr::null_mut(),
    }
}
