use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const PERIOD: u8 = 0x2e;

pub const TIME_MAX: i32 = 2147483647;

#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut debug_trace_level: u32 = 0;
#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut path_dhclient_conf: *mut u8 = _PATH_DHCLIENT_CONF.as_ptr() as *mut u8;
#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut path_dhclient_db: *mut u8 = core::ptr::null_mut();
#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut log_perror: i32 = 1;
pub static mut privfd: i32 = 0;
pub static mut iaddr_broadcast: iaddr = iaddr {
    len: 4,
    iabuf: [255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
pub static mut inaddr_any: in_addr = in_addr { S_un: in_addr__bindgen_ty_1 { S_addr: 0 } };
pub static mut sockaddr_broadcast: sockaddr_in = sockaddr_in {
    sin_family: 0,
    sin_port: 0,
    sin_addr: in_addr { S_un: in_addr__bindgen_ty_1 { S_addr: 0 } },
    sin_zero: [(0u8) as i8; 8],
};
#[no_mangle] // C global (was file-scope in the flipped .c) — must keep the C symbol name
pub static mut log_priority: i32 = 0;
pub static mut no_daemon: i32 = 0;
pub static mut unknown_ok: i32 = 1;
pub static mut routefd: i32 = 0;
pub static mut scripttime: time_t = 0;
pub static mut leaseFile: *mut FILE = core::ptr::null_mut();

// ---- init_client ----
pub unsafe fn init_client() -> i32 {
    ApiInit();
    AdapterInit();
    tzset();

    // Zero out sockaddr_broadcast
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(sockaddr_broadcast) as *mut u8,
        0,
        core::mem::size_of_val(&sockaddr_broadcast),
    );

    sockaddr_broadcast.sin_family = (AF_INET) as i16;
    sockaddr_broadcast.sin_port = htons((REMOTE_PORT) as u16);
    sockaddr_broadcast.sin_addr.S_un.S_addr = INADDR_BROADCAST;
    inaddr_any.S_un.S_addr = INADDR_ANY;
    bootp_packet_handler = Some(do_packet);

    1i32
}

// ---- stop_client ----
pub unsafe fn stop_client() {
    // AdapterStop();
    // ApiFree();
    /* FIXME: Close pipe and kill pipe thread */
}

// ---- check_arp ----
unsafe fn check_arp(ip: *mut interface_info, lp: *mut client_lease) -> i32 {
    1
}

// ---- set_name_servers ----
#[no_mangle]
pub unsafe extern "C" fn set_name_servers(
    Adapter: PDHCP_ADAPTER,
    new_lease: *const client_lease,
) {
    let mut Buffer: [i8; 200] = {
        let src = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\\0";
        let mut buf = [0i8; 200];
        core::ptr::copy_nonoverlapping(src.as_ptr() as *const i8, buf.as_mut_ptr(), src.len());
        buf
    };
    let mut RegKey: HKEY = core::ptr::null_mut();

    // strcat(Buffer, Adapter->DhclientInfo.name)
    // We assume Adapter->DhclientInfo.name is a null-terminated char*.
    strcat(
        Buffer.as_mut_ptr(),
        (*Adapter).DhclientInfo.name.as_ptr(),
    );

    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE, // winreg.h
        Buffer.as_ptr(),
        0,
        0x20006u32, // KEY_WRITE
        &mut RegKey,
    ) != ((0u32) as i32) as u32 // ERROR_SUCCESS
    {
        return;
    }

    if (*new_lease).options[6usize].len != (0u32) as i32 {
        // DHO_DOMAIN_NAME_SERVERS = 6
        let addrs = (*new_lease).options[6usize].len / (4u32) as i32;
        let nsbuf: *mut i8 = malloc((addrs * 16) as usize) as *mut i8; // 16 bytes per IP_ADDRESS_STRING

        if !nsbuf.is_null() {
            *nsbuf = 0i8;
            let mut i: u32 = 0;
            while i < (addrs) as u32 {
                let mut nameserver: iaddr = core::mem::zeroed();
                nameserver.len = (4u32) as i32;
                core::ptr::copy_nonoverlapping(
                    (*new_lease).options[6usize].data.offset((i * 4) as isize),
                    nameserver.iabuf.as_mut_ptr(),
                    4usize,
                );
                strcat(nsbuf, piaddr(nameserver));
                if i != (addrs - 1) as u32 {
                    strcat(nsbuf, b",\0" as *const u8 as *const i8);
                }
                i += 1;
            }

            DH_DbgPrint!(
                0, // MID_TRACE (placeholder)
                b"Setting DhcpNameserver: %s\n\0" as *const u8 as *const i8,
                nsbuf,
            );

            RegSetValueExA(
                RegKey,
                b"DhcpNameServer\0" as *const u8 as *const i8,
                0,
                1u32, // REG_SZ
                nsbuf as LPBYTE,
                (strlen(nsbuf) + 1) as u32,
            );
            free(nsbuf as *mut core::ffi::c_void);
        }
    } else {
        // L"DhcpNameServer" as wide string
        let wide_str: [u16; 15] = [
            0x0044, 0x0068, 0x0063, 0x0070, 0x004e, 0x0061, 0x006d, 0x0065, 0x0053, 0x0065,
            0x0072, 0x0076, 0x0065, 0x0072, 0x0000,
        ];
        RegDeleteValueW(RegKey, wide_str.as_ptr());
    }

    RegCloseKey(RegKey);
}

// ---- unset_name_servers ----
unsafe fn unset_name_servers(Adapter: *mut DHCP_ADAPTER) {
    // (local extern block removed: it declared the advapi32 Reg* entry points as `extern "C"`
    //  (cdecl). They are __stdcall — on i386 that is a real ABI mismatch. The prelude declares
    //  them once, correctly, as `extern "system"`.)

    static WIDE_DHCPNS: [u16; 15] = [
        0x0044, 0x0068, 0x0063, 0x0070, 0x004e, 0x0061, 0x006d, 0x0065,
        0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000,
    ];

    let mut Buffer: [u8; 200] = [0u8; 200];
    let src = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\\0";
    core::ptr::copy_nonoverlapping(src.as_ptr(), Buffer.as_mut_ptr(), src.len());

    // Append adapter name
    strcat(
        Buffer.as_mut_ptr() as *mut i8,
        (*Adapter).DhclientInfo.name.as_ptr() as *const i8,
    );

    let mut RegKey: HKEY = core::ptr::null_mut();
    let result = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        Buffer.as_ptr() as *const i8,
        0,
        KEY_WRITE,
        &mut RegKey,
    );
    if result != 0 {
        return;
    }

    RegDeleteValueW(RegKey, WIDE_DHCPNS.as_ptr());
    RegCloseKey(RegKey);
}

