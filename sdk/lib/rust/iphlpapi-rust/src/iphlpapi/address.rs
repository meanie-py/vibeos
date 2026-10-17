use crate::prelude::*;
use core::ffi::c_void;

// L"EnableDHCP"
static REG_ENABLE_DHCP: [u16; 11] = [
    b'E' as u16, b'n' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b'D' as u16,
    b'H' as u16, b'C' as u16, b'P' as u16, 0,
];
// L"DhcpDomain"
static REG_DHCP_DOMAIN: [u16; 11] = [
    b'D' as u16, b'h' as u16, b'c' as u16, b'p' as u16, b'D' as u16, b'o' as u16, b'm' as u16,
    b'a' as u16, b'i' as u16, b'n' as u16, 0,
];
// L"Domain"
static REG_DOMAIN: [u16; 7] = [
    b'D' as u16, b'o' as u16, b'm' as u16, b'a' as u16, b'i' as u16, b'n' as u16, 0,
];
// L"DeviceDesc"
static REG_DEVICE_DESC: [u16; 11] = [
    b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'D' as u16,
    b'e' as u16, b's' as u16, b'c' as u16, 0,
];
// L"Name"
static REG_NAME: [u16; 5] = [
    b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0,
];

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn GetAdaptersAddresses(
    family: u32,
    flags: u32,
    reserved: *mut core::ffi::c_void,
    adapter_addresses: PIP_ADAPTER_ADDRESSES,
    out_buf_len: *mut u32,
) -> u32 {
    // Initialize locals
    let mut status: NTSTATUS;
    let mut tcp_file: HANDLE = core::ptr::null_mut();
    let mut interfaces_list: *mut TDIEntityID = core::ptr::null_mut();
    let mut interfaces_count: u32 = 0;
    let mut adapters_count: u32 = 0;
    let mut total_size: u32 = 0;
    let mut remaining_size: u32;
    let min_size: u32 = 15 * 1024;
    let mut ptr: *mut u8 = adapter_addresses as *mut u8;
    let mut previous_aa: PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();

    TRACE!(
        b"Family %u, Flags 0x%08x, Reserved %p, pAdapterAddress %p, pOutBufLen %p\n\0"
            .as_ptr() as *const i8,
        family,
        flags,
        reserved,
        adapter_addresses,
        out_buf_len,
    );

    if out_buf_len.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if ptr.is_null() || *out_buf_len < min_size {
        *out_buf_len = min_size;
        return ERROR_BUFFER_OVERFLOW;
    }

    let family = match family {
        AF_INET => AF_INET,
        AF_INET6 => {
            FIXME!("IPv6 is not supported in ReactOS!\n");
            return ERROR_NO_DATA;
        }
        AF_UNSPEC => {
            WARN!(b"IPv6 addresses ignored, IPv4 only\n\0".as_ptr() as *const i8);
            AF_INET
        }
        _ => {
            ERR!(
                "Invalid family 0x%x\n",
                family,
            );
            return ERROR_INVALID_PARAMETER;
        }
    };

    remaining_size = *out_buf_len;
    if !ptr.is_null() {
        core::ptr::write_bytes(ptr, 0, remaining_size as usize);
    }

    status = openTcpFile(&mut tcp_file, FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        ERR!(
            "Could not open handle to tcpip.sys. Status %08x\n",
            status,
        );
        return RtlNtStatusToDosError(status);
    }

    status = GetInterfacesList(tcp_file, &mut interfaces_list, &mut interfaces_count);
    if !NT_SUCCESS(status) {
        ERR!(
            "Could not get adapters list. Status %08x\n",
            status,
        );
        NtClose(tcp_file);
        return RtlNtStatusToDosError(status);
    }

    // C's `goto Error` is a FORWARD jump to a cleanup tail, so the tail must sit AFTER the
    // labeled block and the block must ENCLOSE every jump site. (The LLM emitted the tail as a
    // labeled block placed after all four `break 'error` sites, where the label is unreachable.)
    // The jumps carry no value: `status` is the same local the tail reads, exactly as in C.
    'error: {
    // Main loop over interfaces
    for i in 0..interfaces_count {
        if (*interfaces_list.add(i as usize)).tei_entity == IF_ENTITY {
            // Same C EntryBuffer idiom as GetInterfaceNameInfo below (address.c:415); the
            // zero-length [IFEntry; 0] member carries the alignment the cast needs.
            const ENTRY_BUFFER_SIZE: usize = core::mem::offset_of!(IFEntry, if_descr)
                + core::mem::size_of::<u8>() * ((MAX_ADAPTER_DESCRIPTION_LENGTH + 1) as usize);
            #[repr(C)]
            struct EntryBuf {
                _align: [IFEntry; 0],
                bytes: [u8; ENTRY_BUFFER_SIZE],
            }
            let mut entry_buffer = EntryBuf {
                _align: [],
                bytes: [0u8; ENTRY_BUFFER_SIZE],
            };
            let entry: *mut IFEntry = entry_buffer.bytes.as_mut_ptr() as *mut IFEntry;

            adapters_count += 1;

            if !previous_aa.is_null() {
                (*previous_aa).Next = ptr as PIP_ADAPTER_ADDRESSES;
            }

            // DIVERGES from address.c, which lays out the XP-sized IP_ADAPTER_ADDRESSES. Vista+
            // callers (Wine's wbemprox among them) read the LH fields past it -- the gateway
            // list at FirstGatewayAddress was another entry's bytes and crashed the walk. Each
            // entry is the LH struct, its new fields zero (the buffer is cleared above) except
            // Luid; Length says LH, as Windows does.
            let mut current_aa_size: u32 = core::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
            let mut friendly_size: u32 = 0;
            let mut description_size: u32 = 0;
            let mut dhcp_domain_size: u32 = 0;
            let mut domain_size: u32 = 0;

            status = GetInterfaceEntry(tcp_file, core::ptr::read(interfaces_list.add(i as usize)), entry);
            if !NT_SUCCESS(status) {
                break 'error;
            }

            TRACE!(
                b"Got entity %*s, index %u.\n\0".as_ptr() as *const i8,
                (*entry).if_descrlen,
                (*entry).if_descr.as_ptr(),
                (*entry).if_index,
            );

            current_aa_size += (*entry).if_descrlen + 1; // sizeof(CHAR) = 1

            if (*entry).if_type != IF_TYPE_SOFTWARE_LOOPBACK {
                let mut interface_key: HKEY = core::ptr::null_mut();
                let mut key_name: [i8; 256] = [0; 256];
                let mut value_type: u32 = 0;
                let mut value_size: u32;
                let mut data: u32 = 0;

                let _ = sprintf(
                    key_name.as_mut_ptr(),
                    b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s\0"
                        .as_ptr() as *const i8,
                    (*entry).if_descrlen,
                    (*entry).if_descr.as_ptr(),
                );

                if RegOpenKeyExA(
                    HKEY_LOCAL_MACHINE,
                    key_name.as_ptr() as PCSTR,
                    0,
                    KEY_READ,
                    &mut interface_key,
                ) == (ERROR_SUCCESS as i32)
                {
                    value_size = core::mem::size_of::<u32>() as u32;
                    if RegQueryValueExW(
                        interface_key,
                        REG_ENABLE_DHCP.as_ptr() as PCWSTR,
                        core::ptr::null_mut(),
                        &mut value_type,
                        &mut data as *mut u32 as LPBYTE,
                        &mut value_size,
                    ) == (ERROR_SUCCESS as i32)
                        && value_type == REG_DWORD
                        && data == 1
                    {
                        value_size = core::mem::size_of::<u32>() as u32;
                        if RegQueryValueExW(
                            interface_key,
                            REG_DHCP_DOMAIN.as_ptr() as PCWSTR,
                            core::ptr::null_mut(),
                            &mut value_type,
                            core::ptr::null_mut(),
                            &mut value_size,
                        ) == (ERROR_SUCCESS as i32)
                            && value_type == REG_SZ
                        {
                            dhcp_domain_size = value_size - core::mem::size_of::<u16>() as u32;
                            current_aa_size += dhcp_domain_size;
                        }
                    } else {
                        value_size = core::mem::size_of::<u32>() as u32;
                        if RegQueryValueExW(
                            interface_key,
                            REG_DOMAIN.as_ptr() as PCWSTR,
                            core::ptr::null_mut(),
                            &mut value_type,
                            core::ptr::null_mut(),
                            &mut value_size,
                        ) == (ERROR_SUCCESS as i32)
                            && value_type == REG_SZ
                        {
                            domain_size = value_size - core::mem::size_of::<u16>() as u32;
                            current_aa_size += domain_size;
                        }
                    }
                    RegCloseKey(interface_key);
                }
            }

            current_aa_size += core::mem::size_of::<u16>() as u32;

            if (*entry).if_type == IF_TYPE_SOFTWARE_LOOPBACK {
                description_size = mbstowcs(
                    core::ptr::null_mut(),
                    (*entry).if_descr.as_ptr() as PCSTR,
                    (*entry).if_descrlen as usize,
                ) as u32 * core::mem::size_of::<u16>() as u32;
                current_aa_size += description_size;
            } else {
                let mut connection_key: HKEY = core::ptr::null_mut();
                let mut key_name: [i8; 256] = [0; 256];
                let mut instance_id: [i8; 128] = [0; 128];
                let mut value_type: u32 = 0;
                let mut value_size: u32 = 0;

                let _ = sprintf(
                    key_name.as_mut_ptr(),
                    b"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection\0"
                        .as_ptr() as *const i8,
                    (*entry).if_descrlen,
                    (*entry).if_descr.as_ptr(),
                );

                if RegOpenKeyExA(
                    HKEY_LOCAL_MACHINE,
                    key_name.as_ptr() as PCSTR,
                    0,
                    KEY_READ,
                    &mut connection_key,
                ) == (ERROR_SUCCESS as i32)
                {
                    value_size = 128;
                    RegQueryValueExA(
                        connection_key,
                        b"PnpInstanceID\0".as_ptr() as PCSTR,
                        core::ptr::null_mut(),
                        &mut value_type,
                        instance_id.as_mut_ptr() as LPBYTE,
                        &mut value_size,
                    );
                    RegCloseKey(connection_key);
                }

                if value_size != 0 {
                    let _ = sprintf(
                        key_name.as_mut_ptr(),
                        b"SYSTEM\\CurrentControlSet\\Enum\\%s\0".as_ptr() as *const i8,
                        instance_id.as_ptr(),
                    );

                    if RegOpenKeyExA(
                        HKEY_LOCAL_MACHINE,
                        key_name.as_ptr() as PCSTR,
                        0,
                        KEY_READ,
                        &mut connection_key,
                    ) == (ERROR_SUCCESS as i32)
                    {
                        value_size = 0;
                        if RegQueryValueExW(
                            connection_key,
                            REG_DEVICE_DESC.as_ptr() as PCWSTR,
                            core::ptr::null_mut(),
                            &mut value_type,
                            core::ptr::null_mut(),
                            &mut value_size,
                        ) == (ERROR_SUCCESS as i32)
                            && value_type == REG_SZ
                        {
                            description_size = value_size - core::mem::size_of::<u16>() as u32;
                            current_aa_size += description_size;
                        }
                        RegCloseKey(connection_key);
                    }
                }
            }
            current_aa_size += core::mem::size_of::<u16>() as u32;

            if flags & GAA_FLAG_SKIP_FRIENDLY_NAME == 0 {
                if (*entry).if_type == IF_TYPE_SOFTWARE_LOOPBACK {
                    friendly_size = mbstowcs(
                        core::ptr::null_mut(),
                        (*entry).if_descr.as_ptr() as PCSTR,
                        (*entry).if_descrlen as usize,
                    ) as u32 * core::mem::size_of::<u16>() as u32;
                    current_aa_size += friendly_size;
                } else {
                    let mut connection_key: HKEY = core::ptr::null_mut();
                    let mut key_name: [i8; 256] = [0; 256];

                    let _ = sprintf(
                        key_name.as_mut_ptr(),
                        b"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection\0"
                            .as_ptr() as *const i8,
                        (*entry).if_descrlen,
                        (*entry).if_descr.as_ptr(),
                    );

                    if RegOpenKeyExA(
                        HKEY_LOCAL_MACHINE,
                        key_name.as_ptr() as PCSTR,
                        0,
                        KEY_READ,
                        &mut connection_key,
                    ) == (ERROR_SUCCESS as i32)
                    {
                        let mut value_type: u32 = 0;
                        let mut value_size: u32 = 0;

                        if RegQueryValueExW(
                            connection_key,
                            REG_NAME.as_ptr() as PCWSTR,
                            core::ptr::null_mut(),
                            &mut value_type,
                            core::ptr::null_mut(),
                            &mut value_size,
                        ) == (ERROR_SUCCESS as i32)
                            && value_type == REG_SZ
                        {
                            friendly_size = value_size - core::mem::size_of::<u16>() as u32;
                            current_aa_size += friendly_size;
                        }
                        RegCloseKey(connection_key);
                    }
                }
                current_aa_size += core::mem::size_of::<u16>() as u32;
            }

            if flags & GAA_FLAG_SKIP_DNS_SERVER == 0 {
                let mut interface_key: HKEY = core::ptr::null_mut();
                let mut key_name: [i8; 256] = [0; 256];

                let _ = sprintf(
                    key_name.as_mut_ptr(),
                    b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s\0"
                        .as_ptr() as *const i8,
                    (*entry).if_descrlen,
                    (*entry).if_descr.as_ptr(),
                );

                if RegOpenKeyExA(
                    HKEY_LOCAL_MACHINE,
                    key_name.as_ptr() as PCSTR,
                    0,
                    KEY_READ,
                    &mut interface_key,
                ) == (ERROR_SUCCESS as i32)
                {
                    EnumNameServers(
                        interface_key,
                        core::ptr::null_mut(),
                        &mut current_aa_size as *mut _ as PVOID,
                        Some(EnumerateServerNameSize),
                    );
                    RegCloseKey(interface_key);
                }
            }

            total_size += current_aa_size;

            if !ptr.is_null() && remaining_size >= current_aa_size {
                let current_aa: PIP_ADAPTER_ADDRESSES = ptr as PIP_ADAPTER_ADDRESSES;
                (*current_aa).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
                    core::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
                (*current_aa).__bindgen_anon_1.__bindgen_anon_1.as_mut().IfIndex = (*entry).if_index;
                *(core::ptr::addr_of_mut!((*(current_aa as PIP_ADAPTER_ADDRESSES_LH)).Luid) as *mut u64) =
                    crate::iphlpapi::luid::make_luid((*entry).if_type, (*entry).if_index);
                core::ptr::copy_nonoverlapping(
                    (*entry).if_physaddr.as_ptr(),
                    (*current_aa).PhysicalAddress.as_mut_ptr(),
                    (*entry).if_physaddrlen as usize,
                );
                (*current_aa).PhysicalAddressLength = (*entry).if_physaddrlen;
                QueryFlags(
                    ((*entry).if_descr.as_ptr() as *mut u8),
                    (*entry).if_descrlen,
                    &mut (*current_aa).Flags,
                );
                (*current_aa).Mtu = (*entry).if_mtu;
                (*current_aa).IfType = (*entry).if_type;
                if (*entry).if_operstatus >= IF_OPER_STATUS_CONNECTING {
                    (*current_aa).OperStatus = IfOperStatusUp;
                } else {
                    (*current_aa).OperStatus = IfOperStatusDown;
                }

                ptr = (current_aa as PIP_ADAPTER_ADDRESSES_LH).add(1) as *mut u8; // past the LH fields, not the XP struct
                core::ptr::copy_nonoverlapping(
                    (*entry).if_descr.as_ptr(),
                    ptr,
                    (*entry).if_descrlen as usize,
                );
                (*current_aa).AdapterName = ptr as PCHAR;
                *((*current_aa).AdapterName.add((*entry).if_descrlen as usize)) = (b'\0' as i8);
                ptr = (*current_aa).AdapterName.add((*entry).if_descrlen as usize + 1) as *mut u8;

                (*current_aa).DnsSuffix = ptr as PWCHAR;
                if (*entry).if_type != IF_TYPE_SOFTWARE_LOOPBACK {
                    let mut interface_key: HKEY = core::ptr::null_mut();
                    let mut key_name: [i8; 256] = [0; 256];
                    let mut value_type: u32 = 0;
                    let mut value_size: u32;

                    let _ = sprintf(
                        key_name.as_mut_ptr(),
                        b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s\0"
                            .as_ptr() as *const i8,
                        (*entry).if_descrlen,
                        (*entry).if_descr.as_ptr(),
                    );

                    if RegOpenKeyExA(
                        HKEY_LOCAL_MACHINE,
                        key_name.as_ptr() as PCSTR,
                        0,
                        KEY_READ,
                        &mut interface_key,
                    ) == (ERROR_SUCCESS as i32)
                    {
                        if dhcp_domain_size != 0 {
                            value_size = dhcp_domain_size + core::mem::size_of::<u16>() as u32;
                            if RegQueryValueExW(
                                interface_key,
                                REG_DHCP_DOMAIN.as_ptr() as PCWSTR,
                                core::ptr::null_mut(),
                                &mut value_type,
                                (*current_aa).DnsSuffix as LPBYTE,
                                &mut value_size,
                            ) == (ERROR_SUCCESS as i32)
                                && value_type == REG_SZ
                                && value_size == dhcp_domain_size + core::mem::size_of::<u16>() as u32
                            {
                                ptr = ((*current_aa).DnsSuffix).add(
                                    (value_size / core::mem::size_of::<u16>() as u32) as usize,
                                ) as *mut u8;
                            } else {
                                ERR!(
                                    "DhcpDomain name changed after probe!\n",
                                );
                                dhcp_domain_size = 0;
                            }
                        }

                        if domain_size != 0 {
                            value_size = domain_size + core::mem::size_of::<u16>() as u32;
                            if RegQueryValueExW(
                                interface_key,
                                REG_DOMAIN.as_ptr() as PCWSTR,
                                core::ptr::null_mut(),
                                &mut value_type,
                                (*current_aa).DnsSuffix as LPBYTE,
                                &mut value_size,
                            ) == (ERROR_SUCCESS as i32)
                                && value_type == REG_SZ
                                && value_size == domain_size + core::mem::size_of::<u16>() as u32
                            {
                                ptr = ((*current_aa).DnsSuffix).add(
                                    (value_size / core::mem::size_of::<u16>() as u32) as usize,
                                ) as *mut u8;
                            } else {
                                ERR!(
                                    "Domain name changed after probe!\n",
                                );
                                domain_size = 0;
                            }
                        }
                        RegCloseKey(interface_key);
                    }
                }
                if dhcp_domain_size == 0 && domain_size == 0 {
                    *((*current_aa).DnsSuffix as *mut u16) = 0;
                    ptr = ((*current_aa).DnsSuffix).add(1) as *mut u8;
                }

                (*current_aa).Description = ptr as PWCHAR;
                if description_size != 0 {
                    if (*entry).if_type == IF_TYPE_SOFTWARE_LOOPBACK {
                        let size = mbstowcs(
                            ptr as PWCHAR,
                            (*entry).if_descr.as_ptr() as PCSTR,
                            (*entry).if_descrlen as usize,
                        );
                        *(*current_aa).Description.add(size as usize) = 0;
                        ptr = ((*current_aa).Description).add(size as usize + 1) as *mut u8;
                    } else {
                        let mut connection_key: HKEY = core::ptr::null_mut();
                        let mut key_name: [i8; 256] = [0; 256];
                        let mut instance_id: [i8; 128] = [0; 128];
                        let mut value_type: u32 = 0;
                        let mut value_size: u32 = 0;

                        let _ = sprintf(
                            key_name.as_mut_ptr(),
                            b"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection\0"
                                .as_ptr() as *const i8,
                            (*entry).if_descrlen,
                            (*entry).if_descr.as_ptr(),
                        );

                        if RegOpenKeyExA(
                            HKEY_LOCAL_MACHINE,
                            key_name.as_ptr() as PCSTR,
                            0,
                            KEY_READ,
                            &mut connection_key,
                        ) == (ERROR_SUCCESS as i32)
                        {
                            value_size = 128;
                            RegQueryValueExA(
                                connection_key,
                                b"PnpInstanceID\0".as_ptr() as PCSTR,
                                core::ptr::null_mut(),
                                &mut value_type,
                                instance_id.as_mut_ptr() as LPBYTE,
                                &mut value_size,
                            );
                            RegCloseKey(connection_key);
                        }

                        if value_size != 0 {
                            let _ = sprintf(
                                key_name.as_mut_ptr(),
                                b"SYSTEM\\CurrentControlSet\\Enum\\%s\0".as_ptr() as *const i8,
                                instance_id.as_ptr(),
                            );

                            if RegOpenKeyExA(
                                HKEY_LOCAL_MACHINE,
                                key_name.as_ptr() as PCSTR,
                                0,
                                KEY_READ,
                                &mut connection_key,
                            ) == (ERROR_SUCCESS as i32)
                            {
                                value_size = description_size + core::mem::size_of::<u16>() as u32;
                                if RegQueryValueExW(
                                    connection_key,
                                    REG_DEVICE_DESC.as_ptr() as PCWSTR,
                                    core::ptr::null_mut(),
                                    &mut value_type,
                                    (*current_aa).Description as LPBYTE,
                                    &mut value_size,
                                ) == (ERROR_SUCCESS as i32)
                                    && value_type == REG_SZ
                                    && value_size
                                        == description_size + core::mem::size_of::<u16>() as u32
                                {
                                    ptr = ((*current_aa).Description).add(
                                        (value_size / core::mem::size_of::<u16>() as u32) as usize,
                                    ) as *mut u8;
                                } else {
                                    ERR!(
                                        "Description name changed after probe!\n",
                                    );
                                    description_size = 0;
                                }
                                RegCloseKey(connection_key);
                            }
                        }
                    }
                }
                if description_size == 0 {
                    *((*current_aa).Description as *mut u16) = 0;
                    ptr = ((*current_aa).Description).add(1) as *mut u8;
                }

                if flags & GAA_FLAG_SKIP_FRIENDLY_NAME == 0 {
                    (*current_aa).FriendlyName = ptr as PWCHAR;
                    if friendly_size != 0 {
                        if (*entry).if_type == IF_TYPE_SOFTWARE_LOOPBACK {
                            let size = mbstowcs(
                                (*current_aa).FriendlyName,
                                (*entry).if_descr.as_ptr() as PCSTR,
                                (*entry).if_descrlen as usize,
                            );
                            *(*current_aa).FriendlyName.add(size as usize) = 0;
                            ptr = ((*current_aa).FriendlyName).add(size as usize + 1) as *mut u8;
                        } else {
                            let mut connection_key: HKEY = core::ptr::null_mut();
                            let mut key_name: [i8; 256] = [0; 256];

                            let _ = sprintf(
                                key_name.as_mut_ptr(),
                                b"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection\0"
                                    .as_ptr() as *const i8,
                                (*entry).if_descrlen,
                                (*entry).if_descr.as_ptr(),
                            );

                            if RegOpenKeyExA(
                                HKEY_LOCAL_MACHINE,
                                key_name.as_ptr() as PCSTR,
                                0,
                                KEY_READ,
                                &mut connection_key,
                            ) == (ERROR_SUCCESS as i32)
                            {
                                let mut value_type: u32 = 0;
                                let mut value_size = friendly_size + core::mem::size_of::<u16>() as u32;
                                if RegQueryValueExW(
                                    connection_key,
                                    REG_NAME.as_ptr() as PCWSTR,
                                    core::ptr::null_mut(),
                                    &mut value_type,
                                    (*current_aa).FriendlyName as LPBYTE,
                                    &mut value_size,
                                ) == (ERROR_SUCCESS as i32)
                                    && value_type == REG_SZ
                                    && value_size
                                        == friendly_size + core::mem::size_of::<u16>() as u32
                                {
                                    ptr = ((*current_aa).FriendlyName).add(
                                        (value_size / core::mem::size_of::<u16>() as u32) as usize,
                                    ) as *mut u8;
                                } else {
                                    ERR!(
                                        "Friendly name changed after probe!\n",
                                    );
                                    friendly_size = 0;
                                }
                                RegCloseKey(connection_key);
                            } else {
                                friendly_size = 0;
                            }
                        }
                    }
                    if friendly_size == 0 {
                        *((*current_aa).FriendlyName as *mut u16) = 0;
                        ptr = ((*current_aa).FriendlyName).add(1) as *mut u8;
                    }
                }

                if flags & GAA_FLAG_SKIP_DNS_SERVER == 0 {
                    let mut interface_key: HKEY = core::ptr::null_mut();
                    let mut key_name: [i8; 256] = [0; 256];

                    let _ = sprintf(
                        key_name.as_mut_ptr(),
                        b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s\0"
                            .as_ptr() as *const i8,
                        (*entry).if_descrlen,
                        (*entry).if_descr.as_ptr(),
                    );

                    if RegOpenKeyExA(
                        HKEY_LOCAL_MACHINE,
                        key_name.as_ptr() as PCSTR,
                        0,
                        KEY_READ,
                        &mut interface_key,
                    ) != (ERROR_SUCCESS as i32)
                    {
                        TRACE!(
                            b"Failed opening interface key for interface %*s\n\0".as_ptr()
                                as *const i8,
                            (*entry).if_descrlen,
                            (*entry).if_descr.as_ptr(),
                        );
                    } else {
                        let mut server_address_ptr: *mut PIP_ADAPTER_DNS_SERVER_ADDRESS;
                        (*current_aa).FirstDnsServerAddress =
                            ptr as PIP_ADAPTER_DNS_SERVER_ADDRESS;
                        server_address_ptr =
                            &mut (*current_aa).FirstDnsServerAddress as *mut PIP_ADAPTER_DNS_SERVER_ADDRESS;
                        EnumNameServers(
                            interface_key,
                            core::ptr::null_mut(),
                            &mut server_address_ptr as *mut _ as PVOID,
                            Some(EnumerateServerName),
                        );
                        RegCloseKey(interface_key);
                        ptr = *server_address_ptr as *mut u8;
                        *server_address_ptr = core::ptr::null_mut();
                    }
                }

                previous_aa = current_aa;
                remaining_size -= current_aa_size;
            }
        }
    }

    if adapters_count == 0 {
        ERR!("No Adapters found!\n");
        *out_buf_len = 0;
        return ERROR_NO_DATA;
    }

    // Check skip unicast and include prefix flags
    if (flags & (GAA_FLAG_SKIP_UNICAST | GAA_FLAG_INCLUDE_PREFIX)) == GAA_FLAG_SKIP_UNICAST {
        // goto Success
        // Execute success cleanup
        NtClose(tcp_file);
        HeapFree(GetProcessHeap(), 0, interfaces_list as *mut c_void);
        *out_buf_len = total_size;
        TRACE!(b"TotalSize: %x\n\0".as_ptr() as *const i8, *out_buf_len);
        return ERROR_SUCCESS;
    }

    // Address filling
    for i in 0..interfaces_count {
        let tei_entity = (*interfaces_list.add(i as usize)).tei_entity;
        if tei_entity == CL_NL_ENTITY || tei_entity == CO_NL_ENTITY {
            let mut snmp_info: IPSNMPInfo = core::mem::zeroed();
            let mut current_aa: PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();
            let mut addr_entries: *mut IPAddrEntry = core::ptr::null_mut();

            status = GetSnmpInfo(tcp_file, core::ptr::read(interfaces_list.add(i as usize)), &mut snmp_info);
            if !NT_SUCCESS(status) {
                break 'error;
            }

            if snmp_info.ipsi_numaddr == 0 {
                continue;
            }

            addr_entries = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                ((snmp_info.ipsi_numaddr as usize * core::mem::size_of::<IPAddrEntry>()) as SIZE_T),
            ) as *mut IPAddrEntry;
            if addr_entries.is_null() {
                status = STATUS_NO_MEMORY;
                break 'error;
            }
            status = GetAddrEntries(
                tcp_file,
                core::ptr::read(interfaces_list.add(i as usize)),
                addr_entries,
                snmp_info.ipsi_numaddr,
            );
            if !NT_SUCCESS(status) {
                HeapFree(GetProcessHeap(), 0, addr_entries as *mut c_void);
                break 'error;
            }

            for j in 0..snmp_info.ipsi_numaddr {
                let addr_entry = &*addr_entries.add(j as usize);
                if !adapter_addresses.is_null() {
                    current_aa = adapter_addresses;
                    loop {
                        if (*current_aa).__bindgen_anon_1.__bindgen_anon_1.as_ref().IfIndex == addr_entry.iae_index {
                            break;
                        }
                        current_aa = (*current_aa).Next;
                        if current_aa.is_null() {
                            ERR!(
                                "Got address for interface %u but no adapter was found for it.\n",
                                addr_entry.iae_index,
                            );
                            break;
                        }
                    }
                    if current_aa.is_null() {
                        continue;
                    }
                }

                TRACE!(
                    b"address is 0x%08x, mask is 0x%08x\n\0".as_ptr() as *const i8,
                    addr_entry.iae_addr,
                    addr_entry.iae_mask,
                );

                if flags & GAA_FLAG_SKIP_UNICAST == 0 {
                    let size = core::mem::size_of::<IP_ADAPTER_UNICAST_ADDRESS>() as u32
                        + core::mem::size_of::<SOCKADDR>() as u32;

                    if !ptr.is_null() && remaining_size >= size {
                        let unicast: PIP_ADAPTER_UNICAST_ADDRESS = ptr as *mut IP_ADAPTER_UNICAST_ADDRESS;
                        (*unicast).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
                            core::mem::size_of::<IP_ADAPTER_UNICAST_ADDRESS>() as u32;
                        (*unicast).Next = (*current_aa).FirstUnicastAddress;
                        (*unicast).__bindgen_anon_1.__bindgen_anon_1.as_mut().Flags = 0;
                        (*unicast).PrefixOrigin = IpPrefixOriginOther;
                        (*unicast).SuffixOrigin = IpSuffixOriginOther;
                        (*unicast).DadState = IpDadStatePreferred;
                        (*unicast).ValidLifetime = 0xFFFFFFFF;
                        (*unicast).PreferredLifetime = 0xFFFFFFFF;
                        // C: `(LPSOCKADDR)(UnicastAddress + 1)` — advances by ONE WHOLE
                        // IP_ADAPTER_UNICAST_ADDRESS (the sockaddr is suffixed to the struct),
                        // NOT by one byte. Keep the add() on the TYPED pointer.
                        (*unicast).Address.lpSockaddr = unicast.add(1) as LPSOCKADDR;
                        (*unicast).Address.iSockaddrLength = (core::mem::size_of::<SOCKADDR>() as u32 as i32);
                        // sa_family is set through the GENERIC sockaddr; sin_port/sin_addr
                        // through the LPSOCKADDR_IN cast (C lines 941-943).
                        (*(*unicast).Address.lpSockaddr).sa_family = (AF_INET as u16);
                        (*((*unicast).Address.lpSockaddr as *mut SOCKADDR_IN)).sin_port = 0;
                        (*((*unicast).Address.lpSockaddr as *mut SOCKADDR_IN))
                            .sin_addr
                            .S_un
                            .S_addr = addr_entry.iae_addr;
                        (*current_aa).FirstUnicastAddress = unicast;
                        ptr = ptr.add(size as usize);
                        remaining_size -= size;
                    }
                    total_size += size;
                }

                if flags & GAA_FLAG_INCLUDE_PREFIX != 0 {
                    let size = core::mem::size_of::<IP_ADAPTER_PREFIX>() as u32
                        + core::mem::size_of::<SOCKADDR>() as u32;

                    if !ptr.is_null() && remaining_size >= size {
                        let prefix: PIP_ADAPTER_PREFIX = ptr as *mut IP_ADAPTER_PREFIX;
                        (*prefix).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
                            core::mem::size_of::<IP_ADAPTER_PREFIX>() as u32;
                        (*prefix).Next = (*current_aa).FirstPrefix;
                        // C: `(LPSOCKADDR)(Prefix + 1)` — one WHOLE IP_ADAPTER_PREFIX.
                        (*prefix).Address.lpSockaddr = prefix.add(1) as LPSOCKADDR;
                        (*prefix).Address.iSockaddrLength = (core::mem::size_of::<SOCKADDR>() as u32 as i32);
                        (*(*prefix).Address.lpSockaddr).sa_family = (AF_INET as u16);
                        (*((*prefix).Address.lpSockaddr as *mut SOCKADDR_IN)).sin_port = 0;
                        (*((*prefix).Address.lpSockaddr as *mut SOCKADDR_IN))
                            .sin_addr
                            .S_un
                            .S_addr = addr_entry.iae_addr & addr_entry.iae_mask;
                        (*prefix).PrefixLength = CountPrefixBits(addr_entry.iae_mask);
                        (*current_aa).FirstPrefix = prefix;
                        ptr = ptr.add(size as usize);
                        remaining_size -= size;
                    }
                    total_size += size;
                }
            }
            HeapFree(GetProcessHeap(), 0, addr_entries as *mut c_void);
        }
    }

    // Success path (goto Success)
    NtClose(tcp_file);
    HeapFree(GetProcessHeap(), 0, interfaces_list as *mut c_void);
    *out_buf_len = total_size;
    TRACE!(b"TotalSize: %x\n\0".as_ptr() as *const i8, *out_buf_len);
    return ERROR_SUCCESS;
    } // end 'error block — falling out of it means a `break 'error` fired

    // Error path (the C `Error:` label)
    ERR!("Failed! Status 0x%08x\n", status);
    *out_buf_len = 0;
    if !interfaces_list.is_null() {
        HeapFree(GetProcessHeap(), 0, interfaces_list as *mut c_void);
    }
    NtClose(tcp_file);
    RtlNtStatusToDosError(status)
}
unsafe extern "C" fn GetInterfacesList(
    tcp_file: HANDLE,
    entity_list: *mut *mut TDIEntityID,
    interface_count: *mut ULONG,
) -> NTSTATUS {
    let mut tcp_query_info: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status: NTSTATUS;
    let buffer_size: ULONG_PTR;

    // ZeroMemory equivalent via zeroed()
    tcp_query_info.ID.toi_class = INFO_CLASS_GENERIC;
    tcp_query_info.ID.toi_type = INFO_TYPE_PROVIDER;
    tcp_query_info.ID.toi_id = ENTITY_LIST_ID;
    tcp_query_info.ID.toi_entity.tei_entity = GENERIC_ENTITY;
    tcp_query_info.ID.toi_entity.tei_instance = 0;

    status = NtDeviceIoControlFile(
        tcp_file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut status_block,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut tcp_query_info as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if status == STATUS_PENDING {
        status = NtWaitForSingleObject(tcp_file, 0, core::ptr::null_mut());
        if NT_SUCCESS(status) {
            status = status_block.__bindgen_anon_1.Status;
        }
    }

    if !NT_SUCCESS(status) {
        return status;
    }

    buffer_size = status_block.Information;
    *entity_list = HeapAlloc(GetProcessHeap(), 0, buffer_size) as *mut TDIEntityID;
    if (*entity_list).is_null() {
        return STATUS_NO_MEMORY;
    }

    status = NtDeviceIoControlFile(
        tcp_file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut status_block,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut tcp_query_info as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        *entity_list as *mut core::ffi::c_void,
        buffer_size as u32,
    );

    if status == STATUS_PENDING {
        status = NtWaitForSingleObject(tcp_file, 0, core::ptr::null_mut());
        if NT_SUCCESS(status) {
            status = status_block.__bindgen_anon_1.Status;
        }
    }

    if !NT_SUCCESS(status) {
        HeapFree(GetProcessHeap(), 0, *entity_list as *mut core::ffi::c_void);
        return status;
    }

    *interface_count = (buffer_size as usize / core::mem::size_of::<TDIEntityID>()) as ULONG;
    status
}
unsafe extern "C" fn GetSnmpInfo(TcpFile: HANDLE, InterfaceID: TDIEntityID, Info: *mut IPSNMPInfo) -> NTSTATUS {
    let mut TcpQueryInfo: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    core::ptr::write_bytes(
        &mut TcpQueryInfo as *mut TCP_REQUEST_QUERY_INFORMATION_EX as *mut u8,
        0,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
    );
    TcpQueryInfo.ID.toi_class = INFO_CLASS_PROTOCOL;
    TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
    TcpQueryInfo.ID.toi_id = IP_MIB_STATS_ID;
    TcpQueryInfo.ID.toi_entity = InterfaceID;

    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status = NtDeviceIoControlFile(
        TcpFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut StatusBlock,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut TcpQueryInfo as *mut _ as PVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        Info as PVOID,
        core::mem::size_of::<IPSNMPInfo>() as u32,
    );

    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(TcpFile, 0u8, core::ptr::null_mut());
        if NT_SUCCESS(Status) {
            Status = StatusBlock.__bindgen_anon_1.Status;
        }
    }

    Status
}
unsafe extern "C" fn GetAddrEntries(
    TcpFile: HANDLE,
    InterfaceID: TDIEntityID,
    Entries: *mut IPAddrEntry,
    NumEntries: u32,
) -> NTSTATUS {
    let mut TcpQueryInfo: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ZeroMemory(
        core::ptr::addr_of_mut!(TcpQueryInfo).cast(),
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
    );

    TcpQueryInfo.ID.toi_class = INFO_CLASS_PROTOCOL;
    TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
    TcpQueryInfo.ID.toi_id = IP_MIB_ADDRTABLE_ENTRY_ID;
    TcpQueryInfo.ID.toi_entity = InterfaceID;

    let mut Status = NtDeviceIoControlFile(
        TcpFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut StatusBlock,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut TcpQueryInfo as *mut _ as *mut c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        Entries as *mut c_void,
        NumEntries.wrapping_mul(core::mem::size_of::<IPAddrEntry>() as u32),
    );

    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(TcpFile, 0u8, core::ptr::null_mut());
        if NT_SUCCESS(Status) {
            Status = StatusBlock.__bindgen_anon_1.Status;
        }
    }

    Status
}
unsafe extern "C" fn GetInterfaceEntry(
    tcp_file: HANDLE,
    interface_id: TDIEntityID,
    entry: *mut IFEntry,
) -> NTSTATUS {
    let mut tcp_query_info: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status: NTSTATUS;

    ZeroMemory(
        &mut tcp_query_info as *mut _ as *mut _,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
    );
    tcp_query_info.ID.toi_class = INFO_CLASS_PROTOCOL;
    tcp_query_info.ID.toi_type = INFO_TYPE_PROVIDER;
    tcp_query_info.ID.toi_id = IP_MIB_STATS_ID;
    tcp_query_info.ID.toi_entity = interface_id;

    status = NtDeviceIoControlFile(
        tcp_file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut status_block,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut tcp_query_info as *mut _ as *mut _,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        entry as *mut _,
        (core::mem::offset_of!(IFEntry, if_descr)
            + (MAX_ADAPTER_DESCRIPTION_LENGTH as usize + 1)
                * core::mem::size_of::<u8>()) as u32,
    );

    if status == STATUS_PENDING {
        status = NtWaitForSingleObject(tcp_file, 0u8, core::ptr::null_mut());
        if NT_SUCCESS(status) {
            status = status_block.__bindgen_anon_1.Status;
        }
    }

    status
}
unsafe extern "C" fn EnumerateServerNameSize(
    Interface: *mut u16,
    NameServer: *mut u16,
    Data: *mut core::ffi::c_void,
) {
    // winnt.h `#define UNREFERENCED_PARAMETER(P) ((void)(P))` — a no-op discard.
    let _ = Interface;
    let _ = NameServer;

    let buffer_size = Data as *mut u32;
    *buffer_size += (core::mem::size_of::<IP_ADAPTER_DNS_SERVER_ADDRESS>()
        + core::mem::size_of::<SOCKADDR>()) as u32;
}
unsafe extern "C" fn EnumerateServerName(
    Interface: PWCHAR,
    NameServer: PWCHAR,
    Data: PVOID,
) {
    // C (address.c:239-240): `PIP_ADAPTER_DNS_SERVER_ADDRESS** Ptr = Data;` — a DOUBLE
    // pointer to the list-tail link, and `ServerAddress = **Ptr`. The translation dropped one
    // level of indirection, which would have read the tail pointer from the wrong place and
    // written the `*Ptr = &ServerAddress->Next` relink through it.
    let Ptr: *mut *mut *mut IP_ADAPTER_DNS_SERVER_ADDRESS =
        Data as *mut *mut *mut IP_ADAPTER_DNS_SERVER_ADDRESS;
    let ServerAddress: *mut IP_ADAPTER_DNS_SERVER_ADDRESS = **Ptr;
    let mut Terminator: *const u16 = core::ptr::null();

    // UNREFERENCED_PARAMETER
    let _ = Interface;

    (*ServerAddress).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
        core::mem::size_of::<IP_ADAPTER_DNS_SERVER_ADDRESS>() as _;
    (*ServerAddress).Address.lpSockaddr = ServerAddress.add(1) as *mut SOCKADDR;
    (*ServerAddress).Address.iSockaddrLength = core::mem::size_of::<SOCKADDR>() as _;

    // C sets sa_family through the GENERIC sockaddr (`lpSockaddr->sa_family`, address.c:250)
    // and sin_port/sin_addr through the `(LPSOCKADDR_IN)` cast (address.c:251,254). Those are
    // two different structs; SOCKADDR_IN has no `sa_family` member.
    (*(*ServerAddress).Address.lpSockaddr).sa_family = AF_INET as u16;
    let sockaddr_in = &mut *((*ServerAddress).Address.lpSockaddr as *mut SOCKADDR_IN);
    sockaddr_in.sin_port = 0;

    let Status = RtlIpv4StringToAddressW(
        NameServer,
        0u8, // FALSE
        core::ptr::addr_of_mut!(Terminator),
        core::ptr::addr_of_mut!(sockaddr_in.sin_addr) as *mut _,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    (*ServerAddress).Next = ((*ServerAddress).Address.lpSockaddr as *mut u8).add(core::mem::size_of::<SOCKADDR>()) as *mut IP_ADAPTER_DNS_SERVER_ADDRESS;
    *Ptr = core::ptr::addr_of_mut!((*ServerAddress).Next);
}
unsafe extern "C" fn QueryFlags(
    Interface: *mut u8,
    InterfaceLength: u32,
    Flags: *mut u32,
) {
    let mut InterfaceKey: HKEY = core::ptr::null_mut();
    let mut KeyName: [u8; 256] = [0u8; 256];
    let mut Type: u32 = 0;
    let mut Size: u32 = 0;
    let mut Data: u32 = 0;

    *Flags = 0;

    snprintf(
        KeyName.as_mut_ptr() as *mut i8,
        256,
        b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s\0".as_ptr() as *const i8,
        InterfaceLength as i32,
        Interface as *const i8,
    );

    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        KeyName.as_ptr() as *const i8,
        0,
        KEY_READ,
        &mut InterfaceKey as *mut HKEY,
    ) == (ERROR_SUCCESS as i32)
    {
        Size = core::mem::size_of::<u32>() as u32;
        if RegQueryValueExA(
            InterfaceKey,
            b"EnableDHCP\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            &mut Type as *mut u32,
            &mut Data as *mut u32 as *mut u8,
            &mut Size as *mut u32,
        ) == (ERROR_SUCCESS as i32)
            && Type == REG_DWORD
            && Data == 1
        {
            *Flags |= IP_ADAPTER_DHCP_ENABLED;
        }

        Size = core::mem::size_of::<u32>() as u32;
        if RegQueryValueExA(
            InterfaceKey,
            b"RegisterAdapterName\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            &mut Type as *mut u32,
            &mut Data as *mut u32 as *mut u8,
            &mut Size as *mut u32,
        ) == (ERROR_SUCCESS as i32)
            && Type == REG_DWORD
            && Data == 1
        {
            *Flags |= IP_ADAPTER_REGISTER_ADAPTER_SUFFIX;
        }

        Size = 0;
        if RegQueryValueExA(
            InterfaceKey,
            b"NameServer\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            &mut Type as *mut u32,
            &mut Data as *mut u32 as *mut u8,
            &mut Size as *mut u32,
        ) != (ERROR_SUCCESS as i32)
        {
            *Flags |= IP_ADAPTER_DDNS_ENABLED;
        }

        RegCloseKey(InterfaceKey);
    }
}
unsafe extern "C" fn CountPrefixBits(Netmask: u32) -> u32 {
    let mut BitCount: u32 = 0;

    for i in 0..(32) {
        if (Netmask & (1 << i)) == 0 {
            break;
        }
        BitCount += 1;
    }

    BitCount
}

