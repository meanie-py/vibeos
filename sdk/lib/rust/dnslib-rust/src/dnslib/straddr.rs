use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::dnslib::types::*;
use crate::dnslib::addr::{AF_INET, AF_INET6};
use crate::dnslib::dnsaddr::{DnsAddr_BuildFromIp4, DnsAddr_BuildFromIp6};
use crate::dnslib::string::{UnicodeString, AnsiString};

extern "C" {
    fn wsprintfW(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
}

extern "system" {
    fn Dns_StringCopy(
        dest: *mut c_void, dest_size: *mut u32,
        src: *const c_void, string_size: u32,
        input_type: u32, output_type: u32,
    ) -> u32;
    fn SetLastError(dwErrCode: u32);
    // ws2_32.dll
    fn inet_addr(cp: *const i8) -> u32;
    // ntdll.dll
    fn RtlIpv6StringToAddressA(
        addr: *const i8,
        terminator: *mut *const i8,
        dst: *mut IN6_ADDR,
    ) -> i32;
    fn RtlIpv6StringToAddressW(
        addr: *const u16,
        terminator: *mut *const u16,
        dst: *mut IN6_ADDR,
    ) -> i32;
    fn RtlIpv6StringToAddressExA(
        addr: *const i8,
        dst: *mut IN6_ADDR,
        scope_id: *mut u32,
        port: *mut u16,
    ) -> i32;
    fn RtlIpv6StringToAddressExW(
        addr: *const u16,
        dst: *mut IN6_ADDR,
        scope_id: *mut u32,
        port: *mut u16,
    ) -> i32;
}

const AF_UNSPEC: u16 = 0;
const INET6_ADDRSTRLEN: usize = 46;
const ANSI_NAME_LEN: usize = INET6_ADDRSTRLEN + 10;
const WSA_INVALID_PARAMETER: u32 = 87;
const INADDR_NONE: u32 = 0xFFFF_FFFF;

// "255.255.255.255\0" as a C string literal for strcmp
static BROADCAST_ADDR: [i8; 16] = [
    b'2' as i8, b'5' as i8, b'5' as i8, b'.' as i8,
    b'2' as i8, b'5' as i8, b'5' as i8, b'.' as i8,
    b'2' as i8, b'5' as i8, b'5' as i8, b'.' as i8,
    b'2' as i8, b'5' as i8, b'5' as i8, 0i8,
];

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4ReverseNameToAddress_A(_addr: PIN_ADDR, _name: *const i8) -> i32 {
    // FIXME in C as well
    0
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip6ReverseNameToAddress_A(_addr: PIN6_ADDR, _name: *const i8) -> i32 {
    // FIXME in C as well
    0
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4StringToAddress_A(Address: PIN_ADDR, Name: *const i8) -> i32 {
    // Use inet_addr to convert it...
    let Addr = inet_addr(Name);
    if Addr == INADDR_NONE {
        // Check if it's the wildcard (which is ok...)
        if strcmp(BROADCAST_ADDR.as_ptr(), Name) != 0 {
            return 0; // FALSE
        }
    }
    // Return the address
    (*Address).S_un.S_addr = Addr;
    1 // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4StringToAddress_W(Address: PIN_ADDR, Name: *const u16) -> i32 {
    let mut AnsiName: [i8; 16] = [0i8; 16];
    let mut Size: u32 = core::mem::size_of_val(&AnsiName) as u32;
    let ErrorCode = Dns_StringCopy(
        AnsiName.as_mut_ptr() as *mut c_void,
        &mut Size,
        Name as *const c_void,
        0,
        UnicodeString,
        AnsiString,
    );
    if ErrorCode != 0 {
        // Copy made successfully, now convert it
        return Dns_Ip4StringToAddress_A(Address, AnsiName.as_ptr());
    }
    ErrorCode as i32
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip6StringToAddress_A(Address: PIN6_ADDR, Name: *const i8) -> i32 {
    let mut Terminator: *const i8 = core::ptr::null();
    let Status = RtlIpv6StringToAddressA(Name, &mut Terminator, Address);
    if Status >= 0 { 1 } else { 0 } // NT_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip6StringToAddress_W(Address: PIN6_ADDR, Name: *const u16) -> i32 {
    let mut Terminator: *const u16 = core::ptr::null();
    let Status = RtlIpv6StringToAddressW(Name, &mut Terminator, Address);
    if Status >= 0 { 1 } else { 0 } // NT_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip6AddressToReverseName_W(
    _Name: *mut u16,
    _Address: IN6_ADDR,
) -> *mut u16 {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4AddressToReverseName_W(
    Name: *mut u16,
    Address: IN_ADDR,
) -> *mut u16 {
    static FMT_W: [u16; 26] = [
        0x0025, 0x0075, 0x002e, 0x0025, 0x0075, 0x002e, 0x0025, 0x0075, 0x002e,
        0x0025, 0x0075, 0x002e, 0x0069, 0x006e, 0x002d, 0x0061, 0x0064,
        0x0064, 0x0072, 0x002e, 0x0061, 0x0072, 0x0070, 0x0061, 0x002e,
        0x0000,
    ];

    let addr = unsafe { Address.S_un.S_addr };
    let written = wsprintfW(
        Name,
        FMT_W.as_ptr() as *const u16,
        (addr >> 24) as u32,
        ((addr >> 16) & 0xff) as u32,
        ((addr >> 8) & 0xff) as u32,
        (addr & 0xff) as u32,
    );

    unsafe { Name.offset(written as isize) }
}

#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4ReverseNameToAddress_W(
    Address: *mut IN_ADDR,
    Name: *const u16,
) -> i32 {
    let mut AnsiName: [i8; 32] = [0; 32];
    let mut Size: u32 = 32;
    let ErrorCode = Dns_StringCopy(
        AnsiName.as_mut_ptr() as *mut c_void,
        &mut Size as *mut u32,
        Name as *const c_void,
        0,
        UnicodeString,
        AnsiString,
    );
    if ErrorCode != 0 {
        return Dns_Ip4ReverseNameToAddress_A(Address, AnsiName.as_ptr());
    }
    ErrorCode as i32
}

// Helper: create a zeroed IN6_ADDR (ntdll-sys version uses union with u.Byte)
unsafe fn zero_in6() -> IN6_ADDR {
    IN6_ADDR {
        u: ntdll_sys::nt::in6_addr__bindgen_ty_1 {
            Byte: [0u8; 16],
        },
    }
}

#[no_mangle]
pub unsafe extern "system" fn Dns_StringToAddressEx(
    address: *mut c_void,
    address_size: *mut u32,
    address_name: *const c_void,
    address_family: *mut u32,
    unicode: i32,
    reverse: i32,
) -> i32 {
    let mut Af: u32 = *address_family;
    let AddrSize: u32 = *address_size;
    let mut Addr: IN6_ADDR = zero_in6();
    let mut Return: i32;
    let mut ErrorCode: i32 = 0;
    let mut AnsiName: [u8; ANSI_NAME_LEN] = [0u8; ANSI_NAME_LEN];
    let mut Size: u32 = core::mem::size_of_val(&AnsiName) as u32;
    let mut address_name = address_name;

    if reverse != 0 {
        Dns_StringCopy(
            AnsiName.as_mut_ptr() as *mut c_void,
            &mut Size,
            address_name,
            0,
            UnicodeString,
            AnsiString,
        );
        address_name = AnsiName.as_ptr() as *const c_void;
    }

    if (Af == AF_UNSPEC as u32) || (Af == AF_INET as u32) {
        if reverse != 0 {
            Return = Dns_Ip4ReverseNameToAddress_A(
                &raw mut Addr as PIN_ADDR,
                address_name as *const i8,
            );
        } else if unicode != 0 {
            Return = Dns_Ip4StringToAddress_W(
                &raw mut Addr as PIN_ADDR,
                address_name as *const u16,
            );
        } else {
            Return = Dns_Ip4StringToAddress_A(
                &raw mut Addr as PIN_ADDR,
                address_name as *const i8,
            );
        }

        if Return != 0 {
            Af = AF_INET as u32;
            if AddrSize < (core::mem::size_of::<IN_ADDR>() as u32) {
                ErrorCode = ntdll_sys::nt::ERROR_MORE_DATA as i32;
            } else {
                let src = &raw const Addr as *const IN_ADDR;
                let dst = address as PIN_ADDR;
                *dst = *src;
            }
        }
    }

    if (Af == AF_UNSPEC as u32) || (Af == AF_INET6 as u32) {
        if reverse != 0 {
            Return = Dns_Ip6ReverseNameToAddress_A(
                &raw mut Addr as PIN6_ADDR,
                address_name as *const i8,
            );
        } else if unicode != 0 {
            Return = Dns_Ip6StringToAddress_W(
                &raw mut Addr as PIN6_ADDR,
                address_name as *const u16,
            );
        } else {
            Return = Dns_Ip6StringToAddress_A(
                &raw mut Addr as PIN6_ADDR,
                address_name as *const i8,
            );
        }

        if Return != 0 {
            Af = AF_INET6 as u32;
            if AddrSize < (core::mem::size_of::<IN6_ADDR>() as u32) {
                ErrorCode = ntdll_sys::nt::ERROR_MORE_DATA as i32;
            } else {
                let dst = address as PIN6_ADDR;
                *dst = Addr;
            }
        }
    } else if Af != AF_INET as u32 {
        Af = AF_UNSPEC as u32;
        ErrorCode = WSA_INVALID_PARAMETER as i32;
    }

    if ErrorCode != 0 {
        SetLastError(ErrorCode as u32);
    }

    core::ptr::write_volatile(address_family, Af);
    core::ptr::write_volatile(address_size, AddrSize);

    if ErrorCode == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn Dns_StringToAddressW(
    Address: *mut c_void,
    AddressSize: *mut u32,
    AddressName: *mut u16,
    AddressFamily: *mut u32,
) -> i32 {
    Dns_StringToAddressEx(
        Address,
        AddressSize,
        AddressName as *mut c_void,
        AddressFamily,
        1,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn Dns_StringToDnsAddrEx(
    DnsAddr: PDNS_ADDRESS,
    AddressName: *mut c_void,
    AddressFamily: u32,
    Unicode: i32,   // flag1 = Unicode
    Reverse: i32,   // flag2 = Reverse
) -> i32 {
    let mut Addr: IN6_ADDR = zero_in6();
    let mut Return: i32 = 0;
    let mut AnsiName: [u8; ANSI_NAME_LEN] = [0u8; ANSI_NAME_LEN];
    let mut Size: u32 = core::mem::size_of_val(&AnsiName) as u32;
    let mut address_name: *mut c_void = AddressName;
    let ErrorCode: i32 = 0; // C: INT ErrorCode = ERROR_SUCCESS

    // First check if this is a reverse address string (and Unicode)
    if (Reverse != 0) && (Unicode != 0) {
        // Convert it right now to ANSI as an optimization
        Dns_StringCopy(
            AnsiName.as_mut_ptr() as *mut c_void,
            &mut Size,
            AddressName,
            0,
            UnicodeString,
            AnsiString,
        );
        address_name = AnsiName.as_mut_ptr() as *mut c_void;
    }

    // Assume IPv4 first, or use IPv4 if caller specified it
    let mut AddressFamily = AddressFamily;
    if (AddressFamily == AF_UNSPEC as u32) || (AddressFamily == AF_INET as u32) {
        if Reverse != 0 {
            Return = Dns_Ip4ReverseNameToAddress_A(
                &raw mut Addr as PIN_ADDR,
                address_name as *const i8,
            );
        } else if Unicode != 0 {
            Return = Dns_Ip4StringToAddress_W(
                &raw mut Addr as PIN_ADDR,
                address_name as *const u16,
            );
        } else {
            Return = Dns_Ip4StringToAddress_A(
                &raw mut Addr as PIN_ADDR,
                address_name as *const i8,
            );
        }

        if Return != 0 {
            // Build the IPv4 address — set AddressFamily so we skip IPv6 below
            DnsAddr_BuildFromIp4(DnsAddr, *(&raw const Addr as *const IN_ADDR), 0);
            AddressFamily = AF_INET as u32;
        }
    }

    // If we're here, either AF_INET6 was specified or IPv4 failed
    if (AddressFamily == AF_UNSPEC as u32) || (AddressFamily == AF_INET6 as u32) {
        if Reverse != 0 {
            Return = Dns_Ip6ReverseNameToAddress_A(
                &raw mut Addr as PIN6_ADDR,
                address_name as *const i8,
            );
            if Return != 0 {
                DnsAddr_BuildFromIp6(DnsAddr, &raw const Addr, 0, 0);
            } else {
                // goto Quickie — return ErrorCode == ERROR_SUCCESS (which is TRUE since ErrorCode=0)
                return (ErrorCode == 0) as i32;
            }
        } else {
            // Not reverse — use RtlIpv6StringToAddressEx which fills in scope+port directly
            let ip6 = DnsAddr as *mut SOCKADDR_IN6;
            if Unicode != 0 {
                let status = RtlIpv6StringToAddressExW(
                    address_name as *const u16,
                    &raw mut (*ip6).sin6_addr,
                    &raw mut (*ip6).sin6_scope_id,
                    &raw mut (*ip6).sin6_port,
                );
                Return = if status >= 0 { 1 } else { 0 }; // NT_SUCCESS
            } else {
                let status = RtlIpv6StringToAddressExA(
                    address_name as *const i8,
                    &raw mut (*ip6).sin6_addr,
                    &raw mut (*ip6).sin6_scope_id,
                    &raw mut (*ip6).sin6_port,
                );
                Return = if status >= 0 { 1 } else { 0 }; // NT_SUCCESS
            }
        }

        if Return != 0 {
            // Finish setting up the structure
            let ip6 = DnsAddr as *mut SOCKADDR_IN6;
            (*ip6).sin6_family = AF_INET6;
            (*DnsAddr).AddressLength = core::mem::size_of::<SOCKADDR_IN6>() as u32;
        }
    } else if AddressFamily != AF_INET as u32 {
        // ATM or something unsupported — zero and fail
        core::ptr::write_bytes(DnsAddr as *mut u8, 0, core::mem::size_of::<DNS_ADDRESS>());
        SetLastError(WSA_INVALID_PARAMETER);
    }

    // Return success or failure
    (ErrorCode == 0) as i32
}

#[no_mangle]
pub unsafe extern "system" fn Dns_ReverseNameToDnsAddr_W(
    DnsAddr: PDNS_ADDRESS,
    Name: *mut u16,
) -> i32 {
    Dns_StringToDnsAddrEx(DnsAddr, Name as *mut c_void, 0, 1, 1)
}