// ---- set_domain ----
unsafe fn set_domain(Adapter: *mut _DHCP_ADAPTER, new_lease: *mut _client_lease) {
    let mut Buffer1: [u8; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut Buffer2: [u8; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let prefix1 = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\\0";
    let prefix2 = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\0";
    core::ptr::copy_nonoverlapping(prefix1.as_ptr(), Buffer1.as_mut_ptr(), prefix1.len());
    core::ptr::copy_nonoverlapping(prefix2.as_ptr(), Buffer2.as_mut_ptr(), prefix2.len());

    let buf1_ptr = Buffer1.as_mut_ptr() as *mut i8;
    let name_ptr = core::ptr::addr_of!((*Adapter).DhclientInfo.name) as *const i8;
    strcat(buf1_ptr, name_ptr);

    let mut RegKey1: HKEY = core::ptr::null_mut();
    let mut RegKey2: HKEY = core::ptr::null_mut();

    if RegOpenKeyExA(HKEY_LOCAL_MACHINE, buf1_ptr, 0, KEY_WRITE, &mut RegKey1) != 0 {
        return;
    }

    let buf2_ptr = Buffer2.as_mut_ptr() as *mut i8;
    if RegOpenKeyExA(HKEY_LOCAL_MACHINE, buf2_ptr, 0, KEY_WRITE, &mut RegKey2) != 0 {
        RegCloseKey(RegKey1);
        return;
    }

    if (*new_lease).options[DHO_DOMAIN_NAME as usize].len != 0 {
        DH_DbgPrint!(MID_TRACE, b"Setting DhcpDomain: %s\n\0".as_ptr() as *const i8, (*new_lease).options[DHO_DOMAIN_NAME as usize].data as *const i8);

        RegSetValueExA(
            RegKey1,
            b"DhcpDomain\0".as_ptr() as *const i8,
            0,
            REG_SZ,
            (*new_lease).options[DHO_DOMAIN_NAME as usize].data as LPBYTE,
            (*new_lease).options[DHO_DOMAIN_NAME as usize].len as u32,
        );
        RegSetValueExA(
            RegKey2,
            b"DhcpDomain\0".as_ptr() as *const i8,
            0,
            REG_SZ,
            (*new_lease).options[DHO_DOMAIN_NAME as usize].data as LPBYTE,
            (*new_lease).options[DHO_DOMAIN_NAME as usize].len as u32,
        );
    } else {
        let domain_wide: [u16; 11] = [0x0044, 0x0068, 0x0063, 0x0070, 0x0044, 0x006f, 0x006d, 0x0061, 0x0069, 0x006e, 0x0000];
        RegDeleteValueW(RegKey1, domain_wide.as_ptr());
        RegDeleteValueW(RegKey2, domain_wide.as_ptr());
    }

    RegCloseKey(RegKey1);
    RegCloseKey(RegKey2);
}

// ---- unset_domain ----
unsafe fn unset_domain(Adapter: PDHCP_ADAPTER) {
    const PREFIX1: &[u8] = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\";
    const PREFIX2: &[u8] = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters";
    const WIDE_DOMAIN: &[u16] = &[0x0044, 0x0068, 0x0063, 0x0070, 0x0044, 0x006f, 0x006d, 0x0061, 0x0069, 0x006e, 0x0000];

    let len1 = PREFIX1.len();
    let len2 = PREFIX2.len();
    let mut Buffer1: [u8; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut Buffer2: [u8; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    // Copy prefix1 and ensure null terminator (rest is zeros)
    core::ptr::copy_nonoverlapping(PREFIX1.as_ptr(), Buffer1.as_mut_ptr(), len1);
    *(Buffer1.as_mut_ptr().add(len1)) = 0;
    // Copy prefix2
    core::ptr::copy_nonoverlapping(PREFIX2.as_ptr(), Buffer2.as_mut_ptr(), len2);
    *(Buffer2.as_mut_ptr().add(len2)) = 0;

    // Append adapter name to Buffer1
    strcat(Buffer1.as_mut_ptr() as *mut i8, (*Adapter).DhclientInfo.name.as_ptr());

    let mut RegKey1: HKEY = core::ptr::null_mut();
    let result1 = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        (Buffer1.as_ptr() as *const u8) as *const i8,
        0,
        KEY_WRITE,
        &mut RegKey1 as *mut HKEY,
    );
    if result1 != ((ERROR_SUCCESS) as i32) as u32 {
        return;
    }

    let mut RegKey2: HKEY = core::ptr::null_mut();
    let result2 = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        (Buffer2.as_ptr() as *const u8) as *const i8,
        0,
        KEY_WRITE,
        &mut RegKey2 as *mut HKEY,
    );
    if result2 != ((ERROR_SUCCESS) as i32) as u32 {
        RegCloseKey(RegKey1);
        return;
    }

    RegDeleteValueW(RegKey1, WIDE_DOMAIN.as_ptr());
    RegDeleteValueW(RegKey2, WIDE_DOMAIN.as_ptr());

    RegCloseKey(RegKey1);
    RegCloseKey(RegKey2);
}

// ---- setup_adapter ----
unsafe fn setup_adapter(Adapter: *mut DHCP_ADAPTER, new_lease: *mut client_lease) {
    const INIT_STR: &[u8] = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\\0";
    let mut Buffer: [i8; 200] = [0i8; 200];
    core::ptr::copy_nonoverlapping(INIT_STR.as_ptr() as *const i8, Buffer.as_mut_ptr(), INIT_STR.len());

    strcat(Buffer.as_mut_ptr(), (*Adapter).DhclientInfo.name.as_ptr());

    let mut hkey: HKEY = core::ptr::null_mut();
    if RegOpenKeyExA(HKEY_LOCAL_MACHINE, Buffer.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0 {
        hkey = core::ptr::null_mut();
    }

    if (*Adapter).NteContext != 0 {
        DeleteIPAddress((*Adapter).NteContext);
        (*Adapter).NteContext = 0;
    }

    // The option length is attacker-controlled (it comes off the wire from whichever
    // host answers DHCP first) and the memcpy below targets iaddr::iabuf, which is 16
    // bytes. A subnet mask is exactly one ULONG; anything else is malformed.
    // Upstream-Commit: ed367320c29 (#9056)
    if (*new_lease).options[DHO_SUBNET_MASK as usize].len == core::mem::size_of::<u32>() as _ {
        let mut netmask: iaddr = core::mem::zeroed();
        memcpy(
            (netmask.iabuf.as_mut_ptr() as *mut core::ffi::c_void) as *mut c_void,
            ((*new_lease).options[DHO_SUBNET_MASK as usize].data as *const core::ffi::c_void) as *const c_void,
            ((*new_lease).options[DHO_SUBNET_MASK as usize].len) as usize,
        );

        let address_val = *((*new_lease).address.iabuf.as_ptr() as *const u32);
        let netmask_val = *(netmask.iabuf.as_ptr() as *const u32);
        let Status = AddIPAddress(
            address_val,
            netmask_val,
            (*Adapter).IfMib.dwIndex,
            &mut (*Adapter).NteContext,
            &mut (*Adapter).NteInstance,
        );

        if !hkey.is_null() {
            let addr_str = piaddr((*new_lease).address);
            RegSetValueExA(
                hkey,
                "DhcpIPAddress\0".as_ptr() as *const i8,
                0,
                REG_SZ,
                addr_str as LPBYTE,
                (strlen(addr_str) + 1) as u32,
            );

            Buffer[0] = 0;
            let len_mut = (*new_lease).options[DHO_SUBNET_MASK as usize].len;
            let data_mut = (*new_lease).options[DHO_SUBNET_MASK as usize].data;
            for i in 0..len_mut {
                let pos = strlen(Buffer.as_ptr());
                sprintf(
                    Buffer.as_mut_ptr().add(pos),
                    b"%u\0".as_ptr() as *const i8,
                    *data_mut.add((i) as usize) as u32,
                );
                if (i + 1) < len_mut {
                    strcat(Buffer.as_mut_ptr(), b".\0".as_ptr() as *const i8);
                }
            }
            RegSetValueExA(
                hkey,
                "DhcpSubnetMask\0".as_ptr() as *const i8,
                0,
                REG_SZ,
                Buffer.as_ptr() as LPBYTE,
                (strlen(Buffer.as_ptr()) + 1) as u32,
            );

            let server_str = piaddr((*new_lease).serveraddress);
            RegSetValueExA(
                hkey,
                "DhcpServer\0".as_ptr() as *const i8,
                0,
                REG_SZ,
                server_str as LPBYTE,
                (strlen(server_str) + 1) as u32,
            );

            RegSetValueExA(
                hkey,
                "Lease\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                &mut (*new_lease).lease as *mut _ as LPBYTE,
                core::mem::size_of::<u32>() as u32,
            );
            RegSetValueExA(
                hkey,
                "LeaseObtainedTime\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                &mut (*new_lease).obtained as *mut _ as LPBYTE,
                core::mem::size_of::<u32>() as u32,
            );
            RegSetValueExA(
                hkey,
                "LeaseTerminatesTime\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                &mut (*new_lease).expiry as *mut _ as LPBYTE,
                core::mem::size_of::<u32>() as u32,
            );
            RegSetValueExA(
                hkey,
                "T1\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                &mut (*new_lease).renewal as *mut _ as LPBYTE,
                core::mem::size_of::<u32>() as u32,
            );
            RegSetValueExA(
                hkey,
                "T2\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                &mut (*new_lease).rebind as *mut _ as LPBYTE,
                core::mem::size_of::<u32>() as u32,
            );
        }

        if !NT_SUCCESS((Status) as i32) {
            warning(b"AddIPAddress: %lx\n\0".as_ptr() as *const i8, Status);
        }
    }

    // Same wire-controlled length. A router list is a whole number of ULONGs.
    // Upstream-Commit: ed367320c29 (#9056)
    if (*new_lease).options[DHO_ROUTERS as usize].len > 0
        && (*new_lease).options[DHO_ROUTERS as usize].len % core::mem::size_of::<u32>() as i32 == 0
    {
        (*Adapter).RouterMib.dwForwardDest = 0;
        (*Adapter).RouterMib.dwForwardMask = 0;
        (*Adapter).RouterMib.dwForwardMetric1 = 1;
        (*Adapter).RouterMib.dwForwardIfIndex = (*Adapter).IfMib.dwIndex;

        if (*Adapter).RouterMib.dwForwardNextHop != 0 {
            DeleteIpForwardEntry(&mut (*Adapter).RouterMib);
        }

        (*Adapter).RouterMib.dwForwardNextHop =
            *((*new_lease).options[DHO_ROUTERS as usize].data as *const u32);

        let Status = CreateIpForwardEntry(&mut (*Adapter).RouterMib);

        if !NT_SUCCESS((Status) as i32) {
            warning(b"CreateIpForwardEntry: %lx\n\0".as_ptr() as *const i8, Status);
        }

        if !hkey.is_null() {
            Buffer[0] = 0;
            let data_mut = (*new_lease).options[DHO_ROUTERS as usize].data;
            sprintf(
                Buffer.as_mut_ptr(),
                b"%u.%u.%u.%u\0".as_ptr() as *const i8,
                *data_mut.add(0) as u32,
                *data_mut.add(1) as u32,
                *data_mut.add(2) as u32,
                *data_mut.add(3) as u32,
            );
            RegSetValueExA(
                hkey,
                "DhcpDefaultGateway\0".as_ptr() as *const i8,
                0,
                REG_SZ,
                Buffer.as_ptr() as LPBYTE,
                (strlen(Buffer.as_ptr()) + 1) as u32,
            );
        }
    }

    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
}

// ---- reset_adapter ----
unsafe fn reset_adapter(Adapter: *mut DHCP_ADAPTER) {
    let mut Buffer: [u8; 200] = {
        let src = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\0";
        let mut arr = [0u8; 200];
        core::ptr::copy_nonoverlapping(src.as_ptr(), arr.as_mut_ptr(), src.len().min(200));
        arr
    };
    let IpAddress: [u8; 8] = *b"0.0.0.0\0";
    let SubnetMask: [u8; 10] = *b"255.0.0.0\0";
    let Server: [u8; 16] = *b"255.255.255.255\0";
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut lease: DWORD = 0;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    let mut new_time: time_t = 0;

    time(core::ptr::addr_of_mut!(cur_time));

    strcat((Buffer.as_mut_ptr()) as *mut i8, (*Adapter).DhclientInfo.name.as_ptr());

    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        (Buffer.as_ptr()) as *const i8,
        0,
        KEY_WRITE,
        &mut hkey,
    ) != ((ERROR_SUCCESS) as i32) as u32
    {
        hkey = core::ptr::null_mut();
    }

    if (*Adapter).NteContext != 0 {
        DeleteIPAddress((*Adapter).NteContext);
        (*Adapter).NteContext = 0;
    }

    if !hkey.is_null() {
        RegSetValueExA(
            hkey,
            (b"DhcpIPAddress\0".as_ptr()) as *const i8,
            0,
            REG_SZ,
            IpAddress.as_ptr() as LPBYTE,
            (IpAddress.len() - 1) as DWORD,
        );
        RegSetValueExA(
            hkey,
            (b"DhcpSubnetMask\0".as_ptr()) as *const i8,
            0,
            REG_SZ,
            SubnetMask.as_ptr() as LPBYTE,
            (SubnetMask.len() - 1) as DWORD,
        );
        RegSetValueExA(
            hkey,
            (b"DhcpServer\0".as_ptr()) as *const i8,
            0,
            REG_SZ,
            Server.as_ptr() as LPBYTE,
            (Server.len() - 1) as DWORD,
        );

        lease = 3600u32;
        RegSetValueExA(
            hkey,
            (b"Lease\0".as_ptr()) as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(lease) as *mut u8,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        RegSetValueExA(
            hkey,
            (b"LeaseObtainedTime\0".as_ptr()) as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(cur_time) as *mut u8,
            core::mem::size_of::<time_t>() as DWORD,
        );
        new_time = cur_time + lease as time_t;
        RegSetValueExA(
            hkey,
            (b"LeaseTerminatesTime\0".as_ptr()) as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(new_time) as *mut u8,
            core::mem::size_of::<time_t>() as DWORD,
        );
        new_time = cur_time + lease as time_t / 2;
        RegSetValueExA(
            hkey,
            (b"T1\0".as_ptr()) as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(new_time) as *mut u8,
            core::mem::size_of::<time_t>() as DWORD,
        );
        new_time = cur_time + lease as time_t - lease as time_t / 8;
        RegSetValueExA(
            hkey,
            (b"T2\0".as_ptr()) as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(new_time) as *mut u8,
            core::mem::size_of::<time_t>() as DWORD,
        );
    }

    if (*Adapter).RouterMib.dwForwardNextHop != 0 {
        DeleteIpForwardEntry(&mut (*Adapter).RouterMib);
    }

    if !hkey.is_null() {
        RegDeleteValueA(hkey, (b"DhcpDefaultGateway\0".as_ptr()) as *const i8);
    }

    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
}

// ---- dhcp ----
pub unsafe extern "C" fn dhcp(packet: *mut packet) {
    let handler: unsafe extern "C" fn(*mut packet);
    let type_: *const i8;

    match (*packet).packet_type as u32 {
        DHCPOFFER => {
            handler = dhcpoffer;
            type_ = b"DHCPOFFER\0".as_ptr() as *const i8;
        }
        DHCPNAK => {
            handler = dhcpnak;
            type_ = b"DHCPNACK\0".as_ptr() as *const i8;
        }
        DHCPACK => {
            handler = dhcpack;
            type_ = b"DHCPACK\0".as_ptr() as *const i8;
        }
        _ => return,
    }

    let interface = (*packet).interface;
    let client = (*interface).client;
    let config = (*client).config;
    let mut ap = (*config).reject_list;

    while !ap.is_null() {
        if addr_eq((*packet).client_addr, (*ap).addr) != 0 {
            let addr_str = piaddr((*ap).addr);
            note(
                b"%s from %s rejected.\0".as_ptr() as *const i8,
                type_ as *mut i8,
                addr_str,
            );
            return;
        }
        ap = (*ap).next;
    }

    handler(packet);
}

// ---- state_panic ----
unsafe extern "C" fn state_panic(ipp: *mut core::ffi::c_void) {
    let ip: *mut _interface_info = ipp as *mut _interface_info;
    let mut address_low: u16 = 0;
    let mut i: i32 = 0;
    let mut IpAddress: u32 = 0;
    let mut Buffer: [u32; 20] = [0; 20];
    let mut BufferSize: u32 = 0;
    let mut ret: u32 = 0;
    let Adapter: *mut _DHCP_ADAPTER = AdapterFindInfo(ip);

    note(b"No DHCPOFFERS received.\0".as_ptr() as *const i8);

    if !Adapter.is_null() && (*Adapter).NteContext == 0 {
        DbgPrint(
            b"DHCPCSVC: Failed to receive a response from a DHCP server. An automatic private address will be assigned.\n\0"
                .as_ptr() as *const i8,
        );

        srand(0);

        loop {
            address_low = rand() as u16;
            i = 0;
            while i < (*ip).hw_address.hlen as i32 {
                address_low += (*ip).hw_address.haddr[i as usize] as u16;
                i += 1;
            }

            IpAddress = htonl(0xA9FE0000u32 | address_low as u32);

            BufferSize = core::mem::size_of::<[u32; 20]>() as u32;
            ret = SendARP(
                IpAddress,
                IpAddress,
                (Buffer.as_mut_ptr()) as *mut c_void,
                &mut BufferSize,
            );

            DH_DbgPrint!(
                MID_TRACE,
                b"DHCPCSVC: SendARP returned %lu\n\0".as_ptr() as *const i8,
                ret,
            );

            if ret != 0 {
                DH_DbgPrint!(
                    MID_TRACE,
                    b"DHCPCSVC: Using automatic private address\n\0".as_ptr() as *const i8,
                );
                AddIPAddress(
                    IpAddress,
                    htonl(0xFFFF0000u32),
                    (*Adapter).IfMib.dwIndex,
                    &mut (*Adapter).NteContext,
                    &mut (*Adapter).NteInstance,
                );
                return;
            }
        }
    }
}

// ---- send_decline ----
unsafe fn send_decline(ipp: *mut core::ffi::c_void) {
    let ip = ipp as *mut interface_info;
    note(
        (b"DHCPDECLINE on %s to %s port %d\0".as_ptr() as *const u8) as *const i8,
        (*ip).name,
        inet_ntoa(sockaddr_broadcast.sin_addr),
        ntohs(sockaddr_broadcast.sin_port) as u32,
    );
    send_packet(
        ip,
        core::ptr::addr_of_mut!((*(*ip).client).packet),
        (*(*ip).client).packet_length as usize,
        inaddr_any,
        core::ptr::addr_of_mut!(sockaddr_broadcast),
        core::ptr::null_mut(),
    );
}

// ---- send_release ----
unsafe extern "C" fn send_release(ipp: *mut core::ffi::c_void) {
    let ip: *mut interface_info = ipp as *mut interface_info;
    let mut destination: sockaddr_in = core::mem::zeroed();
    let mut from: in_addr = core::mem::zeroed();

    core::ptr::write_bytes(
        &mut destination as *mut _ as *mut u8,
        0,
        core::mem::size_of::<sockaddr_in>(),
    );

    memcpy(
        (core::ptr::addr_of_mut!(destination.sin_addr.S_un.S_addr) as *mut core::ffi::c_void) as *mut c_void,
        ((*(*ip).client).destination.iabuf.as_ptr() as *const core::ffi::c_void) as *const c_void,
        core::mem::size_of_val(&destination.sin_addr.S_un.S_addr),
    );

    destination.sin_port = htons(6969); // REMOTE_PORT (commonly 67/68 but placeholder)
    destination.sin_family = 2; // AF_INET

    memcpy(
        (&mut from as *mut _ as *mut core::ffi::c_void) as *mut c_void,
        ((*(*(*ip).client).active).address.iabuf.as_ptr() as *const core::ffi::c_void) as *const c_void,
        core::mem::size_of_val(&from),
    );

    note(
        b"DHCPRELEASE on %s to %s port %d\0".as_ptr() as *const core::ffi::c_char,
        (*ip).name.as_ptr() as *const core::ffi::c_char,
        inet_ntoa(destination.sin_addr),
        ntohs(destination.sin_port) as i32,
    );

    send_packet(
        ip,
        core::ptr::addr_of_mut!((*(*ip).client).packet),
        (*(*ip).client).packet_length as usize,
        from,
        core::ptr::addr_of_mut!(destination),
        core::ptr::null_mut(),
    );
}

// ---- make_discover ----
unsafe fn make_discover(ip: *mut _interface_info, lease: *mut _client_lease) {
    let discover: u8 = (DHCPDISCOVER) as u8;
    let mut option_elements: [_tree_cache; 256] = core::mem::zeroed();
    let mut options: [*mut _tree_cache; 256] = [core::ptr::null_mut(); 256];
    let mut foo: ULONG = GetTickCount();

    // dhclient.c:1442-1443 `memset(option_elements, 0, sizeof(option_elements))` — a BYTE count.
    // write_bytes' count is in ELEMENTS of the pointee, so it must go through *mut u8: with the
    // bare `as_mut_ptr()` (a *mut _tree_cache) this zeroed sizeof(array) * sizeof(_tree_cache)
    // bytes — a massive stack overrun past both arrays.
    core::ptr::write_bytes(option_elements.as_mut_ptr() as *mut u8, 0, core::mem::size_of::<[_tree_cache; 256]>());
    core::ptr::write_bytes(options.as_mut_ptr() as *mut u8, 0, core::mem::size_of::<[*mut _tree_cache; 256]>());
    core::ptr::write_bytes(&mut (*(*ip).client).packet as *mut _ as *mut u8, 0, core::mem::size_of_val(&(*(*ip).client).packet));

    // Set DHCP_MESSAGE_TYPE to DHCPDISCOVER
    let mut i: i32 = (DHO_DHCP_MESSAGE_TYPE) as i32;
    options[i as usize] = &mut option_elements[i as usize] as *mut _tree_cache;
    (*options[i as usize]).value = &discover as *const u8 as *mut u8;
    (*options[i as usize]).len = (core::mem::size_of_val(&discover) as u32) as i32;
    (*options[i as usize]).buf_size = (core::mem::size_of_val(&discover) as u32) as i32;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    // Request the options we want
    i = (DHO_DHCP_PARAMETER_REQUEST_LIST) as i32;
    options[i as usize] = &mut option_elements[i as usize] as *mut _tree_cache;
    (*options[i as usize]).value = (*(*(*ip).client).config).requested_options.as_mut_ptr();
    (*options[i as usize]).len = (*(*(*ip).client).config).requested_option_count;
    (*options[i as usize]).buf_size = (*(*(*ip).client).config).requested_option_count;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    // If we had an address, try to get it again.
    if !lease.is_null() {
        (*(*ip).client).requested_address = (*lease).address;
        i = (DHO_DHCP_REQUESTED_ADDRESS) as i32;
        options[i as usize] = &mut option_elements[i as usize] as *mut _tree_cache;
        (*options[i as usize]).value = (*lease).address.iabuf.as_mut_ptr();
        (*options[i as usize]).len = (*lease).address.len;
        (*options[i as usize]).buf_size = (*lease).address.len;
        (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;
    } else {
        (*(*ip).client).requested_address.len = 0;
    }

    // Send any options requested in the config file.
    for i in 0..256 {
        if options[i].is_null() && !(*(*(*ip).client).config).send_options[i].data.is_null() {
            options[i] = &mut option_elements[i] as *mut _tree_cache;
            (*options[i]).value = (*(*(*ip).client).config).send_options[i].data;
            (*options[i]).len = (*(*(*ip).client).config).send_options[i].len;
            (*options[i]).buf_size = (*(*(*ip).client).config).send_options[i].len;
            (*options[i]).timeout = 0xFFFFFFFFu32 as time_t;
        }
    }

    // Set up the option buffer...
    (*(*ip).client).packet_length = cons_options(core::ptr::null_mut(), &mut (*(*ip).client).packet, 0, options.as_mut_ptr());
    if (*(*ip).client).packet_length < (BOOTP_MIN_LEN) as i32 {
        (*(*ip).client).packet_length = (BOOTP_MIN_LEN) as i32;
    }

    (*(*ip).client).packet.op = (BOOTREQUEST) as u8;
    (*(*ip).client).packet.htype = (*ip).hw_address.htype;
    (*(*ip).client).packet.hlen = (*ip).hw_address.hlen;
    (*(*ip).client).packet.hops = 0;
    (*(*ip).client).packet.xid = RtlRandom(&mut foo);
    (*(*ip).client).packet.secs = 0;
    (*(*ip).client).packet.flags = 0;

    core::ptr::write_bytes(&mut (*(*ip).client).packet.ciaddr as *mut _ as *mut u8, 0, core::mem::size_of_val(&(*(*ip).client).packet.ciaddr));
    core::ptr::write_bytes(&mut (*(*ip).client).packet.yiaddr as *mut _ as *mut u8, 0, core::mem::size_of_val(&(*(*ip).client).packet.yiaddr));
    core::ptr::write_bytes(&mut (*(*ip).client).packet.siaddr as *mut _ as *mut u8, 0, core::mem::size_of_val(&(*(*ip).client).packet.siaddr));
    core::ptr::write_bytes(&mut (*(*ip).client).packet.giaddr as *mut _ as *mut u8, 0, core::mem::size_of_val(&(*(*ip).client).packet.giaddr));
    core::ptr::copy_nonoverlapping(
        (*ip).hw_address.haddr.as_ptr(),
        (*(*ip).client).packet.chaddr.as_mut_ptr(),
        (*ip).hw_address.hlen as usize,
    );
}

// ---- make_request ----
unsafe fn make_request(ip: *mut interface_info, lease: *mut client_lease) {
    let mut request: u8 = DHCPREQUEST as u8;
    let mut options: [*mut tree_cache; 256] = [core::ptr::null_mut(); 256];
    let mut option_elements: [tree_cache; 256] = core::mem::zeroed();
    let mut i: i32;

    core::ptr::write_bytes(options.as_mut_ptr(), 0, 256);
    core::ptr::write_bytes(
        &mut (*((*ip).client)).packet as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&(*((*ip).client)).packet),
    );

    /* Set DHCP_MESSAGE_TYPE to DHCPREQUEST */
    i = DHO_DHCP_MESSAGE_TYPE as i32;
    options[i as usize] = &mut option_elements[i as usize] as *mut tree_cache;
    (*(options[i as usize])).value = &mut request as *mut _;
    (*(options[i as usize])).len = (core::mem::size_of_val(&request) as u32) as i32;
    (*(options[i as usize])).buf_size = (core::mem::size_of_val(&request) as u32) as i32;
    (*(options[i as usize])).timeout = 0xFFFFFFFFu32 as time_t;

    /* Request the options we want */
    i = DHO_DHCP_PARAMETER_REQUEST_LIST as i32;
    options[i as usize] = &mut option_elements[i as usize] as *mut tree_cache;
    (*(options[i as usize])).value = (*(*((*ip).client)).config).requested_options.as_mut_ptr();
    (*(options[i as usize])).len = (*(*((*ip).client)).config).requested_option_count;
    (*(options[i as usize])).buf_size = (*(*((*ip).client)).config).requested_option_count;
    (*(options[i as usize])).timeout = 0xFFFFFFFFu32 as time_t;

    /* If we are requesting an address that hasn't yet been assigned
       to us, use the DHCP Requested Address option. */
    if (*((*ip).client)).state == S_REQUESTING {
        /* Send back the server identifier... */
        i = DHO_DHCP_SERVER_IDENTIFIER as i32;
        options[i as usize] = &mut option_elements[i as usize] as *mut tree_cache;
        (*(options[i as usize])).value = (*lease).options[i as usize].data;
        (*(options[i as usize])).len = (*lease).options[i as usize].len;
        (*(options[i as usize])).buf_size = (*lease).options[i as usize].len;
        (*(options[i as usize])).timeout = 0xFFFFFFFFu32 as time_t;
    }
    if (*((*ip).client)).state == S_REQUESTING || (*((*ip).client)).state == S_REBOOTING {
        (*((*ip).client)).requested_address = (*lease).address;
        i = DHO_DHCP_REQUESTED_ADDRESS as i32;
        options[i as usize] = &mut option_elements[i as usize] as *mut tree_cache;
        (*(options[i as usize])).value = (*lease).address.iabuf.as_mut_ptr();
        (*(options[i as usize])).len = (*lease).address.len;
        (*(options[i as usize])).buf_size = (*lease).address.len;
        (*(options[i as usize])).timeout = 0xFFFFFFFFu32 as time_t;
    } else {
        (*((*ip).client)).requested_address.len = 0;
    }

    /* Send any options requested in the config file. */
    i = 0;
    while i < 256 {
        if options[i as usize].is_null()
            && !(*(*((*ip).client)).config).send_options[i as usize].data.is_null()
        {
            options[i as usize] = &mut option_elements[i as usize] as *mut tree_cache;
            (*(options[i as usize])).value = (*(*((*ip).client)).config).send_options[i as usize].data;
            (*(options[i as usize])).len = (*(*((*ip).client)).config).send_options[i as usize].len;
            (*(options[i as usize])).buf_size =
                (*(*((*ip).client)).config).send_options[i as usize].len;
            (*(options[i as usize])).timeout = 0xFFFFFFFFu32 as time_t;
        }
        i += 1;
    }

    /* Set up the option buffer... */
    (*((*ip).client)).packet_length =
        cons_options(core::ptr::null_mut(), &mut (*((*ip).client)).packet, 0, options.as_mut_ptr());
    if (*((*ip).client)).packet_length < (BOOTP_MIN_LEN as u32) as i32 {
        (*((*ip).client)).packet_length = (BOOTP_MIN_LEN as u32) as i32;
    }

    (*((*ip).client)).packet.op = BOOTREQUEST as u8;
    (*((*ip).client)).packet.htype = (*ip).hw_address.htype;
    (*((*ip).client)).packet.hlen = (*ip).hw_address.hlen;
    (*((*ip).client)).packet.hops = 0;
    (*((*ip).client)).packet.xid = (*((*ip).client)).xid;
    (*((*ip).client)).packet.secs = 0; /* Filled in by send_request. */

    /* If we own the address we're requesting, put it in ciaddr;
       otherwise set ciaddr to zero. */
    if (*((*ip).client)).state == S_BOUND
        || (*((*ip).client)).state == S_RENEWING
        || (*((*ip).client)).state == S_REBINDING
    {
        core::ptr::copy_nonoverlapping(
            (*lease).address.iabuf.as_ptr(),
            &mut (*((*ip).client)).packet.ciaddr as *mut _ as *mut u8,
            (*lease).address.len as usize,
        );
        (*((*ip).client)).packet.flags = 0;
    } else {
        core::ptr::write_bytes(
            &mut (*((*ip).client)).packet.ciaddr as *mut _ as *mut u8,
            0,
            core::mem::size_of_val(&(*((*ip).client)).packet.ciaddr),
        );
        (*((*ip).client)).packet.flags = 0;
    }

    core::ptr::write_bytes(
        &mut (*((*ip).client)).packet.yiaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&(*((*ip).client)).packet.yiaddr),
    );
    core::ptr::write_bytes(
        &mut (*((*ip).client)).packet.siaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&(*((*ip).client)).packet.siaddr),
    );
    core::ptr::write_bytes(
        &mut (*((*ip).client)).packet.giaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&(*((*ip).client)).packet.giaddr),
    );
    core::ptr::copy_nonoverlapping(
        (*ip).hw_address.haddr.as_ptr(),
        (*((*ip).client)).packet.chaddr.as_mut_ptr(),
        (*ip).hw_address.hlen as usize,
    );
}