#[no_mangle]
pub unsafe extern "system" fn NhpAllocateAndGetInterfaceInfoFromStack(
    pp_table: *mut *mut IP_INTERFACE_NAME_INFO,
    pdw_count: *mut u32,
    b_order: u8,
    h_heap: HANDLE,
    dw_flags: u32,
) -> u32 {
    let mut status: NTSTATUS = 0;
    let mut tcp_file: HANDLE = core::ptr::null_mut();
    let mut interfaces_list: *mut TDIEntityID = core::ptr::null_mut();
    let mut interfaces_count: u32 = 0;
    let mut adapters_count: u32 = 0;
    let mut p_table: *mut IP_INTERFACE_NAME_INFO = core::ptr::null_mut();
    let mut unicode_guid_string: _UNICODE_STRING = core::mem::zeroed();
    let mut ansi_guid_string: ANSI_STRING = core::mem::zeroed();

    // C (address.c:1048): `BYTE EntryBuffer[FIELD_OFFSET(IFEntry, if_descr) +
    //                       RTL_FIELD_SIZE(IFEntry, if_descr[0]) * (MAX_ADAPTER_DESCRIPTION_LENGTH + 1)]`
    // RTL_FIELD_SIZE(IFEntry, if_descr[0]) is the size of ONE element of the if_descr char
    // array = 1. The zero-length `[IFEntry; 0]` member costs no space but raises the buffer's
    // alignment to align_of::<IFEntry>(), which the `as *mut IFEntry` cast below requires
    // (C got away with a bare BYTE[] because it never had Rust's alignment UB rule; on arm64
    // an under-aligned ULONG load would fault).
    const BUFFER_SIZE: usize = core::mem::offset_of!(IFEntry, if_descr)
        + core::mem::size_of::<u8>() * ((MAX_ADAPTER_DESCRIPTION_LENGTH + 1) as usize);
    #[repr(C)]
    struct EntryBuffer {
        _align: [IFEntry; 0],
        bytes: [u8; BUFFER_SIZE],
    }
    let mut entry_buf = EntryBuffer {
        _align: [],
        bytes: [0u8; BUFFER_SIZE],
    };
    let entry: *mut IFEntry = entry_buf.bytes.as_mut_ptr() as *mut IFEntry;

    'error: {
        // open the tcpip driver
        status = openTcpFile(&mut tcp_file, FILE_READ_DATA);
        if !NT_SUCCESS(status) {
            return RtlNtStatusToDosError(status);
        }

        // Get the interfaces list
        status = GetInterfacesList(tcp_file, &mut interfaces_list, &mut interfaces_count);
        if !NT_SUCCESS(status) {
            NtClose(tcp_file);
            return RtlNtStatusToDosError(status);
        }

        // Count non-loopback adapters
        for i in 0..interfaces_count as usize {
            if (*interfaces_list.add(i)).tei_entity == IF_ENTITY {
                status = GetInterfaceEntry(tcp_file, core::ptr::read(interfaces_list.add(i)), entry);
                if !NT_SUCCESS(status) {
                    break 'error;
                }
                if (*entry).if_type != IF_TYPE_SOFTWARE_LOOPBACK {
                    adapters_count += 1;
                }
            }
        }

        // Allocate output table
        p_table = HeapAlloc(
            h_heap,
            dw_flags,
            ((adapters_count as usize * core::mem::size_of::<IP_INTERFACE_NAME_INFO>()) as SIZE_T),
        ) as *mut IP_INTERFACE_NAME_INFO;
        if p_table.is_null() {
            status = STATUS_NO_MEMORY;
            break 'error;
        }

        // Fill entries
        let mut j: usize = 0;
        for i in 0..interfaces_count as usize {
            if (*interfaces_list.add(i)).tei_entity == IF_ENTITY {
                status = GetInterfaceEntry(tcp_file, core::ptr::read(interfaces_list.add(i)), entry);
                if !NT_SUCCESS(status) {
                    break 'error;
                }
                if (*entry).if_type != IF_TYPE_SOFTWARE_LOOPBACK {
                    (*p_table.add(j)).Index = (*entry).if_index;
                    (*p_table.add(j)).MediaType = (*entry).if_type;
                    (*p_table.add(j)).ConnectionType = 1; // IF_CONNECTION_DEDICATED
                    (*p_table.add(j)).AccessType = 2; // IF_ACCESS_BROADCAST

                    ansi_guid_string.Length = ((*entry).if_descrlen as u16);
                    ansi_guid_string.MaximumLength = ((*entry).if_descrlen as u16);
                    ansi_guid_string.Buffer = ((*entry).if_descr.as_ptr() as *mut u8 as *mut i8);

                    status = RtlAnsiStringToUnicodeString(
                        &mut unicode_guid_string,
                        &mut ansi_guid_string,
                        1u8,
                    );
                    if !NT_SUCCESS(status) {
                        break 'error;
                    }

                    status = RtlGUIDFromString(
                        &mut unicode_guid_string,
                        &mut (*p_table.add(j)).DeviceGuid,
                    );
                    RtlFreeUnicodeString(&mut unicode_guid_string);
                    if !NT_SUCCESS(status) {
                        break 'error;
                    }

                    core::ptr::write_bytes(
                        core::ptr::addr_of_mut!((*p_table.add(j)).InterfaceGuid) as *mut u8,
                        0,
                        core::mem::size_of::<GUID>(),
                    );

                    j += 1;
                }
            }
        }

        // Success
        HeapFree(GetProcessHeap(), 0, interfaces_list as *mut core::ffi::c_void);
        NtClose(tcp_file);
        *pp_table = p_table;
        *pdw_count = adapters_count;
        return ERROR_SUCCESS;
    }

    // Error cleanup
    ERR!("Failed! Status 0x%08x\n", status);
    if !p_table.is_null() {
        HeapFree(h_heap, 0, p_table as *mut core::ffi::c_void);
    }
    *pp_table = core::ptr::null_mut();
    *pdw_count = 0;
    if !interfaces_list.is_null() {
        HeapFree(GetProcessHeap(), 0, interfaces_list as *mut core::ffi::c_void);
    }
    NtClose(tcp_file);
    RtlNtStatusToDosError(status)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/iphlpapi/address.c
