use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut hAdapterStateChangedEvent: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut DhcpSocket: SOCKET = INVALID_SOCKET;

static mut AdapterList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

static mut wsd: WSADATA = unsafe { core::mem::zeroed() };

// ---- GetSubkeyNames ----
#[no_mangle]
pub unsafe extern "system" fn GetSubkeyNames(
    MainKeyName: *mut i8,
    Append: *mut i8,
) -> *mut *mut i8 {
    let mut i: i32 = 0;
    let mut Error: u32;
    let mut MainKey: HKEY = core::ptr::null_mut();
    let Out: *mut *mut i8;
    let mut OutKeyName: *mut i8;
    let CharTotal: usize;
    let AppendLen: usize = 1 + strlen((Append as *const u8) as *const i8) as usize;
    let mut MaxSubKeyLen: u32 = 0;
    let mut MaxSubKeys: u32 = 0;

    Error = (RegOpenKey(HKEY_LOCAL_MACHINE, MainKeyName, &mut MainKey)) as u32;
    if Error != 0 {
        return core::ptr::null_mut();
    }

    Error = RegQueryInfoKey(
        MainKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut MaxSubKeys,
        &mut MaxSubKeyLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    MaxSubKeyLen += 1;

    DH_DbgPrint!(
        MID_TRACE,
        b"MaxSubKeys: %d, MaxSubKeyLen %d\n\0".as_ptr() as *const i8,
        MaxSubKeys,
        MaxSubKeyLen,
    );

    CharTotal = (core::mem::size_of::<*mut i8>() + MaxSubKeyLen as usize + AppendLen)
        * (MaxSubKeys as usize + 1);

    DH_DbgPrint!(
        MID_TRACE,
        b"AppendLen: %d, CharTotal: %d\n\0".as_ptr() as *const i8,
        AppendLen as u32,
        CharTotal as u32,
    );

    Out = malloc(CharTotal) as *mut *mut i8;
    OutKeyName = Out.add(MaxSubKeys as usize + 1) as *mut i8;

    if Out.is_null() {
        RegCloseKey(MainKey);
        return core::ptr::null_mut();
    }

    i = 0;
    loop {
        *Out.add(i as usize) = OutKeyName;
        Error = RegEnumKey(MainKey, i as u32, OutKeyName, MaxSubKeyLen) as u32;
        if Error == 0 {
            strcat(OutKeyName, Append);
            DH_DbgPrint!(
                MID_TRACE,
                b"[%d]: %s\n\0".as_ptr() as *const i8,
                i,
                OutKeyName,
            );
            OutKeyName = OutKeyName.add(strlen((OutKeyName as *const u8) as *const i8) + 1);
            i += 1;
        } else {
            *Out.add(i as usize) = core::ptr::null_mut();
        }
        if Error != 0 {
            break;
        }
    }

    RegCloseKey(MainKey);
    Out
}

// ---- RegReadString ----
unsafe fn RegReadString(RootKey: HKEY, Subkey: PCHAR, Value: PCHAR) -> PCHAR {
    let mut SubOut: PCHAR = core::ptr::null_mut();
    let mut SubOutLen: DWORD = 0;
    let mut Error: DWORD = 0;
    let mut ValueKey: HKEY = core::ptr::null_mut();
    let mut error_path: bool = false;

    DH_DbgPrint!(
        MID_TRACE,
        b"Looking in %x:%s:%s\n\0".as_ptr() as PCHAR,
        RootKey,
        Subkey,
        Value,
    );

    if !Subkey.is_null() && strlen((Subkey as *const u8) as *const i8) != 0 {
        if RegOpenKey((RootKey) as *mut HKEY__, Subkey, &mut ValueKey) != ((ERROR_SUCCESS) as i32) as u32 {
            error_path = true;
        }
    } else {
        ValueKey = (RootKey) as *mut HKEY__;
    }

    if !error_path {
        DH_DbgPrint!(
            MID_TRACE,
            b"Got Key %x\n\0".as_ptr() as PCHAR,
            ValueKey,
        );

        Error = RegQueryValueEx(
            ValueKey,
            Value,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SubOut as LPBYTE,
            &mut SubOutLen,
        );

        if Error != ERROR_SUCCESS {
            error_path = true;
        } else {
            DH_DbgPrint!(
                MID_TRACE,
                b"Value %s has size %d\n\0".as_ptr() as PCHAR,
                Value,
                SubOutLen,
            );

            SubOut = malloc(SubOutLen as usize) as PCHAR;
            if SubOut.is_null() {
                error_path = true;
            } else {
                Error = RegQueryValueEx(
                    ValueKey,
                    Value,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    SubOut as LPBYTE,
                    &mut SubOutLen,
                );

                if Error != ERROR_SUCCESS {
                    error_path = true;
                } else {
                    DH_DbgPrint!(
                        MID_TRACE,
                        b"Value %s is %s\n\0".as_ptr() as PCHAR,
                        Value,
                        SubOut,
                    );
                }
            }
        }
    }

    // regerror:
    if error_path {
        if !SubOut.is_null() {
            free(SubOut as *mut _);
            SubOut = core::ptr::null_mut();
        }
    }

    // cleanup:
    if !ValueKey.is_null() && ValueKey != (RootKey) as *mut HKEY__ {
        DH_DbgPrint!(
            MID_TRACE,
            b"Closing key %x\n\0".as_ptr() as PCHAR,
            ValueKey,
        );
        RegCloseKey(ValueKey);
    }

    DH_DbgPrint!(
        MID_TRACE,
        b"Returning %x with error %d\n\0".as_ptr() as PCHAR,
        SubOut,
        Error,
    );

    SubOut
}

// ---- AdapterInit ----
#[no_mangle]
pub unsafe extern "C" fn AdapterInit() {
    WSAStartup(0x0101, &mut wsd);
    InitializeListHead(core::ptr::addr_of_mut!(AdapterList));
}

// ---- InterfaceConnected ----
unsafe fn InterfaceConnected(IfEntry: *const MIB_IFROW) -> i32 {
    if (*IfEntry).dwOperStatus == IF_OPER_STATUS_CONNECTED ||
        (*IfEntry).dwOperStatus == IF_OPER_STATUS_OPERATIONAL {
        return 1;
    }
    DH_DbgPrint!(MID_TRACE, c"Interface %d is down\n".as_ptr() as *const u8, (*IfEntry).dwIndex);
    0
}

// ---- IsReconnectHackNeeded ----
unsafe fn IsReconnectHackNeeded(Adapter: PDHCP_ADAPTER, IfEntry: *const MIB_IFROW) -> i32 {
    let ZeroAddress = b"0.0.0.0\0".as_ptr() as *const i8;
    let proto = find_protocol_by_adapter(core::ptr::addr_of_mut!((*Adapter).DhclientInfo));
    if (*(*Adapter).DhclientInfo.client).state == S_BOUND && proto.is_null() {
        return 0;
    }
    if (*(*Adapter).DhclientInfo.client).state != S_BOUND && (*(*Adapter).DhclientInfo.client).state != S_STATIC {
        return 0;
    }
    ApiUnlock();
    let mut Size = core::mem::size_of::<IP_ADAPTER_INFO>() as u32;
    let mut AdapterInfo = HeapAlloc(GetProcessHeap(), 0, (Size) as usize) as PIP_ADAPTER_INFO;
    let mut Orig = AdapterInfo;
    if AdapterInfo.is_null() {
        ApiLock();
        return 0;
    }
    let mut Ret = GetAdaptersInfo(AdapterInfo, &mut Size);
    if Ret == ERROR_BUFFER_OVERFLOW {
        HeapFree(GetProcessHeap(), 0, Orig as LPVOID);
        AdapterInfo = HeapAlloc(GetProcessHeap(), 0, (Size) as usize) as PIP_ADAPTER_INFO;
        if AdapterInfo.is_null() {
            ApiLock();
            return 0;
        }
        Orig = AdapterInfo;
        if GetAdaptersInfo(AdapterInfo, &mut Size) != NO_ERROR {
            HeapFree(GetProcessHeap(), 0, Orig as LPVOID);
            ApiLock();
            return 0;
        }
        while !AdapterInfo.is_null() {
            if (*AdapterInfo).Index == (*IfEntry).dwIndex {
                break;
            }
            AdapterInfo = (*AdapterInfo).Next;
        }
        if AdapterInfo.is_null() {
            HeapFree(GetProcessHeap(), 0, Orig as LPVOID);
            ApiLock();
            return 0;
        }
    } else if Ret != NO_ERROR {
        HeapFree(GetProcessHeap(), 0, Orig as LPVOID);
        ApiLock();
        return 0;
    }
    let result: i32;
    if strcmp((*AdapterInfo).IpAddressList.IpAddress.String.as_ptr(), ZeroAddress) == 0 {
        result = 1;
    } else {
        result = 0;
    }
    HeapFree(GetProcessHeap(), 0, Orig as LPVOID);
    ApiLock();
    result
}

// ---- StartAdapterDiscovery ----
#[no_mangle]
pub unsafe extern "system" fn StartAdapterDiscovery(hStopEvent: HANDLE) -> HANDLE {
    CreateThread(core::ptr::null_mut(), 0, Some(AdapterDiscoveryThread as unsafe extern "system" fn(LPVOID) -> u32), hStopEvent as LPVOID, 0, core::ptr::null_mut())
}

// ---- AdapterStop ----
unsafe fn AdapterStop() {
    ApiLock();
    while !IsListEmpty(core::ptr::addr_of!(AdapterList)) {
        let listEntry = RemoveHeadList(core::ptr::addr_of_mut!(AdapterList));
        let listEntryOffset = core::mem::offset_of!(DHCP_ADAPTER, ListEntry);
        let adapter = (listEntry as *mut u8).sub(listEntryOffset) as *mut DHCP_ADAPTER;
        free(adapter as *mut core::ffi::c_void);
    }
    ApiUnlock();
    WSACleanup();
}

// ---- AdapterFindIndex ----
#[no_mangle]
pub unsafe extern "system" fn AdapterFindIndex(indx: u32) -> PDHCP_ADAPTER {
    let mut ListEntry: PLIST_ENTRY = (*core::ptr::addr_of_mut!(AdapterList)).Flink;
    let offset = core::mem::offset_of!(DHCP_ADAPTER, ListEntry);
    while ListEntry != core::ptr::addr_of_mut!(AdapterList) {
        let Adapter: PDHCP_ADAPTER = &mut *((ListEntry as *mut u8).sub(offset) as *mut DHCP_ADAPTER);
        if (*Adapter).IfMib.dwIndex == indx {
            return Adapter;
        }
        ListEntry = (*ListEntry).Flink;
    }
    core::ptr::null_mut()
}

// ---- AdapterFindName ----
pub unsafe fn AdapterFindName(name: *const u16) -> *mut DHCP_ADAPTER {
    let mut list_entry: PLIST_ENTRY = AdapterList.Flink;
    let mut unicode_name: [u16; 45] = [0u16; 45];

    while list_entry as *const _ != &AdapterList as *const LIST_ENTRY as PLIST_ENTRY {
        let adapter: *mut DHCP_ADAPTER = {
            let offset = core::mem::offset_of!(DHCP_ADAPTER, ListEntry);
            (list_entry as usize).wrapping_sub(offset) as *mut DHCP_ADAPTER
        };

        let src = (*adapter).IfMib.bDescr.as_ptr() as *const i8;
        let len = strlen(src) + 1;
        mbstowcs(unicode_name.as_mut_ptr(), src, len);

        if wcsicmp(unicode_name.as_ptr(), name) == 0 {
            return adapter;
        }

        list_entry = (*list_entry).Flink;
    }

    core::ptr::null_mut()
}

// ---- AdapterFindInfo ----
#[no_mangle]
pub unsafe extern "system" fn AdapterFindInfo(ip: *mut interface_info) -> PDHCP_ADAPTER {
    let mut ListEntry: PLIST_ENTRY = AdapterList.Flink;
    let adapter_list_ptr = &mut AdapterList as *mut LIST_ENTRY;
    while ListEntry != adapter_list_ptr {
        let offset = core::mem::offset_of!(DHCP_ADAPTER, ListEntry);
        let Adapter = (ListEntry as usize - offset) as *mut DHCP_ADAPTER;
        if ip == &mut (*Adapter).DhclientInfo {
            return Adapter;
        }
        ListEntry = (*ListEntry).Flink;
    }
    core::ptr::null_mut()
}

// ---- AdapterFindByHardwareAddress ----
pub unsafe fn AdapterFindByHardwareAddress(haddr: *const u8, hlen: u8) -> *mut DHCP_ADAPTER {
    let mut ListEntry: PLIST_ENTRY = AdapterList.Flink;
    while ListEntry != core::ptr::addr_of_mut!(AdapterList) {
        let Adapter = (ListEntry as *mut u8)
            .sub(core::mem::offset_of!(DHCP_ADAPTER, ListEntry))
            as *mut DHCP_ADAPTER;
        if (*Adapter).DhclientInfo.hw_address.hlen == hlen
            && memcmp(
                core::ptr::addr_of!((*Adapter).DhclientInfo.hw_address.haddr[0]) as *const c_void,
                haddr as *const c_void,
                hlen as usize,
            ) == 0
        {
            return Adapter;
        }
        ListEntry = (*ListEntry).Flink;
    }

    core::ptr::null_mut()
}

// ---- AdapterGetFirst ----
unsafe fn AdapterGetFirst() -> *mut _DHCP_ADAPTER {
    if IsListEmpty(&AdapterList as *const _ as *mut _) {
        core::ptr::null_mut()
    } else {
        let flink = AdapterList.Flink;
        let offset = core::mem::offset_of!(_DHCP_ADAPTER, ListEntry) as isize;
        (flink as *mut u8).offset(-offset) as *mut _DHCP_ADAPTER
    }
}

// ---- AdapterGetNext ----
unsafe fn AdapterGetNext(This: PDHCP_ADAPTER) -> PDHCP_ADAPTER {
    if (*This).ListEntry.Flink == core::ptr::addr_of!(AdapterList) as *mut LIST_ENTRY {
        return core::ptr::null_mut();
    }
    let flink = (*This).ListEntry.Flink;
    let offset = core::mem::offset_of!(DHCP_ADAPTER, ListEntry);
    (flink as *mut u8).sub(offset) as *mut DHCP_ADAPTER
}

// ---- if_register_send ----
unsafe fn if_register_send(ip: *mut interface_info) {
}

// ---- if_register_receive ----
unsafe fn if_register_receive(ip: *mut interface_info) {}

// ---- FindAdapterKey ----
pub unsafe fn FindAdapterKey(Adapter: PDHCP_ADAPTER) -> HKEY {
    let EnumKeyName: PCHAR = b"SYSTEM\\CurrentControlSet\\Control\\Class\\{4D36E972-E325-11CE-BFC1-08002BE10318}\0".as_ptr() as PCHAR;
    let TargetKeyNameStart: PCHAR = b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\\0".as_ptr() as PCHAR;
    let mut TargetKeyName: PCHAR = core::ptr::null_mut();
    let mut EnumKeysLinkage: *mut PCHAR = GetSubkeyNames(EnumKeyName, b"\\Linkage\0".as_ptr() as PCHAR);
    let mut EnumKeysTop: *mut PCHAR = GetSubkeyNames(EnumKeyName, b"\0".as_ptr() as PCHAR);
    let mut RootDevice: PCHAR = core::ptr::null_mut();
    let mut EnumKey: HKEY = core::ptr::null_mut();
    let mut OutKey: HKEY = core::ptr::null_mut();
    let mut Error: u32 = 0;

    if EnumKeysLinkage.is_null() || EnumKeysTop.is_null() {
        Error = 1; // dummy non-zero to trigger cleanup
    }

    if Error == 0 {
        Error = RegOpenKey(HKEY_LOCAL_MACHINE, EnumKeyName, &mut EnumKey) as u32;
    }

    if Error == 0 {
        for i in 0.. {
            let current = *EnumKeysLinkage.add(i);
            if current.is_null() {
                break;
            }
            RootDevice = RegReadString(EnumKey, current, b"RootDevice\0".as_ptr() as PCHAR);
            if !RootDevice.is_null() && strcmp(RootDevice, (*Adapter).DhclientInfo.name.as_ptr()) == 0 {
                let len = strlen(TargetKeyNameStart) + strlen(RootDevice) + 1;
                TargetKeyName = malloc(len) as PCHAR;
                if TargetKeyName.is_null() {
                    break;
                }
                sprintf(TargetKeyName, b"%s%s\0".as_ptr() as *const i8, TargetKeyNameStart, RootDevice);
                RegCreateKeyExA(HKEY_LOCAL_MACHINE, TargetKeyName, 0, core::ptr::null_mut(), 0, KEY_READ, core::ptr::null_mut(), &mut OutKey, core::ptr::null_mut());
                break;
            } else {
                if !RootDevice.is_null() {
                    free((RootDevice) as *mut c_void);
                    RootDevice = core::ptr::null_mut();
                }
            }
        }
    }

    // cleanup
    if !RootDevice.is_null() {
        free((RootDevice) as *mut c_void);
    }
    if !EnumKeysLinkage.is_null() {
        free((EnumKeysLinkage) as *mut c_void);
    }
    if !EnumKeysTop.is_null() {
        free((EnumKeysTop) as *mut c_void);
    }
    if !TargetKeyName.is_null() {
        free((TargetKeyName) as *mut c_void);
    }

    OutKey
}

// ---- PrepareAdapterForService ----
#[no_mangle]
pub unsafe extern "system" fn PrepareAdapterForService(Adapter: *mut DHCP_ADAPTER) -> i32 {
    let mut AdapterKey: HKEY = core::ptr::null_mut();
    let mut Error: u32 = 0; // ERROR_SUCCESS
    let mut DhcpEnabled: u32 = 0;
    let mut Length: u32 = core::mem::size_of::<u32>() as u32;

    // Set config pointer
    (*Adapter).DhclientState.config = &mut (*Adapter).DhclientConfig;

    // Copy adapter description to name
    extern "C" { fn strncpy(_: *mut i8, _: *const i8, _: usize) -> *mut i8; }
    let dest_size = core::mem::size_of_val(&(*Adapter).DhclientInfo.name);
    strncpy(
        (*Adapter).DhclientInfo.name.as_mut_ptr() as *mut i8,
        (*Adapter).IfMib.bDescr.as_ptr() as *const i8,
        dest_size,
    );

    AdapterKey = FindAdapterKey(Adapter);
    if !AdapterKey.is_null() {
        // (local extern removed: advapi32 exports RegQueryValueExA/W, never an undecorated
        //  `RegQueryValueEx` — that symbol would not resolve at link. The prelude binds the
        //  ANSI entry point via #[link_name = "RegQueryValueExA"].)
        Error = (RegQueryValueEx(
            AdapterKey,
            b"EnableDHCP\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut DhcpEnabled as *mut u32 as *mut u8,
            &mut Length as *mut u32,
        )) as u32;

        if Error != 0 || Length != core::mem::size_of::<u32>() as u32 {
            DhcpEnabled = 1;
        }

        RegCloseKey(AdapterKey);
    } else {
        DhcpEnabled = 1;
    }

    if DhcpEnabled == 0 {
        DbgPrint(
            b"DHCPCSVC: Adapter Name: [%s] (static)\n\0".as_ptr() as *const i8,
            (*Adapter).DhclientInfo.name.as_ptr() as *const i8,
        );
        (*Adapter).DhclientState.state = S_STATIC;
    } else {
        DbgPrint(
            b"DHCPCSVC: Adapter Name: [%s] (dynamic)\n\0".as_ptr() as *const i8,
            (*Adapter).DhclientInfo.name.as_ptr() as *const i8,
        );
        (*(*Adapter).DhclientInfo.client).state = S_INIT;
    }

    1i32
}

// ---- AdapterDiscoveryThread ----
#[no_mangle]
pub unsafe extern "system" fn AdapterDiscoveryThread(Context: LPVOID) -> DWORD {
    let mut Table: *mut MIB_IFTABLE = malloc(core::mem::size_of::<MIB_IFTABLE>()) as *mut MIB_IFTABLE;
    let mut Error: DWORD;
    let mut Size: DWORD = core::mem::size_of::<MIB_IFTABLE>() as DWORD;
    let mut Adapter: *mut DHCP_ADAPTER = core::ptr::null_mut();
    let hStopEvent: HANDLE = Context as HANDLE;
    let mut ifi: *mut interface_info = core::ptr::null_mut();
    let mut proto: *mut protocol;
    let mut i: i32;
    let mut AdapterCount: i32 = 0;
    let mut Broadcast: i32;

    loop {
        DH_DbgPrint!(MID_TRACE, b"Getting Adapter List...\n\0".as_ptr() as PCHAR);

        loop {
            Error = GetIfTable(Table, &mut Size, 0);
            if Error as u32 != ERROR_INSUFFICIENT_BUFFER {
                break;
            }
            DH_DbgPrint!(
                MID_TRACE,
                b"Error %d, New Buffer Size: %d\n\0".as_ptr() as PCHAR,
                Error,
                Size,
            );
            free(Table as *mut core::ffi::c_void);
            Table = malloc(Size as usize) as *mut MIB_IFTABLE;
        }

        if Error as u32 != NO_ERROR {
            /* HACK: We are waiting until TCP/IP starts */
            Sleep(2000);
            continue;
        }

        DH_DbgPrint!(
            MID_TRACE,
            b"Got Adapter List (%d entries)\n\0".as_ptr() as PCHAR,
            (*Table).dwNumEntries,
        );

        i = ((*Table).dwNumEntries as i32) - 1;
        while i >= 0 {
            DH_DbgPrint!(
                MID_TRACE,
                b"Getting adapter %d attributes\n\0".as_ptr() as PCHAR,
                (*(*Table).table.as_ptr().add(i as usize)).dwIndex,
            );

            ApiLock();

            Adapter = AdapterFindByHardwareAddress(
                (*(*Table).table.as_ptr().add(i as usize)).bPhysAddr.as_ptr(),
                (*(*Table).table.as_ptr().add(i as usize)).dwPhysAddrLen as u8,
            );

            if !Adapter.is_null() {
                proto = find_protocol_by_adapter(&mut (*Adapter).DhclientInfo);

                /* This is an existing adapter */
                if InterfaceConnected((*Table).table.as_ptr().add(i as usize)) != 0 {
                    /* We're still active so we stay in the list */
                    ifi = &mut (*Adapter).DhclientInfo;

                    /* This is a hack because IP helper API sucks */
                    if IsReconnectHackNeeded(Adapter, (*Table).table.as_ptr().add(i as usize)) != 0 {
                        /* This handles a disconnect/reconnect */
                        if !proto.is_null() {
                            remove_protocol(proto);
                        }
                        (*((*Adapter).DhclientInfo.client)).state = S_INIT;

                        /* These are already invalid since the media state change */
                        (*Adapter).RouterMib.dwForwardNextHop = 0;
                        (*Adapter).NteContext = 0;

                        add_protocol(
                            (*Adapter).DhclientInfo.name.as_mut_ptr(),
                            (*Adapter).DhclientInfo.rfdesc,
                            got_one,
                            core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void,
                        );
                        state_init(core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void);

                        SetEvent(hAdapterStateChangedEvent);
                    }
                } else {
                    if !proto.is_null() {
                        remove_protocol(proto);
                    }

                    /* We've lost our link so out we go */
                    RemoveEntryList(&mut (*Adapter).ListEntry);
                    free(Adapter as *mut core::ffi::c_void);
                }

                ApiUnlock();
                i -= 1;
                continue;
            }

            ApiUnlock();

            Adapter = calloc(
                1,
                core::mem::size_of::<DHCP_ADAPTER>() + (*(*Table).table.as_ptr().add(i as usize)).dwMtu as usize,
            ) as *mut DHCP_ADAPTER;

            if !Adapter.is_null()
                && ((*(*Table).table.as_ptr().add(i as usize)).dwType == MIB_IF_TYPE_ETHERNET
                    || (*(*Table).table.as_ptr().add(i as usize)).dwType == IF_TYPE_IEEE80211
                    && InterfaceConnected((*Table).table.as_ptr().add(i as usize)) != 0)
            {
                core::ptr::copy_nonoverlapping(
                    (*Table).table.as_ptr().add(i as usize),
                    &mut (*Adapter).IfMib as *mut MIB_IFROW,
                    1,
                );
                (*Adapter).DhclientInfo.client = &mut (*Adapter).DhclientState;
                (*Adapter).DhclientInfo.rbuf = (*Adapter).recv_buf.as_mut_ptr();
                (*Adapter).DhclientInfo.rbuf_max = ((*(*Table).table.as_ptr().add(i as usize)).dwMtu) as usize;
                (*Adapter).DhclientInfo.rbuf_len = 0;
                (*Adapter).DhclientInfo.rbuf_offset = 0;
                core::ptr::copy_nonoverlapping(
                    (*Adapter).IfMib.bPhysAddr.as_ptr(),
                    (*Adapter).DhclientInfo.hw_address.haddr.as_mut_ptr(),
                    (*Adapter).IfMib.dwPhysAddrLen as usize,
                );
                (*Adapter).DhclientInfo.hw_address.hlen = (*Adapter).IfMib.dwPhysAddrLen as u8;

                /* I'm not sure where else to set this, but
                   some DHCP servers won't take a zero.
                   We checked the hardware type earlier in
                   the if statement. */
                (*Adapter).DhclientInfo.hw_address.htype = HTYPE_ETHER as u8;

                if DhcpSocket == (INVALID_SOCKET as u64) as usize {
                    DhcpSocket = (socket(AF_INET as i32, SOCK_DGRAM as i32, IPPROTO_UDP as i32) as u64) as usize;
                    (*Adapter).DhclientInfo.rfdesc = (DhcpSocket as u64) as i32;
                    (*Adapter).DhclientInfo.wfdesc = (DhcpSocket as u64) as i32;

                    if DhcpSocket != (INVALID_SOCKET as u64) as usize {
                        /* Allow broadcast on this socket */
                        Broadcast = 1;
                        setsockopt(
                            (DhcpSocket as u64) as usize,
                            SOL_SOCKET as i32,
                            SO_BROADCAST as i32,
                            (&Broadcast as *const i32 as *const u8) as *const i8,
                            core::mem::size_of::<i32>() as i32,
                        );

                        (*Adapter).ListenAddr.sin_family = (AF_INET as u16) as i16;
                        (*Adapter).ListenAddr.sin_port = htons((LOCAL_PORT) as u16);
                        (*Adapter).BindStatus = if bind(
                            ((*Adapter).DhclientInfo.rfdesc) as usize,
                            &(*Adapter).ListenAddr as *const sockaddr_in as *const sockaddr,
                            core::mem::size_of::<sockaddr_in>() as i32,
                        ) == 0
                        {
                            0
                        } else {
                            (WSAGetLastError()) as u32
                        };
                    } else {
                        error(
                            b"socket() failed: %d\n\0".as_ptr() as PCHAR,
                            WSAGetLastError(),
                        );
                    }
                } else {
                    (*Adapter).DhclientInfo.rfdesc = (DhcpSocket as u64) as i32;
                    (*Adapter).DhclientInfo.wfdesc = (DhcpSocket as u64) as i32;
                }

                (*Adapter).DhclientConfig.timeout = (DHCP_PANIC_TIMEOUT) as time_t;
                (*Adapter).DhclientConfig.initial_interval = (DHCP_DISCOVER_INTERVAL) as time_t;
                (*Adapter).DhclientConfig.retry_interval = (DHCP_DISCOVER_INTERVAL) as time_t;
                (*Adapter).DhclientConfig.select_interval = 1;
                (*Adapter).DhclientConfig.reboot_timeout = (DHCP_REBOOT_TIMEOUT) as time_t;
                (*Adapter).DhclientConfig.backoff_cutoff = (DHCP_BACKOFF_MAX) as time_t;
                (*Adapter).DhclientState.interval = (*Adapter).DhclientConfig.retry_interval;

                if PrepareAdapterForService(Adapter) != 0 {
                    (*Adapter).DhclientInfo.next = ifi;
                    ifi = &mut (*Adapter).DhclientInfo;

                    read_client_conf(&mut (*Adapter).DhclientInfo);

                    if (*(*Adapter).DhclientInfo.client).state == S_INIT {
                        add_protocol(
                            (*Adapter).DhclientInfo.name.as_mut_ptr(),
                            (*Adapter).DhclientInfo.rfdesc,
                            got_one,
                            core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void,
                        );
                        state_init(core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void);
                    }

                    ApiLock();
                    InsertTailList(&mut AdapterList, &mut (*Adapter).ListEntry);
                    AdapterCount += 1;
                    SetEvent(hAdapterStateChangedEvent);
                    ApiUnlock();
                } else {
                    free(Adapter as *mut core::ffi::c_void);
                    Adapter = core::ptr::null_mut();
                }
            } else {
                free(Adapter as *mut core::ffi::c_void);
                Adapter = core::ptr::null_mut();
            }

            if Adapter.is_null() {
                DH_DbgPrint!(
                    MID_TRACE,
                    b"Adapter %d was rejected\n\0".as_ptr() as PCHAR,
                    (*(*Table).table.as_ptr().add(i as usize)).dwIndex,
                );
            }

            i -= 1;
        }

        if WaitForSingleObject(hStopEvent, 3000) == WAIT_OBJECT_0 {
            DPRINT!(b"Stopping the discovery thread!\n\0".as_ptr() as PCHAR);
            break;
        }
    }

    if !Table.is_null() {
        free(Table as *mut core::ffi::c_void);
    }

    DPRINT!(
        b"Adapter discovery thread terminated! (Error: %d)\n\0".as_ptr() as PCHAR,
        Error,
    );

    Error
}
