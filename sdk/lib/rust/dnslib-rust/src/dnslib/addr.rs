pub use crate::dnslib::types::*;

// Address family constants from winsock
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 23;
pub const AF_ATM: u16 = 22;

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
        // sizeof(SOCKADDR_ATM) = 76 (with pshpack4 alignment):
        //   u_short satm_family(2) + pad(2) + ATM_ADDRESS(28) + ATM_BLLI(28) + ATM_BHLI(16)
        SockaddrSize: 76,
        // FIELD_OFFSET(SOCKADDR_ATM, satm_number) = 4 (after u_short + 2 bytes padding)
        AddressOffset: 4,
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
