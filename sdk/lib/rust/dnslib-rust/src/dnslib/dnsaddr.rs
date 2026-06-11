use ntdll_sys::nt::*;
use core::ffi::c_void;
pub use crate::dnslib::types::*;
use crate::dnslib::addr::{AF_INET, AF_INET6, DNS_TYPE_A, DNS_TYPE_AAAA, DNS_TYPE_ATMA, DNS_ATMA_MAX_ADDR_LENGTH};

extern "system" {
    fn Dns_Free(ptr: *mut c_void);
    fn Dns_AllocZero(size: usize) -> *mut c_void;
}

// DNS_ADDRESS: Ip4Address at offset 0 covers sin_family/sin_port.
// Ip6Address and AtmAddress are accessed via raw pointer casts.
unsafe fn dns_addr_ip6(addr: *mut DNS_ADDRESS) -> *mut SOCKADDR_IN6 {
    addr as *mut SOCKADDR_IN6
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddrArray_Create(Count: u32) -> *mut DNS_ARRAY {
    let DnsAddrArray = Dns_AllocZero(
        core::mem::size_of::<DNS_ARRAY>() + (Count as usize * core::mem::size_of::<DNS_ADDRESS>())
    ) as *mut DNS_ARRAY;

    if !DnsAddrArray.is_null() {
        (*DnsAddrArray).AllocatedAddresses = Count;
    }

    DnsAddrArray
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddrArray_Free(DnsAddrArray: *mut DNS_ARRAY) {
    Dns_Free(DnsAddrArray as *mut c_void);
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddrArray_AddIp4(
    DnsAddrArray: PDNS_ARRAY,
    Address: IN_ADDR,
    AddressType: DWORD,
) -> BOOL {
    let mut DnsAddress = core::mem::zeroed::<DNS_ADDRESS>();
    DnsAddr_BuildFromIp4(&raw mut DnsAddress, Address, 0u16);
    DnsAddrArray_AddAddr(DnsAddrArray, &raw const DnsAddress, 0u16, AddressType)
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddrArray_AddAddr(
    DnsAddrArray: *mut DNS_ARRAY,
    DnsAddress: *const DNS_ADDRESS,
    AddressFamily: u16,
    AddressType: u32,
) -> i32 {
    if DnsAddrArray.is_null() {
        return 0;
    }

    if AddressFamily != 0 {
        if AddressFamily != (*DnsAddress).Ip4Address.sin_family {
            return 1;
        }
    }

    if AddressType != 0 {
        if DnsAddrArray_ContainsAddr(DnsAddrArray, DnsAddress, AddressType) == 0 {
            return 1;
        }
    }

    if (*DnsAddrArray).AllocatedAddresses < (*DnsAddrArray).UsedAddresses {
        return 0;
    }

    let dest = (&raw mut (*DnsAddrArray).Addresses as *mut DNS_ADDRESS)
        .add((*DnsAddrArray).UsedAddresses as usize);
    core::ptr::copy_nonoverlapping(DnsAddress, dest, 1);

    1
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddr_BuildFromIp4(
    DnsAddress: *mut DNS_ADDRESS,
    Address: IN_ADDR,
    Port: u16,
) {
    core::ptr::write_bytes(DnsAddress as *mut u8, 0, core::mem::size_of::<DNS_ADDRESS>());
    (*DnsAddress).Ip4Address.sin_family = AF_INET;
    (*DnsAddress).Ip4Address.sin_port = Port;
    (*DnsAddress).Ip4Address.sin_addr = Address;
    (*DnsAddress).AddressLength = core::mem::size_of::<SOCKADDR_IN>() as u32;
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddr_BuildFromIp6(
    DnsAddress: *mut DNS_ADDRESS,
    Address: *const IN6_ADDR,
    ScopeId: u32,
    Port: u16,
) {
    core::ptr::write_bytes(DnsAddress as *mut u8, 0, core::mem::size_of::<DNS_ADDRESS>());
    let ip6 = dns_addr_ip6(DnsAddress);
    (*ip6).sin6_family = AF_INET6;
    (*ip6).sin6_port = Port;
    (*ip6).sin6_addr = *Address;
    (*ip6).sin6_scope_id = ScopeId;
    (*DnsAddress).AddressLength = core::mem::size_of::<SOCKADDR_IN6>() as u32;
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddr_BuildFromAtm(
    dns_address: *mut DNS_ADDRESS,
    address_type: u32,
    address_data: *mut c_void,
) {
    let address = ATM_ADDRESS {
        AddressType: address_type,
        NumofDigits: DNS_ATMA_MAX_ADDR_LENGTH,
        Addr: {
            let mut buf: [u8; 20] = [0; 20];
            core::ptr::copy_nonoverlapping(
                address_data as *const u8,
                buf.as_mut_ptr(),
                DNS_ATMA_MAX_ADDR_LENGTH as usize,
            );
            buf
        },
    };

    core::ptr::write_bytes(dns_address as *mut u8, 0, core::mem::size_of::<DNS_ADDRESS>());
    // Write ATM_ADDRESS at offset 4 (after WORD AddressFamily + WORD Port in the union).
    // The anonymous struct in DNS_ADDRESS is: { WORD AddressFamily; WORD Port; ATM_ADDRESS AtmAddress; }
    let atm_dst = (dns_address as *mut u8).add(4) as *mut ATM_ADDRESS;
    *atm_dst = address;
    (*dns_address).AddressLength = core::mem::size_of::<ATM_ADDRESS>() as u32;
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddr_BuildFromDnsRecord(
    DnsRecord: *mut DNS_RECORD,
    DnsAddr: *mut DNS_ADDRESS,
) -> i32 {
    match (*DnsRecord).wType {
        DNS_TYPE_A => {
            DnsAddr_BuildFromIp4(
                DnsAddr,
                IN_ADDR {
                    S_un: ntdll_sys::nt::in_addr__bindgen_ty_1 {
                        S_addr: (*DnsRecord).Data.A.IpAddress,
                    },
                },
                0,
            );
        }
        DNS_TYPE_AAAA => {
            let ip6 = (*DnsRecord).Data.AAAA.Ip6Address;
            // IP6_ADDRESS is [u32; 4], IN6_ADDR has u.Byte: [UCHAR; 16]
            // Transmute via raw bytes
            let in6: IN6_ADDR = core::mem::transmute::<[u32; 4], IN6_ADDR>(ip6);
            DnsAddr_BuildFromIp6(DnsAddr, &raw const in6, (*DnsRecord).dwReserved, 0);
        }
        DNS_TYPE_ATMA => {
            let addr_type = (*DnsRecord).Data.Atma.AddressType as u32;
            let addr_data = (*DnsRecord).Data.Atma.Address.as_ptr() as *mut c_void;
            DnsAddr_BuildFromAtm(DnsAddr, addr_type, addr_data);
        }
        _ => {}
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn DnsAddrArray_ContainsAddr(
    _DnsAddrArray: PDNS_ARRAY,
    _DnsAddress: *const DNS_ADDRESS,
    _AddressType: u32,
) -> i32 {
    // FIXME - stub returns TRUE to allow additions
    1
}