// ---- make_decline ----
unsafe fn make_decline(ip: *mut interface_info, lease: *mut client_lease) {
    let mut options: [*mut tree_cache; 256] = [core::ptr::null_mut(); 256];
    let mut message_type_tree: tree_cache = core::mem::zeroed();
    let mut requested_address_tree: tree_cache = core::mem::zeroed();
    let mut server_id_tree: tree_cache = core::mem::zeroed();
    let mut client_id_tree: tree_cache = core::mem::zeroed();
    let decline: u8 = (DHCPDECLINE) as u8;
    let mut i: i32;

    let client = &mut *(*ip).client;
    let hw = &(*ip).hw_address;
    let config = &*client.config;

    core::ptr::write_bytes(
        &mut client.packet as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&client.packet),
    );

    /* Set DHCP_MESSAGE_TYPE to DHCPDECLINE */
    i = (DHO_DHCP_MESSAGE_TYPE) as i32;
    options[i as usize] = &mut message_type_tree;
    (*options[i as usize]).value = &decline as *const _ as *mut u8;
    (*options[i as usize]).len = (core::mem::size_of::<u8>() as u32) as i32;
    (*options[i as usize]).buf_size = (core::mem::size_of::<u8>() as u32) as i32;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    /* Send back the server identifier... */
    i = (DHO_DHCP_SERVER_IDENTIFIER) as i32;
    options[i as usize] = &mut server_id_tree;
    (*options[i as usize]).value = (*lease).options[i as usize].data;
    (*options[i as usize]).len = (*lease).options[i as usize].len;
    (*options[i as usize]).buf_size = (*lease).options[i as usize].len;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    /* Send back the address we're declining. */
    i = (DHO_DHCP_REQUESTED_ADDRESS) as i32;
    options[i as usize] = &mut requested_address_tree;
    (*options[i as usize]).value = (*lease).address.iabuf.as_mut_ptr();
    (*options[i as usize]).len = (*lease).address.len;
    (*options[i as usize]).buf_size = (*lease).address.len;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    /* Send the uid if the user supplied one. */
    i = (DHO_DHCP_CLIENT_IDENTIFIER) as i32;
    if config.send_options[i as usize].len != 0 {
        options[i as usize] = &mut client_id_tree;
        (*options[i as usize]).value = config.send_options[i as usize].data;
        (*options[i as usize]).len = config.send_options[i as usize].len;
        (*options[i as usize]).buf_size = config.send_options[i as usize].len;
        (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;
    }

    /* Set up the option buffer... */
    client.packet_length = cons_options(core::ptr::null_mut(), &mut client.packet, 0, options.as_mut_ptr());
    if client.packet_length < (BOOTP_MIN_LEN) as i32 {
        client.packet_length = (BOOTP_MIN_LEN) as i32;
    }

    client.packet.op = (BOOTREQUEST) as u8;
    client.packet.htype = hw.htype;
    client.packet.hlen = hw.hlen;
    client.packet.hops = 0;
    client.packet.xid = client.xid;
    client.packet.secs = 0;  /* Filled in by send_request. */
    client.packet.flags = 0;

    /* ciaddr must always be zero. */
    core::ptr::write_bytes(
        &mut client.packet.ciaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&client.packet.ciaddr),
    );
    core::ptr::write_bytes(
        &mut client.packet.yiaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&client.packet.yiaddr),
    );
    core::ptr::write_bytes(
        &mut client.packet.siaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&client.packet.siaddr),
    );
    core::ptr::write_bytes(
        &mut client.packet.giaddr as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&client.packet.giaddr),
    );
    core::ptr::copy_nonoverlapping(
        hw.haddr.as_ptr(),
        client.packet.chaddr.as_mut_ptr(),
        hw.hlen as usize,
    );
}