//  * PURPOSE:         iphlpapi implementation - Adapter Address APIs
//  * PROGRAMMERS:     Jérôme Gardou (jerome.gardou@reactos.org)
//  */
//
// #include "iphlpapi_private.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi);
// #ifdef GetAdaptersAddressesV2
// /* Helper for GetAdaptersAddresses:
//  * Retrieves the list of network adapters from tcpip.sys */
// static
// NTSTATUS
// GetInterfacesList(
//      HANDLE TcpFile,
//      TDIEntityID **EntityList,
//      ULONG* InterfaceCount)
// {
//
//     TCP_REQUEST_QUERY_INFORMATION_EX TcpQueryInfo;
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//     ULONG_PTR BufferSize;
//
//     ZeroMemory(&TcpQueryInfo, sizeof(TcpQueryInfo));
//     TcpQueryInfo.ID.toi_class = INFO_CLASS_GENERIC;
//     TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
//     TcpQueryInfo.ID.toi_id = ENTITY_LIST_ID;
//     TcpQueryInfo.ID.toi_entity.tei_entity = GENERIC_ENTITY;
//     TcpQueryInfo.ID.toi_entity.tei_instance = 0;
//
//     Status = NtDeviceIoControlFile(
//         TcpFile,
//         NULL,
//         NULL,
//         NULL,
//         &StatusBlock,
//         IOCTL_TCP_QUERY_INFORMATION_EX,
//         &TcpQueryInfo,
//         sizeof(TcpQueryInfo),
//         NULL,
//         0);
//     if (Status == STATUS_PENDING)
//     {
//         /* So we have to wait a bit */
//         Status = NtWaitForSingleObject(TcpFile, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = StatusBlock.Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     BufferSize = StatusBlock.Information;
//     *EntityList = HeapAlloc(GetProcessHeap(), 0, BufferSize);
//     if (!*EntityList)
//         return STATUS_NO_MEMORY;
//
//     /* Do the real call */
//     Status = NtDeviceIoControlFile(
//         TcpFile,
//         NULL,
//         NULL,
//         NULL,
//         &StatusBlock,
//         IOCTL_TCP_QUERY_INFORMATION_EX,
//         &TcpQueryInfo,
//         sizeof(TcpQueryInfo),
//         *EntityList,
//         BufferSize);
//     if (Status == STATUS_PENDING)
//     {
//         /* So we have to wait a bit */
//         Status = NtWaitForSingleObject(TcpFile, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = StatusBlock.Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         HeapFree(GetProcessHeap(), 0, *EntityList);
//         return Status;
//     }
//
//     *InterfaceCount = BufferSize / sizeof(TDIEntityID);
//     return Status;
// }
//
// static
// NTSTATUS
// GetSnmpInfo(
//      HANDLE TcpFile,
//      TDIEntityID InterfaceID,
//      IPSNMPInfo* Info)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX TcpQueryInfo;
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//
//     ZeroMemory(&TcpQueryInfo, sizeof(TcpQueryInfo));
//     TcpQueryInfo.ID.toi_class = INFO_CLASS_PROTOCOL;
//     TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
//     TcpQueryInfo.ID.toi_id = IP_MIB_STATS_ID;
//     TcpQueryInfo.ID.toi_entity = InterfaceID;
//
//     Status = NtDeviceIoControlFile(
//         TcpFile,
//         NULL,
//         NULL,
//         NULL,
//         &StatusBlock,
//         IOCTL_TCP_QUERY_INFORMATION_EX,
//         &TcpQueryInfo,
//         sizeof(TcpQueryInfo),
//         Info,
//         sizeof(*Info));
//     if (Status == STATUS_PENDING)
//     {
//         /* So we have to wait a bit */
//         Status = NtWaitForSingleObject(TcpFile, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = StatusBlock.Status;
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// GetAddrEntries(
//      HANDLE TcpFile,
//      TDIEntityID InterfaceID,
//      IPAddrEntry* Entries,
//      ULONG NumEntries)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX TcpQueryInfo;
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//
//     ZeroMemory(&TcpQueryInfo, sizeof(TcpQueryInfo));
//     TcpQueryInfo.ID.toi_class = INFO_CLASS_PROTOCOL;
//     TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
//     TcpQueryInfo.ID.toi_id = IP_MIB_ADDRTABLE_ENTRY_ID;
//     TcpQueryInfo.ID.toi_entity = InterfaceID;
//
//     Status = NtDeviceIoControlFile(
//         TcpFile,
//         NULL,
//         NULL,
//         NULL,
//         &StatusBlock,
//         IOCTL_TCP_QUERY_INFORMATION_EX,
//         &TcpQueryInfo,
//         sizeof(TcpQueryInfo),
//         Entries,
//         NumEntries * sizeof(Entries[0]));
//     if (Status == STATUS_PENDING)
//     {
//         /* So we have to wait a bit */
//         Status = NtWaitForSingleObject(TcpFile, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = StatusBlock.Status;
//     }
//
//     return Status;
// }
//
// /*
//  * Fills the IFEntry buffer from tcpip.sys.
//  * The buffer size MUST be FIELD_OFFSET(IFEntry, if_descr[MAX_ADAPTER_DESCRIPTION_LENGTH + 1]).
//  * See MSDN IFEntry struct definition if you don't believe me. ;-)
//  */
// static
// NTSTATUS
// GetInterfaceEntry(
//      HANDLE TcpFile,
//      TDIEntityID InterfaceID,
//      IFEntry* Entry)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX TcpQueryInfo;
//     IO_STATUS_BLOCK StatusBlock;
//     NTSTATUS Status;
//
//     ZeroMemory(&TcpQueryInfo, sizeof(TcpQueryInfo));
//     TcpQueryInfo.ID.toi_class = INFO_CLASS_PROTOCOL;
//     TcpQueryInfo.ID.toi_type = INFO_TYPE_PROVIDER;
//     TcpQueryInfo.ID.toi_id = IP_MIB_STATS_ID;
//     TcpQueryInfo.ID.toi_entity = InterfaceID;
//
//     Status = NtDeviceIoControlFile(
//         TcpFile,
//         NULL,
//         NULL,
//         NULL,
//         &StatusBlock,
//         IOCTL_TCP_QUERY_INFORMATION_EX,
//         &TcpQueryInfo,
//         sizeof(TcpQueryInfo),
//         Entry,
//         FIELD_OFFSET(IFEntry, if_descr[MAX_ADAPTER_DESCRIPTION_LENGTH + 1]));
//     if (Status == STATUS_PENDING)
//     {
//         /* So we have to wait a bit */
//         Status = NtWaitForSingleObject(TcpFile, FALSE, NULL);
//         if (NT_SUCCESS(Status))
//             Status = StatusBlock.Status;
//     }
//
//     return Status;
// }
//
// /* Helpers to get the list of DNS for an interface */
// static
// VOID
// EnumerateServerNameSize(
//      PWCHAR Interface,
//      PWCHAR NameServer,
//      PVOID Data)
// {
//     ULONG* BufferSize = Data;
//
//     /* This is just sizing here */
//     UNREFERENCED_PARAMETER(Interface);
//     UNREFERENCED_PARAMETER(NameServer);
//
//     *BufferSize += sizeof(IP_ADAPTER_DNS_SERVER_ADDRESS) + sizeof(SOCKADDR);
// }
//
// static
// VOID
// EnumerateServerName(
//      PWCHAR Interface,
//      PWCHAR NameServer,
//      PVOID Data)
// {
//     PIP_ADAPTER_DNS_SERVER_ADDRESS** Ptr = Data;
//     PIP_ADAPTER_DNS_SERVER_ADDRESS ServerAddress = **Ptr;
//     PCWSTR Terminator;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(Interface);
//
//     ServerAddress->Length = sizeof(IP_ADAPTER_DNS_SERVER_ADDRESS);
//     ServerAddress->Address.lpSockaddr = (PVOID)(ServerAddress + 1);
//     ServerAddress->Address.iSockaddrLength = sizeof(SOCKADDR);
//
//     ServerAddress->Address.lpSockaddr->sa_family = AF_INET;
//     ((LPSOCKADDR_IN)ServerAddress->Address.lpSockaddr)->sin_port = 0;
//
//     Status = RtlIpv4StringToAddressW(NameServer, FALSE, &Terminator,
//                                      &((LPSOCKADDR_IN)ServerAddress->Address.lpSockaddr)->sin_addr);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Pass along, name conversion failed */
//         ERR!("%S is not a valid IP address\n", NameServer);
//         return;
//     }
//
//     /* Go to next item */
//     ServerAddress->Next = (PVOID)(ServerAddress->Address.lpSockaddr + 1);
//     *Ptr = &ServerAddress->Next;
// }
//
// static
// VOID
// QueryFlags(
//      PUCHAR Interface,
//      DWORD InterfaceLength,
//      LPDWORD Flags)
// {
//     HKEY InterfaceKey;
//     CHAR KeyName[256];
//     DWORD Type, Size, Data;
//
//     *Flags = 0;
//
//     snprintf(KeyName, 256,
//              "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s",
//              InterfaceLength, Interface);
//
//     if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &InterfaceKey) == ERROR_SUCCESS)
//     {
//         Size = sizeof(DWORD);
//         if (RegQueryValueExA(InterfaceKey, "EnableDHCP", NULL, &Type, (LPBYTE)&Data, &Size) == ERROR_SUCCESS &&
//             Type == REG_DWORD && Data == 1)
//         {
//             *Flags |= IP_ADAPTER_DHCP_ENABLED;
//         }
//
//         Size = sizeof(DWORD);
//         if (RegQueryValueExA(InterfaceKey, "RegisterAdapterName", NULL, &Type, (LPBYTE)&Data, &Size) == ERROR_SUCCESS &&
//             Type == REG_DWORD && Data == 1)
//         {
//             *Flags |= IP_ADAPTER_REGISTER_ADAPTER_SUFFIX;
//         }
//
//         Size = 0;
//         if (RegQueryValueExA(InterfaceKey, "NameServer", NULL, &Type, (LPBYTE)&Data, &Size) != ERROR_SUCCESS)
//         {
//             *Flags |= IP_ADAPTER_DDNS_ENABLED;
//         }
//
//         RegCloseKey(InterfaceKey);
//     }
//
//     // FIXME: handle 0x8 -> 0x20
// }
//
// static
// ULONG
// CountPrefixBits(
//      ULONG Netmask)
// {
//     ULONG i, BitCount = 0;
//
//     for (i = 0; i < sizeof(ULONG) * 8; i++)
//     {
//         if ((Netmask & (1 << i)) == 0)
//             break;
//         BitCount++;
//     }
//
//     return BitCount;
// }
//
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetAdaptersAddresses(
//      ULONG Family,
//      ULONG Flags,
//      PVOID Reserved,
//      PIP_ADAPTER_ADDRESSES pAdapterAddresses,
//      PULONG pOutBufLen)
// {
//     NTSTATUS Status;
//     HANDLE TcpFile;
//     TDIEntityID* InterfacesList;
//     ULONG InterfacesCount;
//     ULONG AdaptersCount = 0;
//     ULONG i;
//     ULONG TotalSize = 0, RemainingSize;
//     BYTE* Ptr = (BYTE*)pAdapterAddresses;
//     DWORD MIN_SIZE = 15 * 1024;
//     PIP_ADAPTER_ADDRESSES PreviousAA = NULL;
//
//     TRACE!("Family %u, Flags 0x%08x, Reserved %p, pAdapterAddress %p, pOutBufLen %p\n",
//           Family, Flags, Reserved, pAdapterAddresses, pOutBufLen);
//
//     if (!pOutBufLen)
//         return ERROR_INVALID_PARAMETER;
//
//     // FIXME: the exact needed size should be computed first, BEFORE doing any write to the output buffer.
//     // As suggested by MSDN, require a 15 KB buffer, which allows to React properly to length checks.
//     if(!Ptr || *pOutBufLen < MIN_SIZE)
//     {
//         *pOutBufLen = MIN_SIZE;
//         return ERROR_BUFFER_OVERFLOW;
//     }
//
//     switch(Family)
//     {
//         case AF_INET:
//             break;
//         case AF_INET6:
//             /* One day maybe... */
//             FIXME!("IPv6 is not supported in ReactOS!\n");
//             /* We got nothing to say in this case */
//             return ERROR_NO_DATA;
//             break;
//         case AF_UNSPEC:
//             WARN!("IPv6 addresses ignored, IPv4 only\n");
//             Family = AF_INET;
//             break;
//         default:
//             ERR!("Invalid family 0x%x\n", Family);
//             return ERROR_INVALID_PARAMETER;
//             break;
//     }
//
//     RemainingSize = *pOutBufLen;
//     if (Ptr)
//         ZeroMemory(Ptr, RemainingSize);
//
//     /* open the tcpip driver */
//     Status = openTcpFile(&TcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Could not open handle to tcpip.sys. Status %08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Get the interfaces list */
//     Status = GetInterfacesList(TcpFile, &InterfacesList, &InterfacesCount);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Could not get adapters list. Status %08x\n", Status);
//         NtClose(TcpFile);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Let's see if we got any adapter. */
//     for (i = 0; i < InterfacesCount; i++)
//     {
//         PIP_ADAPTER_ADDRESSES CurrentAA = (PIP_ADAPTER_ADDRESSES)Ptr;
//         ULONG CurrentAASize = 0;
//         ULONG FriendlySize = 0, DescriptionSize = 0;
//         ULONG DhcpDomainSize = 0, DomainSize = 0;
//
//         if (InterfacesList[i].tei_entity == IF_ENTITY)
//         {
//             BYTE EntryBuffer[FIELD_OFFSET(IFEntry, if_descr) +
//                              RTL_FIELD_SIZE(IFEntry, if_descr[0]) * (MAX_ADAPTER_DESCRIPTION_LENGTH + 1)];
//             IFEntry* Entry = (IFEntry*)EntryBuffer;
//
//             /* Remember we got one */
//             AdaptersCount++;
//
//             /* Set the pointer to this instance in the previous one*/
//             if(PreviousAA)
//                 PreviousAA->Next = CurrentAA;
//
//             /* Of course we need some space for the base structure. */
//             CurrentAASize = sizeof(IP_ADAPTER_ADDRESSES);
//
//             /* Get the entry */
//             Status = GetInterfaceEntry(TcpFile, InterfacesList[i], Entry);
//             if (!NT_SUCCESS(Status))
//                 goto Error;
//
//             TRACE!("Got entity %*s, index %u.\n",
//                 Entry->if_descrlen, &Entry->if_descr[0], Entry->if_index);
//
//             /* Add the adapter name */
//             CurrentAASize += Entry->if_descrlen + sizeof(CHAR);
//
//             /* Add the DNS suffix */
//             if (Entry->if_type != IF_TYPE_SOFTWARE_LOOPBACK)
//             {
//                 HKEY InterfaceKey;
//                 CHAR KeyName[256];
//                 DWORD ValueType, ValueSize, Data;
//
//                 snprintf(KeyName, 256,
//                     "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s",
//                     Entry->if_descrlen, &Entry->if_descr[0]);
//
//                 if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &InterfaceKey) == ERROR_SUCCESS)
//                 {
//                     ValueSize = sizeof(DWORD);
//                     if (RegQueryValueExW(InterfaceKey, L"EnableDHCP", NULL, &ValueType, (LPBYTE)&Data, &ValueSize) == ERROR_SUCCESS &&
//                         ValueType == REG_DWORD && Data == 1)
//                     {
//                         ValueSize = sizeof(DWORD);
//                         if (RegQueryValueExW(InterfaceKey, L"DhcpDomain", NULL, &ValueType, NULL, &ValueSize) == ERROR_SUCCESS &&
//                             ValueType == REG_SZ)
//                         {
//                             /* We remove the null char, it will be re-added after */
//                             DhcpDomainSize = ValueSize - sizeof(WCHAR);
//                             CurrentAASize += DhcpDomainSize;
//                         }
//                     }
//                     else
//                     {
//                         ValueSize = sizeof(DWORD);
//                         if (RegQueryValueExW(InterfaceKey, L"Domain", NULL, &ValueType, NULL, &ValueSize) == ERROR_SUCCESS &&
//                             ValueType == REG_SZ)
//                         {
//                             /* We remove the null char, it will be re-added after */
//                             DomainSize = ValueSize - sizeof(WCHAR);
//                             CurrentAASize += DomainSize;
//                         }
//                     }
//
//                     RegCloseKey(InterfaceKey);
//                 }
//             }
//
//             CurrentAASize += sizeof(WCHAR);
//
//             /* Add the description. */
//             if (Entry->if_type == IF_TYPE_SOFTWARE_LOOPBACK)
//             {
//                 DescriptionSize = mbstowcs(NULL, (PCSTR)&Entry->if_descr[0], Entry->if_descrlen) * sizeof(WCHAR);
//                 CurrentAASize += DescriptionSize;
//             }
//             else
//             {
//                 HKEY ConnectionKey;
//                 CHAR KeyName[256];
//                 CHAR InstanceID[128];
//                 DWORD ValueType = 0;
//                 DWORD ValueSize = 0;
//
//                 snprintf(KeyName, 256,
//                     "SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection",
//                     Entry->if_descrlen, &Entry->if_descr[0]);
//
//                 if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                 {
//                     ValueSize = 128;
//                     RegQueryValueExA(ConnectionKey, "PnpInstanceID", NULL, &ValueType, (LPBYTE)InstanceID, &ValueSize);
//                     RegCloseKey(ConnectionKey);
//                 }
//
//                 if (ValueSize != 0)
//                 {
//                     sprintf(KeyName, "SYSTEM\\CurrentControlSet\\Enum\\%s", InstanceID);
//
//                     if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                     {
//                         ValueSize = 0;
//                         if (RegQueryValueExW(ConnectionKey, L"DeviceDesc", NULL, &ValueType, NULL, &ValueSize) == ERROR_SUCCESS &&
//                             ValueType == REG_SZ)
//                         {
//                             /* We remove the null char, it will be re-added after */
//                             DescriptionSize = ValueSize - sizeof(WCHAR);
//                             CurrentAASize += DescriptionSize;
//                         }
//
//                         RegCloseKey(ConnectionKey);
//                     }
//                 }
//             }
//
//             /* We always make sure to have enough room for empty string */
//             CurrentAASize += sizeof(WCHAR);
//
//             if (!(Flags & GAA_FLAG_SKIP_FRIENDLY_NAME))
//             {
//                 if (Entry->if_type == IF_TYPE_SOFTWARE_LOOPBACK)
//                 {
//                     FriendlySize = mbstowcs(NULL, (PCSTR)&Entry->if_descr[0], Entry->if_descrlen) * sizeof(WCHAR);
//                     CurrentAASize += FriendlySize;
//                 }
//                 else
//                 {
//                     /* Get the friendly name */
//                     HKEY ConnectionKey;
//                     CHAR KeyName[256];
//
//                     snprintf(KeyName, 256,
//                         "SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection",
//                         Entry->if_descrlen, &Entry->if_descr[0]);
//
//                     if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                     {
//                         DWORD ValueType;
//                         DWORD ValueSize = 0;
//
//                         if (RegQueryValueExW(ConnectionKey, L"Name", NULL, &ValueType, NULL, &ValueSize) == ERROR_SUCCESS &&
//                             ValueType == REG_SZ)
//                         {
//                             /* We remove the null char, it will be re-added after */
//                             FriendlySize = ValueSize - sizeof(WCHAR);
//                             CurrentAASize += FriendlySize;
//                         }
//
//                         RegCloseKey(ConnectionKey);
//                     }
//                 }
//
//                 /* We always make sure to have enough room for empty string */
//                 CurrentAASize += sizeof(WCHAR);
//             }
//
//             if (!(Flags & GAA_FLAG_SKIP_DNS_SERVER))
//             {
//                 /* Enumerate the name servers */
//                 HKEY InterfaceKey;
//                 CHAR KeyName[256];
//
//                 snprintf(KeyName, 256,
//                     "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s",
//                     Entry->if_descrlen, &Entry->if_descr[0]);
//
//                 if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &InterfaceKey) == ERROR_SUCCESS)
//                 {
//                     EnumNameServers(InterfaceKey, NULL, &CurrentAASize, EnumerateServerNameSize);
//                     RegCloseKey(InterfaceKey);
//                 }
//             }
//
//             /* This is part of what we will need */
//             TotalSize += CurrentAASize;
//
//             /* Fill in the data */
//             if ((CurrentAA) && (RemainingSize >= CurrentAASize))
//             {
//                 CurrentAA->Length = sizeof(IP_ADAPTER_ADDRESSES);
//                 CurrentAA->IfIndex = Entry->if_index;
//                 CopyMemory(CurrentAA->PhysicalAddress, Entry->if_physaddr, Entry->if_physaddrlen);
//                 CurrentAA->PhysicalAddressLength = Entry->if_physaddrlen;
//                 QueryFlags(&Entry->if_descr[0], Entry->if_descrlen, &CurrentAA->Flags);
//                 CurrentAA->Mtu = Entry->if_mtu;
//                 CurrentAA->IfType = Entry->if_type;
//                 if(Entry->if_operstatus >= IF_OPER_STATUS_CONNECTING)
//                     CurrentAA->OperStatus = IfOperStatusUp;
//                 else
//                     CurrentAA->OperStatus = IfOperStatusDown;
//
//                 /* Next items */
//                 Ptr = (BYTE*)(CurrentAA + 1);
//
//                 /* Now fill in the name */
//                 CopyMemory(Ptr, &Entry->if_descr[0], Entry->if_descrlen);
//                 CurrentAA->AdapterName = (PCHAR)Ptr;
//                 CurrentAA->AdapterName[Entry->if_descrlen] = '\0';
//                 /* Next items */
//                 Ptr = (BYTE*)(CurrentAA->AdapterName + Entry->if_descrlen + 1);
//
//                 /* The DNS suffix */
//                 CurrentAA->DnsSuffix = (PWCHAR)Ptr;
//
//                 if (Entry->if_type != IF_TYPE_SOFTWARE_LOOPBACK)
//                 {
//                     HKEY InterfaceKey;
//                     CHAR KeyName[256];
//                     DWORD ValueType, ValueSize;
//
//                     snprintf(KeyName, 256,
//                         "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s",
//                         Entry->if_descrlen, &Entry->if_descr[0]);
//
//                     if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &InterfaceKey) == ERROR_SUCCESS)
//                     {
//                         if (DhcpDomainSize != 0)
//                         {
//                             ValueSize = DhcpDomainSize + sizeof(WCHAR);
//                             if (RegQueryValueExW(InterfaceKey, L"DhcpDomain", NULL, &ValueType, (LPBYTE)CurrentAA->DnsSuffix, &ValueSize) == ERROR_SUCCESS &&
//                                 ValueType == REG_SZ && ValueSize == DhcpDomainSize + sizeof(WCHAR))
//                             {
//                                 /* We're done, next items */
//                                 Ptr = (BYTE*)(CurrentAA->DnsSuffix + (ValueSize / sizeof(WCHAR)));
//                             }
//                             else
//                             {
//                                 /* Fail */
//                                 ERR!("DhcpDomain name changed after probe!\n");
//                                 DhcpDomainSize = 0;
//                             }
//                         }
//
//                         if (DomainSize != 0)
//                         {
//                             ValueSize = DomainSize + sizeof(WCHAR);
//                             if (RegQueryValueExW(InterfaceKey, L"Domain", NULL, &ValueType, (LPBYTE)CurrentAA->DnsSuffix, &ValueSize) == ERROR_SUCCESS &&
//                                 ValueType == REG_SZ && ValueSize == DomainSize + sizeof(WCHAR))
//                             {
//                                 /* We're done, next items */
//                                 Ptr = (BYTE*)(CurrentAA->DnsSuffix + (ValueSize / sizeof(WCHAR)));
//                             }
//                             else
//                             {
//                                 /* Fail */
//                                 ERR!("Domain name changed after probe!\n");
//                                 DomainSize = 0;
//                             }
//                         }
//
//                         RegCloseKey(InterfaceKey);
//                     }
//                 }
//
//                 /* In case of failure (or no name) */
//                 if ((DhcpDomainSize == 0) && (DomainSize == 0))
//                 {
//                     CurrentAA->DnsSuffix[0] = L'\0';
//                     /* Next items */
//                     Ptr = (BYTE*)(CurrentAA->DnsSuffix + 1);
//                 }
//
//                 /* The description */
//                 CurrentAA->Description = (PWCHAR)Ptr;
//
//                 if (DescriptionSize != 0)
//                 {
//                     if (Entry->if_type == IF_TYPE_SOFTWARE_LOOPBACK)
//                     {
//                         size_t size;
//                         size = mbstowcs((PWCHAR)Ptr, (PCSTR)&Entry->if_descr[0], Entry->if_descrlen);
//                         CurrentAA->Description = (PWCHAR)Ptr;
//                         CurrentAA->Description[size] = '\0';
//                         /* Next items */
//                         Ptr = (BYTE*)(CurrentAA->Description + size + 1);
//                     }
//                     else
//                     {
//                         HKEY ConnectionKey;
//                         CHAR KeyName[256];
//                         CHAR InstanceID[128];
//                         DWORD ValueType = 0;
//                         DWORD ValueSize = 0;
//
//                         snprintf(KeyName, 256,
//                             "SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection",
//                             Entry->if_descrlen, &Entry->if_descr[0]);
//
//                         if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                         {
//                             ValueSize = 128;
//                             RegQueryValueExA(ConnectionKey, "PnpInstanceID", NULL, &ValueType, (LPBYTE)InstanceID, &ValueSize);
//                             RegCloseKey(ConnectionKey);
//                         }
//
//                         if (ValueSize != 0)
//                         {
//                             sprintf(KeyName, "SYSTEM\\CurrentControlSet\\Enum\\%s", InstanceID);
//
//                             if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                             {
//                                 ValueSize = DescriptionSize + sizeof(WCHAR);
//                                 if (RegQueryValueExW(ConnectionKey, L"DeviceDesc", NULL, &ValueType, (LPBYTE)CurrentAA->Description, &ValueSize) == ERROR_SUCCESS &&
//                                     ValueType == REG_SZ && ValueSize == DescriptionSize + sizeof(WCHAR))
//                                 {
//                                     /* We're done, next items */
//                                     Ptr = (BYTE*)(CurrentAA->Description + (ValueSize / sizeof(WCHAR)));
//                                 }
//                                 else
//                                 {
//                                     /* Fail */
//                                     ERR!("Description name changed after probe!\n");
//                                     DescriptionSize = 0;
//                                 }
//
//                                 RegCloseKey(ConnectionKey);
//                             }
//                         }
//                     }
//                 }
//
//                 /* In case of failure (or no name) */
//                 if (DescriptionSize == 0)
//                 {
//                     CurrentAA->Description[0] = L'\0';
//                     /* Next items */
//                     Ptr = (BYTE*)(CurrentAA->Description + 1);
//                 }
//
//                 /* The friendly name */
//                 if (!(Flags & GAA_FLAG_SKIP_FRIENDLY_NAME))
//                 {
//                     CurrentAA->FriendlyName = (PWCHAR)Ptr;
//
//                     if (FriendlySize != 0)
//                     {
//                         /* Get the friendly name */
//                         if (Entry->if_type == IF_TYPE_SOFTWARE_LOOPBACK)
//                         {
//                             size_t size;
//                             size = mbstowcs(CurrentAA->FriendlyName, (PCSTR)&Entry->if_descr[0], Entry->if_descrlen);
//                             CurrentAA->FriendlyName[size] = '\0';
//                             /* Next items */
//                             Ptr = (BYTE*)(CurrentAA->FriendlyName + size + 1);
//                         }
//                         else
//                         {
//                             HKEY ConnectionKey;
//                             CHAR KeyName[256];
//
//                             snprintf(KeyName, 256,
//                                 "SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\%*s\\Connection",
//                                 Entry->if_descrlen, &Entry->if_descr[0]);
//
//                             if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &ConnectionKey) == ERROR_SUCCESS)
//                             {
//                                 DWORD ValueType;
//                                 DWORD ValueSize = FriendlySize + sizeof(WCHAR);
//
//                                 if (RegQueryValueExW(ConnectionKey, L"Name", NULL, &ValueType, (LPBYTE)CurrentAA->FriendlyName, &ValueSize) == ERROR_SUCCESS &&
//                                     ValueType == REG_SZ && ValueSize == FriendlySize + sizeof(WCHAR))
//                                 {
//                                     /* We're done, next items */
//                                     Ptr = (BYTE*)(CurrentAA->FriendlyName + (ValueSize / sizeof(WCHAR)));
//                                 }
//                                 else
//                                 {
//                                     /* Fail */
//                                     ERR!("Friendly name changed after probe!\n");
//                                     FriendlySize = 0;
//                                 }
//
//                                 RegCloseKey(ConnectionKey);
//                             }
//                             else
//                             {
//                                 /* Fail */
//                                 FriendlySize = 0;
//                             }
//                         }
//                     }
//
//                     /* In case of failure (or no name) */
//                     if (FriendlySize == 0)
//                     {
//                         CurrentAA->FriendlyName[0] = L'\0';
//                         /* Next items */
//                         Ptr = (BYTE*)(CurrentAA->FriendlyName + 1);
//                     }
//                 }
//
//                 /* The DNS Servers */
//                 if (!(Flags & GAA_FLAG_SKIP_DNS_SERVER))
//                 {
//                     /* Enumerate the name servers */
//                     HKEY InterfaceKey;
//                     CHAR KeyName[256];
//
//                     snprintf(KeyName, 256,
//                         "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%*s",
//                         Entry->if_descrlen, &Entry->if_descr[0]);
//
//                     if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, KeyName, 0, KEY_READ, &InterfaceKey) != ERROR_SUCCESS)
//                     {
//                         TRACE!("Failed opening interface key for interface %*s\n", Entry->if_descrlen, &Entry->if_descr[0]);
//                     }
//                     else
//                     {
//                         PIP_ADAPTER_DNS_SERVER_ADDRESS* ServerAddressPtr;
//
//                         CurrentAA->FirstDnsServerAddress = (PIP_ADAPTER_DNS_SERVER_ADDRESS)Ptr;
//                         ServerAddressPtr = &CurrentAA->FirstDnsServerAddress;
//
//                         EnumNameServers(InterfaceKey, NULL, &ServerAddressPtr, EnumerateServerName);
//                         RegCloseKey(InterfaceKey);
//
//                         /* Set the last entry in the list as having NULL next member */
//                         Ptr = (BYTE*)*ServerAddressPtr;
//                         *ServerAddressPtr = NULL;
//                     }
//                 }
//
//                 /* We're done for this interface */
//                 PreviousAA = CurrentAA;
//                 RemainingSize -= CurrentAASize;
//             }
//         }
//     }
//
//     if (AdaptersCount == 0)
//     {
//         /* Uh? Not even localhost ?! */
//         ERR!("No Adapters found!\n");
//         *pOutBufLen = 0;
//         return ERROR_NO_DATA;
//     }
//
//     /* See if we have anything to add */
//     // FIXME: Anycast and multicast
//     if ((Flags & (GAA_FLAG_SKIP_UNICAST | GAA_FLAG_INCLUDE_PREFIX)) == GAA_FLAG_SKIP_UNICAST)
//         goto Success;
//
//     /* Now fill in the addresses */
//     for (i = 0; i < InterfacesCount; i++)
//     {
//         /* Look for network layers */
//         if ((InterfacesList[i].tei_entity == CL_NL_ENTITY)
//                 || (InterfacesList[i].tei_entity == CO_NL_ENTITY))
//         {
//             IPSNMPInfo SnmpInfo;
//             PIP_ADAPTER_ADDRESSES CurrentAA = NULL;
//             IPAddrEntry* AddrEntries;
//             ULONG j;
//
//             /* Get its SNMP info */
//             Status = GetSnmpInfo(TcpFile, InterfacesList[i], &SnmpInfo);
//             if (!NT_SUCCESS(Status))
//                 goto Error;
//
//             if (SnmpInfo.ipsi_numaddr == 0)
//                 continue;
//
//             /* Allocate the address entry array and get them */
//             AddrEntries = HeapAlloc(GetProcessHeap(),
//                 HEAP_ZERO_MEMORY,
//                 SnmpInfo.ipsi_numaddr * sizeof(AddrEntries[0]));
//             if (!AddrEntries)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 goto Error;
//             }
//             Status = GetAddrEntries(TcpFile, InterfacesList[i], AddrEntries, SnmpInfo.ipsi_numaddr);
//             if (!NT_SUCCESS(Status))
//             {
//                 HeapFree(GetProcessHeap(), 0, AddrEntries);
//                 goto Error;
//             }
//
//             for (j = 0; j < SnmpInfo.ipsi_numaddr; j++)
//             {
//                 /* Find the adapters struct for this address. */
//                 if (pAdapterAddresses)
//                 {
//                     CurrentAA = pAdapterAddresses;
//                     while (CurrentAA)
//                     {
//                         if (CurrentAA->IfIndex == AddrEntries[j].iae_index)
//                             break;
//
//                         CurrentAA = CurrentAA->Next;
//                     }
//
//                     if (!CurrentAA)
//                     {
//                         ERR!("Got address for interface %u but no adapter was found for it.\n", AddrEntries[j].iae_index);
//                         /* Go to the next address */
//                         continue;
//                     }
//                 }
//
//                 TRACE!("address is 0x%08x, mask is 0x%08x\n", AddrEntries[j].iae_addr, AddrEntries[j].iae_mask);
//
//                 //FIXME: For now reactos only supports unicast addresses
//                 if (!(Flags & GAA_FLAG_SKIP_UNICAST))
//                 {
//                     ULONG Size = sizeof(IP_ADAPTER_UNICAST_ADDRESS) + sizeof(SOCKADDR);
//
//                     if (Ptr && (RemainingSize >= Size))
//                     {
//                         PIP_ADAPTER_UNICAST_ADDRESS UnicastAddress = (PIP_ADAPTER_UNICAST_ADDRESS)Ptr;
//
//                         /* Fill in the structure */
//                         UnicastAddress->Length = sizeof(IP_ADAPTER_UNICAST_ADDRESS);
//                         UnicastAddress->Next = CurrentAA->FirstUnicastAddress;
//
//                         // FIXME: Put meaningful value here
//                         UnicastAddress->Flags = 0;
//                         UnicastAddress->PrefixOrigin = IpPrefixOriginOther;
//                         UnicastAddress->SuffixOrigin = IpSuffixOriginOther;
//                         UnicastAddress->DadState = IpDadStatePreferred;
//                         UnicastAddress->ValidLifetime = 0xFFFFFFFF;
//                         UnicastAddress->PreferredLifetime = 0xFFFFFFFF;
//
//                         /* Set the address */
//                         //FIXME: ipv4 only (again...)
//                         UnicastAddress->Address.lpSockaddr = (LPSOCKADDR)(UnicastAddress + 1);
//                         UnicastAddress->Address.iSockaddrLength = sizeof(SOCKADDR);
//                         UnicastAddress->Address.lpSockaddr->sa_family = AF_INET;
//                         ((LPSOCKADDR_IN)UnicastAddress->Address.lpSockaddr)->sin_port = 0;
//                         ((LPSOCKADDR_IN)UnicastAddress->Address.lpSockaddr)->sin_addr.S_un.S_addr = AddrEntries[j].iae_addr;
//
//                         CurrentAA->FirstUnicastAddress = UnicastAddress;
//                         Ptr += Size;
//                         RemainingSize -= Size;
//                     }
//
//                     TotalSize += Size;
//                 }
//
//                 if (Flags & GAA_FLAG_INCLUDE_PREFIX)
//                 {
//                     ULONG Size = sizeof(IP_ADAPTER_PREFIX) + sizeof(SOCKADDR);
//
//                     if (Ptr && (RemainingSize >= Size))
//                     {
//                         PIP_ADAPTER_PREFIX Prefix = (PIP_ADAPTER_PREFIX)Ptr;
//
//                         /* Fill in the structure */
//                         Prefix->Length = sizeof(IP_ADAPTER_PREFIX);
//                         Prefix->Next = CurrentAA->FirstPrefix;
//
//                         /* Set the address */
//                         //FIXME: ipv4 only (again...)
//                         Prefix->Address.lpSockaddr = (LPSOCKADDR)(Prefix + 1);
//                         Prefix->Address.iSockaddrLength = sizeof(SOCKADDR);
//                         Prefix->Address.lpSockaddr->sa_family = AF_INET;
//                         ((LPSOCKADDR_IN)Prefix->Address.lpSockaddr)->sin_port = 0;
//                         ((LPSOCKADDR_IN)Prefix->Address.lpSockaddr)->sin_addr.S_un.S_addr = AddrEntries[j].iae_addr & AddrEntries[j].iae_mask;
//
//                         /* Compute the prefix size */
//                         Prefix->PrefixLength = CountPrefixBits(AddrEntries[j].iae_mask);
//
//                         CurrentAA->FirstPrefix = Prefix;
//                         Ptr += Size;
//                         RemainingSize -= Size;
//                     }
//
//                     TotalSize += Size;
//                 }
//             }
//
//             HeapFree(GetProcessHeap(), 0, AddrEntries);
//         }
//     }
//
// Success:
//     /* We're done */
//     HeapFree(GetProcessHeap(), 0, InterfacesList);
//     NtClose(TcpFile);
//     *pOutBufLen = TotalSize;
//     TRACE!("TotalSize: %x\n", *pOutBufLen);
//     return ERROR_SUCCESS;
//
// Error:
//     ERR!("Failed! Status 0x%08x\n", Status);
//     *pOutBufLen = 0;
//     HeapFree(GetProcessHeap(), 0, InterfacesList);
//     NtClose(TcpFile);
//     return RtlNtStatusToDosError(Status);
// }
//
//
// DWORD
// WINAPI
// NhpAllocateAndGetInterfaceInfoFromStack(
//      IP_INTERFACE_NAME_INFO **ppTable,
//      PDWORD pdwCount,
//      BOOL bOrder,
//      HANDLE hHeap,
//      DWORD dwFlags)
// {
//     NTSTATUS Status;
//     HANDLE TcpFile;
//     TDIEntityID* InterfacesList;
//     ULONG InterfacesCount;
//     ULONG AdaptersCount = 0;
//     ULONG i, j;
//     IP_INTERFACE_NAME_INFO *pTable = NULL;
//     UNICODE_STRING UnicodeGuidString;
//     ANSI_STRING AnsiGuidString;
//
//     TRACE!("ppTable %p, pdwCount %p, bOrder %u, hHeap %p, dwFlags %lu\n",
//           ppTable, pdwCount, bOrder, hHeap, dwFlags);
//
//     /* open the tcpip driver */
//     Status = openTcpFile(&TcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Could not open handle to tcpip.sys. Status %08x\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Get the interfaces list */
//     Status = GetInterfacesList(TcpFile, &InterfacesList, &InterfacesCount);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Could not get adapters list. Status %08x\n", Status);
//         NtClose(TcpFile);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     for (i = 0; i < InterfacesCount; i++)
//     {
//         if (InterfacesList[i].tei_entity == IF_ENTITY)
//         {
//             BYTE EntryBuffer[FIELD_OFFSET(IFEntry, if_descr) +
//                              RTL_FIELD_SIZE(IFEntry, if_descr[0]) * (MAX_ADAPTER_DESCRIPTION_LENGTH + 1)];
//             IFEntry* Entry = (IFEntry*)EntryBuffer;
//
//             /* Get the entry */
//             Status = GetInterfaceEntry(TcpFile, InterfacesList[i], Entry);
//             if (!NT_SUCCESS(Status))
//                 goto Error;
//
//             if (Entry->if_type != IF_TYPE_SOFTWARE_LOOPBACK)
//                 AdaptersCount++;
//         }
//     }
//
//     TRACE!("InterfacesCount %lu  AdaptersCount %lu\n", InterfacesCount, AdaptersCount);
//
//     pTable = HeapAlloc(hHeap, dwFlags, AdaptersCount * sizeof(IP_INTERFACE_NAME_INFO));
//     if (pTable == NULL)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Error;
//     }
//
//     for (i = 0, j = 0; i < InterfacesCount; i++)
//     {
//         if (InterfacesList[i].tei_entity == IF_ENTITY)
//         {
//             BYTE EntryBuffer[FIELD_OFFSET(IFEntry, if_descr) +
//                              RTL_FIELD_SIZE(IFEntry, if_descr[0]) * (MAX_ADAPTER_DESCRIPTION_LENGTH + 1)];
//             IFEntry* Entry = (IFEntry*)EntryBuffer;
//
//             /* Get the entry */
//             Status = GetInterfaceEntry(TcpFile, InterfacesList[i], Entry);
//             if (!NT_SUCCESS(Status))
//                 goto Error;
//
//             TRACE!("Got entity %*s, index %u.\n",
//                   Entry->if_descrlen, &Entry->if_descr[0], Entry->if_index);
//
//             if (Entry->if_type != IF_TYPE_SOFTWARE_LOOPBACK)
//             {
//                 pTable[j].Index = Entry->if_index;
//                 pTable[j].MediaType = Entry->if_type;
//                 pTable[j].ConnectionType = 1; //IF_CONNECTION_DEDICATED;
//                 pTable[j].AccessType = 2; //IF_ACCESS_BROADCAST;
//
//                 AnsiGuidString.Length = Entry->if_descrlen;
//                 AnsiGuidString.MaximumLength = Entry->if_descrlen;
//                 AnsiGuidString.Buffer = (PCHAR)Entry->if_descr;
//                 Status = RtlAnsiStringToUnicodeString(&UnicodeGuidString,
//                                                       &AnsiGuidString, TRUE);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     ERR!("Status %lx\n", Status);
//                     goto Error;
//                 }
//
//                 Status = RtlGUIDFromString(&UnicodeGuidString, &pTable[j].DeviceGuid);
//                 RtlFreeUnicodeString(&UnicodeGuidString);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     ERR!("Status %lx\n", Status);
//                     goto Error;
//                 }
//
//                 memset(&pTable[j].InterfaceGuid, 0, sizeof(GUID));
//
//                 j++;
//             }
//         }
//     }
//
//     /* We're done */
//     HeapFree(GetProcessHeap(), 0, InterfacesList);
//     NtClose(TcpFile);
//     *ppTable = pTable;
//     *pdwCount = AdaptersCount;
//     return ERROR_SUCCESS;
//
// Error:
//     ERR!("Failed! Status 0x%08x\n", Status);
//     if (pTable)
//         HeapFree(hHeap, 0, pTable);
//     *ppTable = NULL;
//     *pdwCount = 0;
//     HeapFree(GetProcessHeap(), 0, InterfacesList);
//     NtClose(TcpFile);
//     return RtlNtStatusToDosError(Status);
// }
//
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