// ---- make_release ----
#[no_mangle]
pub unsafe extern "C" fn make_release(
    ip: *mut _interface_info,
    lease: *mut _client_lease,
) {
    let mut options: [*mut _tree_cache; 256] = [core::ptr::null_mut(); 256];
    let mut message_type_tree: _tree_cache = core::mem::zeroed();
    let mut server_id_tree: _tree_cache = core::mem::zeroed();
    let mut client_id_tree: _tree_cache = core::mem::zeroed();
    let mut release: u8 = (DHCPRELEASE) as u8;
    let mut i: i32;

    // Zero the packet
    core::ptr::write_bytes(
        &mut (*(*ip).client).packet as *mut dhcp_packet as *mut u8,
        0,
        core::mem::size_of::<_packet>(),
    );

    // Set DHCP_MESSAGE_TYPE
    i = (DHO_DHCP_MESSAGE_TYPE) as i32;
    options[i as usize] = &mut message_type_tree;
    (*options[i as usize]).value = &mut release as *mut u8;
    (*options[i as usize]).len = (core::mem::size_of::<u8>() as u32) as i32;
    (*options[i as usize]).buf_size = (core::mem::size_of::<u8>() as u32) as i32;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    // Send back the server identifier
    i = (DHO_DHCP_SERVER_IDENTIFIER) as i32;
    options[i as usize] = &mut server_id_tree;
    (*options[i as usize]).value = (*lease).options[i as usize].data;
    (*options[i as usize]).len = (*lease).options[i as usize].len;
    (*options[i as usize]).buf_size = (*lease).options[i as usize].len;
    (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;

    // Send the uid if the user supplied one
    i = (DHO_DHCP_CLIENT_IDENTIFIER) as i32;
    if (*(*(*ip).client).config).send_options[i as usize].len != 0 {
        options[i as usize] = &mut client_id_tree;
        (*options[i as usize]).value = ((*((*(*(*ip).client).config).send_options[i as usize].data))) as *mut u8;
        (*options[i as usize]).len = (*(*(*ip).client).config).send_options[i as usize].len;
        (*options[i as usize]).buf_size = (*(*(*ip).client).config).send_options[i as usize].len;
        (*options[i as usize]).timeout = 0xFFFFFFFFu32 as time_t;
    }

    // Set up the option buffer
    (*(*ip).client).packet_length = cons_options(
        core::ptr::null_mut(),
        &mut (*(*ip).client).packet,
        (0u32) as i32,
        options.as_mut_ptr(),
    );
    if (*(*ip).client).packet_length < (BOOTP_MIN_LEN) as i32 {
        (*(*ip).client).packet_length = (BOOTP_MIN_LEN) as i32;
    }

    (*(*ip).client).packet.op = (BOOTREQUEST) as u8;
    (*(*ip).client).packet.htype = (*ip).hw_address.htype;
    (*(*ip).client).packet.hlen = (*ip).hw_address.hlen;
    (*(*ip).client).packet.hops = 0;
    (*(*ip).client).packet.xid = 0;
    (*(*ip).client).packet.secs = 0;
    (*(*ip).client).packet.flags = 0;

    // ciaddr is the address to be released
    memcpy(
        (&mut (*(*ip).client).packet.ciaddr as *mut _ as *mut u8) as *mut c_void,
        ((*lease).address.iabuf.as_ptr()) as *const c_void,
        (*lease).address.len as usize,
    );
    memset(
        (&mut (*(*ip).client).packet.yiaddr as *mut _ as *mut u8) as *mut c_void,
        0,
        core::mem::size_of_val(&(*lease).address.iabuf),  // approximate; replace with proper size
    );
    memset(
        (&mut (*(*ip).client).packet.siaddr as *mut _ as *mut u8) as *mut c_void,
        0,
        core::mem::size_of_val(&(*lease).address.iabuf),
    );
    memset(
        (&mut (*(*ip).client).packet.giaddr as *mut _ as *mut u8) as *mut c_void,
        0,
        core::mem::size_of_val(&(*lease).address.iabuf),
    );
    memcpy(
        ((*(*ip).client).packet.chaddr.as_mut_ptr()) as *mut c_void,
        ((*ip).hw_address.haddr.as_ptr()) as *const c_void,
        (*ip).hw_address.hlen as usize,
    );
}

// ---- free_client_lease ----
unsafe fn free_client_lease(lease: *mut client_lease) {
    if !(*lease).server_name.is_null() {
        free((*lease).server_name as *mut core::ffi::c_void);
    }
    if !(*lease).filename.is_null() {
        free((*lease).filename as *mut core::ffi::c_void);
    }
    for i in 0..256 {
        if (*lease).options[i].len != 0 {
            free((*lease).options[i].data as *mut core::ffi::c_void);
        }
    }
    free(lease as *mut core::ffi::c_void);
}

// ---- priv_script_init ----
pub unsafe extern "system" fn priv_script_init(ip: *mut interface_info, reason: *mut i8, medium: *mut i8) {}

// ---- priv_script_write_params ----
#[no_mangle]
pub unsafe extern "system" fn priv_script_write_params(
    ip: *mut interface_info,
    prefix: *mut i8,
    lease: *mut client_lease,
) {
    let _ = prefix; // unused
    let mut dbuf: [u8; 1500] = [0; 1500];
    let mut dp: *mut u8;
    let mut len: i32;

    let client = (*ip).client;
    let config = (*client).config;
    let defaults = (*config).defaults;
    let default_actions = (*config).default_actions;
    let options = (*lease).options;

    for i in 0..256 {
        dp = core::ptr::null_mut();
        if defaults[i].len != 0 {
            if options[i].len != 0 {
                match default_actions[i] {
                    0 => { // ACTION_DEFAULT
                        dp = options[i].data;
                        len = options[i].len;
                    }
                    1 => { // ACTION_SUPERSEDE
                        dp = defaults[i].data;
                        len = defaults[i].len;
                    }
                    2 => { // ACTION_PREPEND
                        let new_len = defaults[i].len + options[i].len;
                        if new_len >= 1500 {
                            warning(
                                b"no space to %s %s\0".as_ptr() as *const i8,
                                b"prepend option\0".as_ptr() as *const i8,
                                dhcp_options[i].name,
                            );
                            dp = defaults[i].data;
                            len = defaults[i].len;
                        } else {
                            core::ptr::copy_nonoverlapping(
                                defaults[i].data,
                                dbuf.as_mut_ptr(),
                                defaults[i].len as usize,
                            );
                            core::ptr::copy_nonoverlapping(
                                options[i].data,
                                dbuf.as_mut_ptr().add(defaults[i].len as usize),
                                options[i].len as usize,
                            );
                            *dbuf.as_mut_ptr().add(new_len as usize) = 0u8;
                            dp = dbuf.as_mut_ptr();
                            len = new_len;
                        }
                    }
                    3 => { // ACTION_APPEND
                        let new_len = defaults[i].len + options[i].len + 1;
                        if new_len > 1500 {
                            warning(
                                b"no space to %s %s\0".as_ptr() as *const i8,
                                b"append option\0".as_ptr() as *const i8,
                                dhcp_options[i].name,
                            );
                            dp = defaults[i].data;
                            len = defaults[i].len;
                        } else {
                            core::ptr::copy_nonoverlapping(
                                options[i].data,
                                dbuf.as_mut_ptr(),
                                options[i].len as usize,
                            );
                            core::ptr::copy_nonoverlapping(
                                defaults[i].data,
                                dbuf.as_mut_ptr().add(options[i].len as usize),
                                defaults[i].len as usize,
                            );
                            *dbuf.as_mut_ptr().add(new_len as usize - 1) = 0u8;
                            dp = dbuf.as_mut_ptr();
                            len = new_len;
                        }
                    }
                    _ => {}
                }
            } else {
                dp = defaults[i].data;
                len = defaults[i].len;
            }
        } else if options[i].len != 0 {
            dp = options[i].data;
            len = options[i].len;
        } else {
            len = 0;
        }
    }
}

// ---- dhcp_option_ev_name ----
unsafe fn dhcp_option_ev_name(buf: *mut u8, buflen: usize, option: *mut dhcp_option) -> i32 {
    let name_ptr = core::ptr::addr_of!((*option).name) as *const u8;
    let mut i = 0;
    loop {
        let c = *name_ptr.add(i);
        if c == 0 {
            *buf.add(i) = 0;
            return 1;
        }
        if i + 1 == buflen {
            return 0;
        }
        if c == b'-' {
            *buf.add(i) = b'_';
        } else {
            *buf.add(i) = c;
        }
        i += 1;
    }
}

// ---- res_hnok ----
unsafe fn res_hnok(dn: *const i8) -> i32 {
    let mut p = dn;
    let mut pch: i32 = 0x2e_i32;
    let mut ch: i32 = *p as i32;
    p = p.offset(1);
    while ch != 0 {
        let nch: i32 = *p as i32;
        p = p.offset(1);
        if periodchar(ch) != 0 {
            // nothing
        } else if periodchar(pch) != 0 {
            if borderchar(ch) == 0 {
                return 0;
            }
        } else if periodchar(nch) != 0 || nch == 0 {
            if borderchar(ch) == 0 {
                return 0;
            }
        } else {
            if middlechar(ch) == 0 {
                return 0;
            }
        }
        pch = ch;
        ch = nch;
    }
    1
}

// ---- ipv4addrs ----
unsafe fn ipv4addrs(mut buf: *mut i8) -> i32 {
    let mut tmp: *mut i8;
    let mut jnk = core::mem::zeroed::<in_addr>();
    let mut i: i32 = 0;

    note(b"Input: %s\0".as_ptr() as *const i8, buf);

    loop {
        tmp = strtok(buf, b" \0".as_ptr() as *const i8);
        note(b"got %s\0".as_ptr() as *const i8, tmp);
        if !tmp.is_null() && inet_aton((tmp as *const i8) as *const u8, &mut jnk as *mut in_addr) != 0 {
            i += 1;
        }
        buf = core::ptr::null_mut();
        if tmp.is_null() {
            break;
        }
    }

    i
}

// ---- option_as_string ----
unsafe fn option_as_string(code: u32, data: *mut u8, len: i32) -> *mut i8 {
    static mut OPTBUF: [u8; 32768] = [0; 32768];
    let mut op = OPTBUF.as_mut_ptr() as *mut i8;
    let mut opleft: i32 = 32768;
    let mut dp = data;
    let end = data.offset(len as isize);

    if code > 255 {
        error(b"option_as_string: bad code %d\0".as_ptr() as *const i8, code);
    }

    while dp < end {
        let c = *dp;
        if isascii(c as i32) == 0 || isprint(c as i32) == 0 {
            if dp.offset(1) != end || c != 0 {
                _snprintf(
                    op,
                    opleft as usize,
                    b"\\%03o\0".as_ptr() as *const i8,
                    c as i32,
                );
                op = op.offset(4);
                opleft -= 4;
            }
        } else if c == b'"' || c == b'\'' || c == b'$' || c == b'`' || c == b'\\' {
            *op = b'\\' as i8;
            op = op.offset(1);
            *op = c as i8;
            op = op.offset(1);
            opleft -= 2;
        } else {
            *op = c as i8;
            op = op.offset(1);
            opleft -= 1;
        }
        dp = dp.offset(1);
    }

    if opleft < 1 {
        warning(b"dhcp option too large\0".as_ptr() as *const i8);
        return b"<error>\0".as_ptr() as *mut i8;
    }
    *op = 0;
    OPTBUF.as_mut_ptr() as *mut i8
}

// ---- send_discover ----
unsafe extern "C" fn send_discover(ipp: *mut core::ffi::c_void) {
    let ip = ipp as *mut interface_info;
    let (mut interval, mut increase) = (0i32, 1i32);
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    DH_DbgPrint!(MID_TRACE, b"Doing discover on interface %p\n\0".as_ptr() as *const i8, ip);
    time(core::ptr::addr_of_mut!(cur_time) as *mut time_t);
    interval = (cur_time - (*(*ip).client).first_sending) as i32;
    if interval > ((*(*(*ip).client).config).timeout) as i32 {
        state_panic((ip) as *mut c_void);
        (*(*ip).client).first_sending = cur_time;
    }
    if (*(*ip).client).offered_leases.is_null() && !(*(*(*ip).client).config).media.is_null() {
        let mut fail = 0i32;
        if !(*(*ip).client).medium.is_null() {
            (*(*ip).client).medium = (*(*(*ip).client).medium).next;
            increase = 0;
        }
        if (*(*ip).client).medium.is_null() {
            if fail != 0 {
                error(b"No valid media types for %s!\0".as_ptr() as *const i8, (*ip).name);
            }
            (*(*ip).client).medium = (*(*(*ip).client).config).media;
            increase = 1;
        }
        note(b"Trying medium \"%s\" %d\0".as_ptr() as *const i8, (*(*(*ip).client).medium).string, increase);
    }
    if increase != 0 {
        if (*(*ip).client).interval == 0 {
            (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;
        } else {
            (*(*ip).client).interval += ((rand() >> 2) as time_t) % (2 * (*(*ip).client).interval);
        }
        if (*(*ip).client).interval > (*(*(*ip).client).config).backoff_cutoff {
            (*(*ip).client).interval = ((*(*(*ip).client).config).backoff_cutoff / 2)
                + (((rand() >> 2) as time_t) % (*(*(*ip).client).config).backoff_cutoff);
        }
    } else if (*(*ip).client).interval == 0 {
        (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;
    }
    if cur_time + (*(*ip).client).interval > (*(*ip).client).first_sending + (*(*(*ip).client).config).timeout {
        (*(*ip).client).interval = ((*(*ip).client).first_sending + (*(*(*ip).client).config).timeout) - cur_time + 1;
    }
    if interval < 65536 {
        (*(*ip).client).packet.secs = htons(interval as u16);
    } else {
        (*(*ip).client).packet.secs = htons(65535u16);
    }
    (*(*ip).client).secs = (*(*ip).client).packet.secs;
    note(
        b"DHCPDISCOVER on %s to %s port %d interval %ld\0".as_ptr() as *const i8,
        (*ip).name,
        inet_ntoa(sockaddr_broadcast.sin_addr),
        ntohs(sockaddr_broadcast.sin_port) as i32,
        (*(*ip).client).interval as i64,
    );
    send_packet(
        ip,
        &mut (*(*ip).client).packet,
        ((*(*ip).client).packet_length) as usize,
        inaddr_any,
        &mut sockaddr_broadcast,
        core::ptr::null_mut(),
    );
    DH_DbgPrint!(
        MID_TRACE,
        b"discover timeout: now %x -> then %x\n\0".as_ptr() as *const i8,
        cur_time,
        cur_time + (*(*ip).client).interval,
    );
    add_timeout((cur_time + (*(*ip).client).interval) as time_t, send_discover, (ip) as *mut c_void);
}

// ---- bind_lease ----
unsafe fn bind_lease(ip: *mut interface_info) {
    let Adapter: *mut DHCP_ADAPTER;
    let new_lease: *mut client_lease = (*(*ip).client).new;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;

    time(core::ptr::addr_of_mut!(cur_time));

    (*(*(*ip).client).new).medium = (*(*ip).client).medium;

    if !((*(*ip).client).active).is_null() {
        free_client_lease((*(*ip).client).active);
    }
    (*(*ip).client).active = (*(*ip).client).new;
    (*(*ip).client).new = core::ptr::null_mut();

    if ((*(*(*ip).client).active).renewal - cur_time) != 0 {
        add_timeout(((*(*(*ip).client).active).renewal) as time_t, state_bound, (ip) as *mut c_void);
    }

    note(
        b"bound to %s -- renewal in %ld seconds.\0".as_ptr() as *const i8,
        piaddr((*(*(*ip).client).active).address),
        ((*(*(*ip).client).active).renewal - cur_time) as i64,
    );

    (*(*ip).client).state = S_BOUND;

    Adapter = AdapterFindInfo(ip);

    if !Adapter.is_null() {
        setup_adapter(Adapter, new_lease);
    } else {
        warning(
            b"Could not find adapter for info %p\n\0".as_ptr() as *const i8,
            ip,
        );
        return;
    }
    set_name_servers(Adapter, new_lease);
    set_domain(Adapter, new_lease);
}

// ---- unbind_lease ----
unsafe fn unbind_lease(ip: *mut interface_info) {
    let active = (*(*ip).client).active;
    if !active.is_null() {
        free_client_lease(active);
        (*(*ip).client).active = core::ptr::null_mut();
    }

    let adapter = AdapterFindInfo(ip);
    if !adapter.is_null() {
        reset_adapter(adapter);
        unset_name_servers(adapter);
        unset_domain(adapter);
    } else {
        warning(b"Could not find adapter for info %p\n\0".as_ptr() as *const i8, ip);
        return;
    }
}

// ---- check_option ----
unsafe fn check_option(l: *mut client_lease, option: i32) -> i32 {
    let opbuf: *mut i8;
    let sbuf: *mut i8;

    opbuf = (pretty_print_option(
        (option) as u32,
        (*l).options[option as usize].data,
        (*l).options[option as usize].len,
        0,
        0,
    )) as *mut i8;

    sbuf = option_as_string(
        (option) as u32,
        (*l).options[option as usize].data,
        (*l).options[option as usize].len,
    );

    match option as u32 {
        DHO_SUBNET_MASK
        | DHO_TIME_SERVERS
        | DHO_NAME_SERVERS
        | DHO_ROUTERS
        | DHO_DOMAIN_NAME_SERVERS
        | DHO_LOG_SERVERS
        | DHO_COOKIE_SERVERS
        | DHO_LPR_SERVERS
        | DHO_IMPRESS_SERVERS
        | DHO_RESOURCE_LOCATION_SERVERS
        | DHO_SWAP_SERVER
        | DHO_BROADCAST_ADDRESS
        | DHO_NIS_SERVERS
        | DHO_NTP_SERVERS
        | DHO_NETBIOS_NAME_SERVERS
        | DHO_NETBIOS_DD_SERVER
        | DHO_FONT_SERVERS
        | DHO_DHCP_SERVER_IDENTIFIER => {
            if ipv4addrs(opbuf) == 0 {
                warning(
                    "Invalid IP address in option(%d): %s\0".as_ptr() as *const i8,
                    option,
                    opbuf,
                );
                return 0;
            }
            1
        }
        DHO_HOST_NAME | DHO_DOMAIN_NAME | DHO_NIS_DOMAIN => {
            if res_hnok(sbuf as *const i8) == 0 {
                warning(
                    "Bogus Host Name option %d: %s (%s)\0".as_ptr() as *const i8,
                    option,
                    sbuf,
                    opbuf,
                );
            }
            1
        }
        DHO_PAD
        | DHO_TIME_OFFSET
        | DHO_BOOT_SIZE
        | DHO_MERIT_DUMP
        | DHO_ROOT_PATH
        | DHO_EXTENSIONS_PATH
        | DHO_IP_FORWARDING
        | DHO_NON_LOCAL_SOURCE_ROUTING
        | DHO_POLICY_FILTER
        | DHO_MAX_DGRAM_REASSEMBLY
        | DHO_DEFAULT_IP_TTL
        | DHO_PATH_MTU_AGING_TIMEOUT
        | DHO_PATH_MTU_PLATEAU_TABLE
        | DHO_INTERFACE_MTU
        | DHO_ALL_SUBNETS_LOCAL
        | DHO_PERFORM_MASK_DISCOVERY
        | DHO_MASK_SUPPLIER
        | DHO_ROUTER_DISCOVERY
        | DHO_ROUTER_SOLICITATION_ADDRESS
        | DHO_STATIC_ROUTES
        | DHO_TRAILER_ENCAPSULATION
        | DHO_ARP_CACHE_TIMEOUT
        | DHO_IEEE802_3_ENCAPSULATION
        | DHO_DEFAULT_TCP_TTL
        | DHO_TCP_KEEPALIVE_INTERVAL
        | DHO_TCP_KEEPALIVE_GARBAGE
        | DHO_VENDOR_ENCAPSULATED_OPTIONS
        | DHO_NETBIOS_NODE_TYPE
        | DHO_NETBIOS_SCOPE
        | DHO_X_DISPLAY_MANAGER
        | DHO_DHCP_REQUESTED_ADDRESS
        | DHO_DHCP_LEASE_TIME
        | DHO_DHCP_OPTION_OVERLOAD
        | DHO_DHCP_MESSAGE_TYPE
        | DHO_DHCP_PARAMETER_REQUEST_LIST
        | DHO_DHCP_MESSAGE
        | DHO_DHCP_MAX_MESSAGE_SIZE
        | DHO_DHCP_RENEWAL_TIME
        | DHO_DHCP_REBINDING_TIME
        | DHO_DHCP_CLASS_IDENTIFIER
        | DHO_DHCP_CLIENT_IDENTIFIER
        | DHO_DHCP_USER_CLASS_ID
        | DHO_END => 1,
        _ => {
            warning(
                "unknown dhcp option value 0x%x\0".as_ptr() as *const i8,
                option,
            );
            unknown_ok
        }
    }
}

// ---- state_init ----
pub unsafe extern "C" fn state_init(ipp: *mut core::ffi::c_void) {
    let ip = ipp as *mut _interface_info;
    // ASSERT_STATE(state, S_INIT); // macro expands to nothing
    let client: *mut client_state = (*ip).client;
    make_discover(ip, (*client).active);
    (*client).xid = (*client).packet.xid;
    (*client).destination = iaddr_broadcast;
    (*client).state = _dhcp_state_S_SELECTING;
    time(core::ptr::addr_of_mut!((*client).first_sending));
    (*client).interval = (*(*client).config).initial_interval;
    send_discover(ip as *mut core::ffi::c_void);
}

// ---- state_release ----
#[no_mangle]
pub unsafe extern "C" fn state_release(ipp: *mut core::ffi::c_void) {
    let ip: *mut _interface_info = ipp as *mut _interface_info;
    let mut foo: ULONG = GetTickCount();
    make_release(ip, (*(*ip).client).active);
    (*(*ip).client).xid = RtlRandom(core::ptr::addr_of_mut!(foo));
    send_release(ip as *mut core::ffi::c_void);
    (*(*ip).client).state = S_RELEASED;
    unbind_lease(ip as *mut interface_info);
}

// ---- packet_to_lease ----
unsafe fn packet_to_lease(packet: *mut packet) -> *mut client_lease {
    let lease: *mut client_lease = malloc(core::mem::size_of::<client_lease>()) as *mut client_lease;
    if lease.is_null() {
        warning(b"dhcpoffer: no memory to record lease.\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    memset(lease as *mut core::ffi::c_void, 0, core::mem::size_of::<client_lease>());

    for i in 0i32..256i32 {
        if (*packet).options[i as usize].len != 0 {
            let data: *mut u8 = malloc(((*packet).options[i as usize].len + 1) as usize) as *mut u8;
            if data.is_null() {
                warning(b"dhcpoffer: no memory for option %d\0".as_ptr() as *const i8, i);
                free_client_lease(lease);
                return core::ptr::null_mut();
            }
            memcpy(
                (data as *mut core::ffi::c_void) as *mut c_void,
                ((*packet).options[i as usize].data as *const core::ffi::c_void) as *const c_void,
                (*packet).options[i as usize].len as usize,
            );
            (*lease).options[i as usize].len = (*packet).options[i as usize].len;
            (*lease).options[i as usize].data = data;
            *((*lease).options[i as usize].data).add((*lease).options[i as usize].len as usize) = 0u8;
            if check_option(lease, i) == 0 {
                warning(b"Invalid lease option - ignoring offer\0".as_ptr() as *const i8);
                free_client_lease(lease);
                return core::ptr::null_mut();
            }
        }
    }

    (*lease).address.len = core::mem::size_of::<in_addr>() as _;
    memcpy(
        (core::ptr::addr_of_mut!((*lease).address.iabuf) as *mut core::ffi::c_void) as *mut c_void,
        (core::ptr::addr_of!((*(*packet).raw).yiaddr) as *const core::ffi::c_void) as *const c_void,
        (*lease).address.len as usize,
    );

    // __REACTOS__ block
    if (*(*packet).raw).siaddr.S_un.S_addr == 0 {
        (*lease).serveraddress.len = (*packet).client_addr.len;
        memcpy(
            ((*lease).serveraddress.iabuf.as_mut_ptr() as *mut core::ffi::c_void) as *mut c_void,
            ((*packet).client_addr.iabuf.as_ptr() as *const core::ffi::c_void) as *const c_void,
            (*packet).client_addr.len as usize,
        );
    } else {
        (*lease).serveraddress.len = core::mem::size_of::<in_addr>() as _;
        memcpy(
            (core::ptr::addr_of_mut!((*lease).serveraddress.iabuf) as *mut core::ffi::c_void) as *mut c_void,
            (core::ptr::addr_of!((*(*packet).raw).siaddr) as *const core::ffi::c_void) as *const c_void,
            (*lease).serveraddress.len as usize,
        );
    }

    let overload_len = (*packet).options[DHO_DHCP_OPTION_OVERLOAD as usize].len;
    let overload_data = (*packet).options[DHO_DHCP_OPTION_OVERLOAD as usize].data;
    if (overload_len == 0 || ((*overload_data) & 2u8) == 0) && (*(*packet).raw).sname[0] != 0 {
        (*lease).server_name = malloc((DHCP_SNAME_LEN + 1) as usize) as *mut i8;
        if (*lease).server_name.is_null() {
            warning(b"dhcpoffer: no memory for server name.\0".as_ptr() as *const i8);
            free_client_lease(lease);
            return core::ptr::null_mut();
        }
        memcpy(
            ((*lease).server_name as *mut core::ffi::c_void) as *mut c_void,
            ((*(*packet).raw).sname.as_ptr() as *const core::ffi::c_void) as *const c_void,
            DHCP_SNAME_LEN as usize,
        );
        *((*lease).server_name).add(DHCP_SNAME_LEN as usize) = 0i8;
        if res_hnok((*lease).server_name) == 0 {
            warning(b"Bogus server name %s\0".as_ptr() as *const i8, (*lease).server_name);
            free_client_lease(lease);
            return core::ptr::null_mut();
        }
    }

    if (overload_len == 0 || ((*overload_data) & 1u8) == 0) && (*(*packet).raw).file[0] != 0 {
        (*lease).filename = malloc((DHCP_FILE_LEN + 1) as usize) as *mut i8;
        if (*lease).filename.is_null() {
            warning(b"dhcpoffer: no memory for filename.\0".as_ptr() as *const i8);
            free_client_lease(lease);
            return core::ptr::null_mut();
        }
        memcpy(
            ((*lease).filename as *mut core::ffi::c_void) as *mut c_void,
            ((*(*packet).raw).file.as_ptr() as *const core::ffi::c_void) as *const c_void,
            DHCP_FILE_LEN as usize,
        );
        *((*lease).filename).add(DHCP_FILE_LEN as usize) = 0i8;
    }

    lease
}

// ---- dhcpnak ----
unsafe extern "C" fn dhcpnak(packet: *mut packet) {
    let ip = (*packet).interface;
    let client = (*ip).client;
    let raw = (*packet).raw;

    // Check transaction ID, hardware address length, and hardware address
    if ((*client).xid != (*raw).xid) ||
       ((*ip).hw_address.hlen != (*raw).hlen) ||
       (memcmp(
           &(*ip).hw_address.haddr as *const _ as *const core::ffi::c_void,
           &(*raw).chaddr as *const _ as *const core::ffi::c_void,
           (*raw).hlen as usize,
       ) != 0)
    {
        return;
    }

    // Only handle certain states
    if ((*client).state != S_REBOOTING &&
        (*client).state != S_REQUESTING &&
        (*client).state != S_RENEWING &&
        (*client).state != S_REBINDING)
    {
        return;
    }

    note(b"DHCPNAK from %s\0".as_ptr() as *const i8, piaddr((*packet).client_addr));

    if ((*client).active.is_null()) {
        note(b"DHCPNAK with no active lease.\n\0".as_ptr() as *const i8);
        return;
    }

    free_client_lease((*client).active);
    (*client).active = core::ptr::null_mut();

    cancel_timeout(send_request, ip as *mut core::ffi::c_void);

    (*client).state = S_INIT;
    state_init(ip as *mut core::ffi::c_void);
}

// ---- send_request ----
pub unsafe extern "C" fn send_request(ipp: *mut core::ffi::c_void) {
    let ip: *mut interface_info = ipp as *mut interface_info;
    let mut destination: sockaddr_in = core::mem::zeroed();
    let mut from: in_addr = core::mem::zeroed();
    let mut interval: i32;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;

    time(&mut cur_time as *mut time_t);

    interval = (cur_time - (*(*ip).client).first_sending) as i32;

    if (((*(*ip).client).state == S_REBOOTING || (*(*ip).client).state == S_REQUESTING) &&
        interval > ((*(*(*ip).client).config).reboot_timeout) as i32)
    {
        (*(*ip).client).state = S_INIT;
        cancel_timeout(send_request, ip as *mut core::ffi::c_void);
        state_init(ip as *mut core::ffi::c_void);
        return;
    }

    if ((*(*ip).client).state == S_REBOOTING &&
        (*(*ip).client).medium.is_null() &&
        !((*(*(*ip).client).active).medium).is_null())
    {
        (*(*ip).client).medium = (*(*(*ip).client).active).medium;
    }

    if ((*(*ip).client).state != S_REQUESTING &&
        cur_time > (*(*(*ip).client).active).expiry)
    {
        let Adapter: *mut DHCP_ADAPTER = AdapterFindInfo((ip as *mut core::ffi::c_void) as *mut interface_info);
        if !Adapter.is_null() {
            DeleteIPAddress((*Adapter).NteContext);
            (*Adapter).NteContext = 0;
        }
        (*(*ip).client).state = S_INIT;
        state_init(ip as *mut core::ffi::c_void);
        return;
    }

    if (*(*ip).client).interval == 0 {
        (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;
    } else {
        (*(*ip).client).interval += ((rand() >> 2) as time_t) % (2 * (*(*ip).client).interval);
    }

    if (*(*ip).client).interval > (*(*(*ip).client).config).backoff_cutoff {
        (*(*ip).client).interval = ((*(*(*ip).client).config).backoff_cutoff / 2)
            + (((rand() >> 2) as time_t) % (*(*ip).client).interval);
    }

    if (*(*ip).client).state != S_REQUESTING &&
        cur_time + (*(*ip).client).interval > (*(*(*ip).client).active).expiry
    {
        (*(*ip).client).interval = (*(*(*ip).client).active).expiry - cur_time + 1;
    }

    memset((&mut destination as *mut sockaddr_in as *mut i8) as *mut c_void, 0i32, core::mem::size_of::<sockaddr_in>());
    if (*(*ip).client).state == S_REQUESTING || (*(*ip).client).state == S_REBOOTING || cur_time > (*(*(*ip).client).active).rebind {
        destination.sin_addr.S_un.S_addr = 0xFFFFFFFFu32; // INADDR_BROADCAST
    } else {
        memcpy(
            (&mut destination.sin_addr.S_un.S_addr as *mut u32 as *mut i8) as *mut c_void,
            ((*(*ip).client).destination.iabuf.as_ptr() as *const i8) as *const c_void,
            core::mem::size_of::<u32>(),
        );
    }
    destination.sin_port = htons((REMOTE_PORT as u16));
    destination.sin_family = 2; // AF_INET

    if (*(*ip).client).state != S_REQUESTING {
        memcpy(
            ((&mut from as *mut in_addr as *mut i8) as *mut c_void) as *mut c_void,
            (((*(*(*ip).client).active).address.iabuf.as_ptr() as *const i8) as *const c_void) as *const c_void,
            core::mem::size_of::<in_addr>(),
        );
    } else {
        from.S_un.S_addr = 0u32; // INADDR_ANY
    }

    if (*(*ip).client).state == S_REQUESTING {
        (*(*ip).client).packet.secs = (*(*ip).client).secs;
    } else {
        if interval < 65536 {
            (*(*ip).client).packet.secs = htons(interval as u16);
        } else {
            (*(*ip).client).packet.secs = htons(65535u16);
        }
    }

    note(
        b"DHCPREQUEST on %s to %s port %d\0".as_ptr() as *const i8,
        (*ip).name,
        inet_ntoa(destination.sin_addr),
        ntohs(destination.sin_port) as u32,
    );

    send_packet(
        ip,
        core::ptr::addr_of_mut!((*(*ip).client).packet),
        ((*(*ip).client).packet_length) as usize,
        from,
        core::ptr::addr_of_mut!(destination),
        core::ptr::null_mut(),
    );

    add_timeout(
        (cur_time + (*(*ip).client).interval) as time_t,
        send_request,
        ip as *mut core::ffi::c_void,
    );
}

// ---- dhcpack ----
unsafe extern "C" fn dhcpack(packet: *mut packet) {
    let ip: *mut interface_info = (*packet).interface;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    time(core::ptr::addr_of_mut!(cur_time));

    if (*(*ip).client).xid != (*(*packet).raw).xid
        || (*ip).hw_address.hlen != (*(*packet).raw).hlen
        || memcmp(
            ((*ip).hw_address.haddr.as_ptr() as *const u8) as *const c_void,
            ((*(*packet).raw).chaddr.as_ptr() as *const u8) as *const c_void,
            (*(*packet).raw).hlen as usize,
        ) != 0
    {
        return;
    }

    let state = (*(*ip).client).state;
    if state != S_REBOOTING
        && state != S_REQUESTING
        && state != S_RENEWING
        && state != S_REBINDING
    {
        return;
    }

    note(
        b"DHCPACK from %s\0".as_ptr() as *const i8,
        piaddr((*packet).client_addr),
    );

    let lease: *mut client_lease = packet_to_lease(packet);
    if lease.is_null() {
        note(b"packet_to_lease failed.\0".as_ptr() as *const i8);
        return;
    }

    (*(*ip).client).new = lease;

    cancel_timeout(send_request, ip as *mut core::ffi::c_void);

    if !(*(*(*ip).client).new).options[DHO_DHCP_LEASE_TIME as usize].data.is_null() {
        (*(*(*ip).client).new).expiry = (getULong(
            (*(*(*ip).client).new).options[DHO_DHCP_LEASE_TIME as usize].data,
        )) as time_t;
    } else {
        (*(*(*ip).client).new).expiry = (DHCP_DEFAULT_LEASE_TIME) as time_t;
    }

    if (*(*(*ip).client).new).expiry < 0 as time_t {
        (*(*(*ip).client).new).expiry = (TIME_MAX) as time_t;
    }

    if (*(*(*ip).client).new).expiry < 60 as time_t {
        (*(*(*ip).client).new).expiry = 60 as time_t;
    }

    if (*(*(*ip).client).new).options[DHO_DHCP_RENEWAL_TIME as usize].len != 0 {
        (*(*(*ip).client).new).renewal = (getULong(
            (*(*(*ip).client).new).options[DHO_DHCP_RENEWAL_TIME as usize].data,
        )) as time_t;
    } else {
        (*(*(*ip).client).new).renewal = (*(*(*ip).client).new).expiry / 2;
    }

    if (*(*(*ip).client).new).options[DHO_DHCP_REBINDING_TIME as usize].len != 0 {
        (*(*(*ip).client).new).rebind = (getULong(
            (*(*(*ip).client).new).options[DHO_DHCP_REBINDING_TIME as usize].data,
        )) as time_t;
    } else {
        let renewal = (*(*(*ip).client).new).renewal;
        (*(*(*ip).client).new).rebind = renewal + renewal / 2 + renewal / 4;
    }

    // __REACTOS__ block
    (*(*(*ip).client).new).lease = ((*(*(*ip).client).new).expiry) as u32;
    (*(*(*ip).client).new).obtained = cur_time;

    (*(*(*ip).client).new).expiry += cur_time;
    if (*(*(*ip).client).new).expiry < cur_time {
        (*(*(*ip).client).new).expiry = (TIME_MAX) as time_t;
    }

    (*(*(*ip).client).new).renewal += cur_time;
    if (*(*(*ip).client).new).renewal < cur_time {
        (*(*(*ip).client).new).renewal = (TIME_MAX) as time_t;
    }

    (*(*(*ip).client).new).rebind += cur_time;
    if (*(*(*ip).client).new).rebind < cur_time {
        (*(*(*ip).client).new).rebind = (TIME_MAX) as time_t;
    }

    bind_lease(ip);
}

// ---- state_reboot ----
pub unsafe extern "C" fn state_reboot(ipp: *mut core::ffi::c_void) {
    let ip: *mut interface_info = ipp as *mut _;
    let mut foo: u32 = GetTickCount() as u32;

    // If we don't remember an active lease, go straight to INIT.
    if (*(*ip).client).active.is_null() || (*(*(*ip).client).active).is_bootp() != 0 {
        state_init(ip as *mut core::ffi::c_void);
        return;
    }

    // We are in the rebooting state.
    (*(*ip).client).state = S_REBOOTING;

    // make_request doesn't initialize xid because it normally comes
    // from the DHCPDISCOVER, but we haven't sent a DHCPDISCOVER,
    // so pick an xid now.
    (*(*ip).client).xid = RtlRandom(core::ptr::addr_of_mut!(foo));

    // Make a DHCPREQUEST packet, and set appropriate per-interface flags.
    make_request(ip, (*(*ip).client).active);

    (*(*ip).client).destination = iaddr_broadcast;
    time(core::ptr::addr_of_mut!((*(*ip).client).first_sending));
    (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;

    // Zap the medium list...
    (*(*ip).client).medium = core::ptr::null_mut();

    // Send out the first DHCPREQUEST packet.
    send_request(ip as *mut core::ffi::c_void);
}

// ---- state_selecting ----
unsafe extern "C" fn state_selecting(ipp: *mut core::ffi::c_void) {
    let ip = ipp as *mut interface_info;
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    time(core::ptr::addr_of_mut!(cur_time));
    cancel_timeout(state_selecting, ip as *mut core::ffi::c_void);
    cancel_timeout(send_discover, ip as *mut core::ffi::c_void);
    let mut picked: *mut client_lease = core::ptr::null_mut();
    let mut lp = (*(*ip).client).offered_leases;
    while !lp.is_null() {
        let next = (*lp).next;
        if picked.is_null() {
            if check_arp(ip, lp) == 0 {
                free_client_lease(lp);
            } else {
                picked = lp;
                (*picked).next = core::ptr::null_mut();
            }
        } else {
            free_client_lease(lp);
        }
        lp = next;
    }
    (*(*ip).client).offered_leases = core::ptr::null_mut();
    if picked.is_null() {
        (*(*ip).client).state = S_INIT;
        state_init(ip as *mut core::ffi::c_void);
        return;
    }
    if (*picked).options[DHO_DHCP_MESSAGE_TYPE as usize].len == 0 {
        (*(*ip).client).new = picked;
        (*(*(*ip).client).new).expiry = cur_time + 12000;
        (*(*(*ip).client).new).renewal += cur_time + 8000;
        (*(*(*ip).client).new).rebind += cur_time + 10000;
        (*(*ip).client).state = S_REQUESTING;
        bind_lease(ip);
        return;
    }
    (*(*ip).client).destination = iaddr_broadcast;
    (*(*ip).client).state = S_REQUESTING;
    (*(*ip).client).first_sending = cur_time;
    (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;
    make_request(ip, picked);
    (*(*ip).client).xid = (*(*ip).client).packet.xid;
    free_client_lease(picked);
    send_request(ip as *mut core::ffi::c_void);
}

// ---- state_bound ----
unsafe extern "C" fn state_bound(ipp: *mut core::ffi::c_void) {
    let ip: *mut interface_info = ipp as *mut interface_info;

    // ASSERT_STATE(state, S_BOUND) is a no-op

    /* T1 has expired. */
    make_request(ip, (*(*ip).client).active);
    (*(*ip).client).xid = (*(*ip).client).packet.xid;

    if (*(*(*ip).client).active).options[DHO_DHCP_SERVER_IDENTIFIER as usize].len == 4 {
        let src = (*(*(*ip).client).active).options[DHO_DHCP_SERVER_IDENTIFIER as usize].data;
        let dst = (*(*ip).client).destination.iabuf.as_mut_ptr();
        core::ptr::copy_nonoverlapping(src, dst, 4);
        (*(*ip).client).destination.len = 4;
    } else {
        (*(*ip).client).destination = iaddr_broadcast;
    }

    time(&mut (*(*ip).client).first_sending as *mut _ as *mut time_t);
    (*(*ip).client).interval = (*(*(*ip).client).config).initial_interval;
    (*(*ip).client).state = S_RENEWING;

    /* Send the first packet immediately. */
    send_request(ip as *mut core::ffi::c_void);
}

// ---- dhcpoffer ----
unsafe extern "C" fn dhcpoffer(packet: *mut _packet) {
    let ip: *mut _interface_info = (*packet).interface;
    let mut lease: *mut _client_lease;
    let mut lp: *mut _client_lease;
    let mut i: i32;
    let mut arp_timeout_needed: i32 = 0;
    let mut stop_selecting: i32;
    let name: *const i8 = if (*packet).options[DHO_DHCP_MESSAGE_TYPE as usize].len != 0 {
        b"DHCPOFFER\0".as_ptr() as *const i8
    } else {
        b"BOOTREPLY\0".as_ptr() as *const i8
    };
    // C declares `time_t cur_time;` as a LOCAL in each of these functions — one per
    // activation. dhcpd.h's `extern time_t cur_time` is VESTIGIAL ISC baggage: no TU in
    // this DLL defines it (bindgen emits the decl, the linker finds no symbol). Making it
    // a shared global would also alias the value across the dispatch loop's callbacks.
    let mut cur_time: time_t = 0;
    time(core::ptr::addr_of_mut!(cur_time));

    if ((*(*ip).client).state != S_SELECTING) || ((*(*(*packet).interface).client).xid != (*(*packet).raw).xid) || ((*(*packet).interface).hw_address.hlen != (*(*packet).raw).hlen) || (memcmp((*(*packet).interface).hw_address.haddr.as_ptr() as *const core::ffi::c_void, (*(*packet).raw).chaddr.as_ptr() as *const core::ffi::c_void, (*(*packet).raw).hlen as usize) != 0) {
        return;
    }

    note(b"%s from %s\0".as_ptr() as *const i8, name, piaddr((*packet).client_addr));

    i = 0;
    while (*(*(*ip).client).config).required_options[i as usize] != 0 {
        if (*packet).options[(*(*(*ip).client).config).required_options[i as usize] as usize].len == 0 {
            note(b"%s isn't satisfactory.\0".as_ptr() as *const i8, name);
            return;
        }
        i += 1;
    }

    lease = (*(*ip).client).offered_leases;
    while !lease.is_null() {
        if ((*lease).address.len == (core::mem::size_of_val(&(*(*packet).raw).yiaddr)) as i32) && (memcmp((*lease).address.iabuf.as_ptr() as *const core::ffi::c_void, &(*(*packet).raw).yiaddr as *const _ as *const core::ffi::c_void, (*lease).address.len as usize) == 0) {
            debug(b"%s already seen.\0".as_ptr() as *const i8, name);
            return;
        }
        lease = (*lease).next;
    }

    lease = packet_to_lease(packet);
    if lease.is_null() {
        note(b"packet_to_lease failed.\0".as_ptr() as *const i8);
        return;
    }

    if (*packet).options[DHO_DHCP_MESSAGE_TYPE as usize].len == 0 {
        (*lease).set_is_bootp(1);
    }

    (*lease).medium = (*(*ip).client).medium;

    if check_arp(ip, lease) == 0 {
        note(b"Arp check failed\n\0".as_ptr() as *const i8);
        return;
    }

    stop_selecting = ((*(*ip).client).first_sending + (*(*(*ip).client).config).select_interval) as i32;

    if (*lease).address.len == (*(*ip).client).requested_address.len && memcmp((*lease).address.iabuf.as_ptr() as *const core::ffi::c_void, (*(*ip).client).requested_address.iabuf.as_ptr() as *const core::ffi::c_void, (*(*ip).client).requested_address.len as usize) == 0 {
        (*lease).next = (*(*ip).client).offered_leases;
        (*(*ip).client).offered_leases = lease;
    } else {
        if !(*(*ip).client).offered_leases.is_null() && (cur_time + arp_timeout_needed as time_t) > stop_selecting as time_t {
            arp_timeout_needed = 0;
        }

        (*lease).next = core::ptr::null_mut();
        if (*(*ip).client).offered_leases.is_null() {
            (*(*ip).client).offered_leases = lease;
        } else {
            lp = (*(*ip).client).offered_leases;
            while !(*lp).next.is_null() {
                lp = (*lp).next;
            }
            (*lp).next = lease;
        }
    }

    if (stop_selecting as i64) - (cur_time as i64) < (arp_timeout_needed as i64) {
        stop_selecting = (cur_time as i64 + arp_timeout_needed as i64) as i32;
    }

    if stop_selecting <= 0 {
        state_selecting(ip as *mut core::ffi::c_void);
    } else {
        add_timeout((stop_selecting) as time_t, state_selecting, ip as *mut core::ffi::c_void);
        cancel_timeout(send_discover, ip as *mut core::ffi::c_void);
    }
}

// ---- bootp ----
pub unsafe extern "C" fn bootp(packet: *mut _packet) {
    if (*(*packet).raw).op != (BOOTREPLY) as u8 {
        return;
    }

    let mut ap: *mut _iaddrlist = (*(*(*(*packet).interface).client).config).reject_list;
    while !ap.is_null() {
        if addr_eq((*packet).client_addr, (*ap).addr) != 0 {
            note(
                b"BOOTREPLY from %s rejected.\0".as_ptr() as *const i8,
                piaddr((*ap).addr) as *const i8,
            );
            return;
        }
        ap = (*ap).next;
    }

    dhcpoffer(packet);
}

// ---- rewrite_client_leases ----
unsafe fn rewrite_client_leases(ifi: *mut interface_info) {
    let mut lp: *mut client_lease;
    if leaseFile.is_null() {
        leaseFile = fopen((path_dhclient_db) as *const i8, b"w\0".as_ptr() as *const i8);
        if leaseFile.is_null() {
            error(b"can't create %s\0".as_ptr() as *const i8, path_dhclient_db);
        }
    } else {
        fflush(leaseFile);
        rewind(leaseFile);
    }
    lp = (*(*ifi).client).leases;
    while !lp.is_null() {
        write_client_lease(ifi, lp, 1);
        lp = (*lp).next;
    }
    if !(*(*ifi).client).active.is_null() {
        write_client_lease(ifi, (*(*ifi).client).active, 1);
    }
    fflush(leaseFile);
}

// ---- write_client_lease ----
unsafe fn write_client_lease(ip: *mut interface_info, lease: *mut client_lease, rewrite: i32) {
    static LEASES_WRITTEN: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);

    if rewrite == 0 {
        let old = LEASES_WRITTEN.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if old > 20 {
            rewrite_client_leases(ip);
            LEASES_WRITTEN.store(0, core::sync::atomic::Ordering::Relaxed);
        }
    }

    if (*lease).is_static() != 0 {
        return;
    }

    if leaseFile.is_null() {
        leaseFile = fopen((path_dhclient_db) as *const i8, b"w\0".as_ptr() as *const i8);
        if leaseFile.is_null() {
            error(b"can't create %s\0".as_ptr() as *const i8, path_dhclient_db);
            return;
        }
    }

    fprintf(leaseFile, b"lease {\n\0".as_ptr() as *const i8);
    if (*lease).is_bootp() != 0 {
        fprintf(leaseFile, b"  bootp;\n\0".as_ptr() as *const i8);
    }
    fprintf(
        leaseFile,
        b"  interface \"%s\";\n\0".as_ptr() as *const i8,
        (*ip).name
    );
    fprintf(
        leaseFile,
        b"  fixed-address %s;\n\0".as_ptr() as *const i8,
        piaddr((*lease).address)
    );
    if !(*lease).filename.is_null() {
        fprintf(
            leaseFile,
            b"  filename \"%s\";\n\0".as_ptr() as *const i8,
            (*lease).filename
        );
    }
    if !(*lease).server_name.is_null() {
        fprintf(
            leaseFile,
            b"  server-name \"%s\";\n\0".as_ptr() as *const i8,
            (*lease).server_name
        );
    }
    if !(*lease).medium.is_null() {
        fprintf(
            leaseFile,
            b"  medium \"%s\";\n\0".as_ptr() as *const i8,
            (*(*lease).medium).string
        );
    }
    for i in 0..256 {
        if (*lease).options[i as usize].len != 0 {
            fprintf(
                leaseFile,
                b"  option %s %s;\n\0".as_ptr() as *const i8,
                dhcp_options[i as usize].name,
                pretty_print_option(
                    i,
                    (*lease).options[i as usize].data,
                    (*lease).options[i as usize].len,
                    1,
                    1
                )
            );
        }
    }

    let t = gmtime(core::ptr::addr_of_mut!((*lease).renewal));
    if !t.is_null() {
        fprintf(
            leaseFile,
            b"  renew %d %d/%d/%d %02d:%02d:%02d;\n\0".as_ptr() as *const i8,
            (*t).tm_wday,
            (*t).tm_year + 1900,
            (*t).tm_mon + 1,
            (*t).tm_mday,
            (*t).tm_hour,
            (*t).tm_min,
            (*t).tm_sec
        );
    }

    let t = gmtime(core::ptr::addr_of_mut!((*lease).rebind));
    if !t.is_null() {
        fprintf(
            leaseFile,
            b"  rebind %d %d/%d/%d %02d:%02d:%02d;\n\0".as_ptr() as *const i8,
            (*t).tm_wday,
            (*t).tm_year + 1900,
            (*t).tm_mon + 1,
            (*t).tm_mday,
            (*t).tm_hour,
            (*t).tm_min,
            (*t).tm_sec
        );
    }

    let t = gmtime(core::ptr::addr_of_mut!((*lease).expiry));
    if !t.is_null() {
        fprintf(
            leaseFile,
            b"  expire %d %d/%d/%d %02d:%02d:%02d;\n\0".as_ptr() as *const i8,
            (*t).tm_wday,
            (*t).tm_year + 1900,
            (*t).tm_mon + 1,
            (*t).tm_mday,
            (*t).tm_hour,
            (*t).tm_min,
            (*t).tm_sec
        );
    }

    fprintf(leaseFile, b"}\n\0".as_ptr() as *const i8);
    fflush(leaseFile);
}
