use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// NOTE (C fidelity): ipstats_reactos.c lines 26-58 carry an `#ifndef TCPS_X / # define TCPS_X
// TCP_X / #endif` ladder — a Wine portability vestige mapping BSD `TCPS_*` socket-state names
// onto Linux `TCP_*` ones. In the ReactOS build BOTH families are undefined, and crucially
// NOTHING in the translation unit ever references a `TCPS_*` name, so the macros are never
// expanded and the ladder contributes zero code. (An unexpanded object-like macro whose body
// names an undefined identifier is perfectly legal C — the error would only appear at a use
// site, and there is none.)
//
// The LLM materialized that dead preprocessor text into 11 live `pub const TCPS_* = TCP_*;`
// items, which is what made the 11 `TCP_*` names look "missing". They are not missing: they
// never existed in the compiled C. Transcribing them would have meant inventing values for
// identifiers that no ReactOS header defines. Deliberately rendered as this comment instead —
// the C counterpart is likewise pure commentary to the compiler.

#[repr(C)]
pub struct _TABLE_CALL {
    pub TOIID: u32,
    pub UdpSize: usize,
    pub TcpSize: usize,
    pub UdpOffset: usize,
    pub TcpOffset: usize,
}

pub static UdpTcpTableCall: [_TABLE_CALL; 3] = [
    _TABLE_CALL {
        TOIID: IP_MIB_ARPTABLE_ENTRY_ID,
        UdpSize: core::mem::size_of::<MIB_UDPROW>() as usize,
        TcpSize: core::mem::size_of::<MIB_TCPROW>() as usize,
        UdpOffset: core::mem::offset_of!(MIB_UDPTABLE, table) as usize,
        TcpOffset: core::mem::offset_of!(MIB_TCPTABLE, table) as usize,
    },
    _TABLE_CALL {
        TOIID: IP_MIB_ADDRTABLE_ENTRY_ID,
        UdpSize: core::mem::size_of::<MIB_UDPROW_OWNER_PID>() as usize,
        TcpSize: core::mem::size_of::<MIB_TCPROW_OWNER_PID>() as usize,
        UdpOffset: core::mem::offset_of!(MIB_UDPTABLE_OWNER_PID, table) as usize,
        TcpOffset: core::mem::offset_of!(MIB_TCPTABLE_OWNER_PID, table) as usize,
    },
    _TABLE_CALL {
        TOIID: IP_SPECIFIC_MODULE_ENTRY_ID,
        UdpSize: core::mem::size_of::<MIB_UDPROW_OWNER_MODULE>() as usize,
        TcpSize: core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>() as usize,
        UdpOffset: core::mem::offset_of!(MIB_UDPTABLE_OWNER_MODULE, table) as usize,
        TcpOffset: core::mem::offset_of!(MIB_TCPTABLE_OWNER_MODULE, table) as usize,
    },
];
#[no_mangle]
pub unsafe extern "C" fn isIpEntity(tcp_file: HANDLE, ent: *mut TDIEntityID) -> i32 {
    if ((*ent).tei_entity == CL_NL_ENTITY) || ((*ent).tei_entity == CO_NL_ENTITY) { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn getNthIpEntity(tcp_file: HANDLE, index: u32, ent: *mut TDIEntityID) -> NTSTATUS {
    let mut num_entities: u32 = 0;
    let mut num_routes: u32 = 0;
    let mut entity_set: *mut TDIEntityID = core::ptr::null_mut();
    let status = tdiGetEntityIDSet(tcp_file, &mut entity_set, &mut num_entities);
    if !NT_SUCCESS(status) {
        return status;
    }
    let mut i: u32 = 0;
    while i < num_entities {
        if isIpEntity(tcp_file, entity_set.add(i as usize)) != 0 {
            if num_routes == index {
                break;
            } else {
                num_routes += 1;
            }
        }
        i += 1;
    }
    if num_routes == index && i < num_entities {
        core::ptr::copy_nonoverlapping(entity_set.add(i as usize), ent, 1);
        tdiFreeThingSet((entity_set as *mut c_void));
        STATUS_SUCCESS
    } else {
        tdiFreeThingSet((entity_set as *mut c_void));
        STATUS_UNSUCCESSFUL
    }
}
unsafe extern "C" fn tdiGetMibForIpEntity(
    tcpFile: HANDLE,
    ent: *mut TDIEntityID,
    entry: *mut MIB_IPSTATS,
) -> NTSTATUS {
    let mut req: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut returnSize: DWORD = 0;

    ZeroMemory(entry as *mut c_void, core::mem::size_of::<MIB_IPSTATS>());

    req.ID.toi_class = INFO_CLASS_PROTOCOL;
    req.ID.toi_type = INFO_TYPE_PROVIDER;
    req.ID.toi_id = IP_MIB_STATS_ID;
    req.ID.toi_entity = core::ptr::read(ent);

    let status = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &req as *const _ as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
        entry as LPVOID,
        core::mem::size_of::<MIB_IPSTATS>() as DWORD,
        &mut returnSize,
        core::ptr::null_mut(),
    );

    status
}
unsafe extern "C" fn tdiGetMibForTcpEntity(
    tcpFile: HANDLE,
    ent: *mut TDIEntityID,
    entry: *mut MIB_TCPSTATS,
) -> NTSTATUS {
    let mut req: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut returnSize: DWORD = 0;

    core::ptr::write_bytes(entry as *mut u8, 0, core::mem::size_of::<MIB_TCPSTATS>());

    req.ID.toi_class = INFO_CLASS_PROTOCOL;
    req.ID.toi_type = INFO_TYPE_PROVIDER;
    req.ID.toi_id = TCP_MIB_STAT_ID;
    req.ID.toi_entity = core::ptr::read(ent);

    let status = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut req as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        entry as *mut core::ffi::c_void,
        core::mem::size_of::<MIB_TCPSTATS>() as u32,
        &mut returnSize,
        core::ptr::null_mut(),
    );

    status
}
unsafe extern "C" fn tdiGetMibForUdpEntity(
    tcpFile: HANDLE,
    ent: *const TDIEntityID,
    entry: *mut MIB_UDPSTATS,
) -> NTSTATUS {
    core::ptr::write_bytes(entry as *mut u8, 0, core::mem::size_of::<MIB_UDPSTATS>());

    let mut req: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    (*core::ptr::addr_of_mut!(req.ID)).toi_class = INFO_CLASS_PROTOCOL;
    (*core::ptr::addr_of_mut!(req.ID)).toi_type = INFO_TYPE_PROVIDER;
    (*core::ptr::addr_of_mut!(req.ID)).toi_id = UDP_MIB_STAT_ID;
    (*core::ptr::addr_of_mut!(req.ID)).toi_entity = core::ptr::read(ent);

    let mut returnSize: u32 = 0;
    let status = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut req as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        entry as *mut core::ffi::c_void,
        core::mem::size_of::<MIB_UDPSTATS>() as u32,
        &mut returnSize,
        core::ptr::null_mut(),
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn tdiGetRoutesForIpEntity(
    tcpFile: HANDLE,
    ent: *mut TDIEntityID,
    routes: *mut *mut IPRouteEntry,
    numRoutes: PDWORD,
) -> NTSTATUS {
    tdiGetSetOfThings(
        tcpFile,
        INFO_CLASS_PROTOCOL,
        INFO_TYPE_PROVIDER,
        IP_MIB_ARPTABLE_ENTRY_ID,
        CL_NL_ENTITY,
        (*ent).tei_instance,
        0,
        (core::mem::size_of::<IPRouteEntry>() as u32),
        routes as *mut PVOID,
        numRoutes,
    )
}
#[no_mangle]
pub unsafe extern "C" fn tdiGetIpAddrsForIpEntity(
    tcpFile: HANDLE,
    ent: *mut TDIEntityID,
    addrs: *mut *mut IPAddrEntry,
    numAddrs: PDWORD,
) -> NTSTATUS {
    let status = tdiGetSetOfThings(
        tcpFile,
        INFO_CLASS_PROTOCOL,
        INFO_TYPE_PROVIDER,
        IP_MIB_ADDRTABLE_ENTRY_ID,
        CL_NL_ENTITY,
        (*ent).tei_instance,
        0,
        core::mem::size_of::<IPAddrEntry>() as u32,
        addrs as *mut *mut core::ffi::c_void,
        numAddrs,
    );
    status
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceStatsByName(name: *const u8, entry: *mut MIB_IFROW) -> u32 {
    if name.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if entry.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceStatsByIndex(index: u32, entry: *mut core::ffi::c_void) -> u32 {
    ERROR_INVALID_PARAMETER
}
#[no_mangle]
pub unsafe extern "C" fn getICMPStats(stats: *mut MIB_ICMP) -> u32 {
    if stats.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    core::ptr::write_bytes(stats as *mut u8, 0, core::mem::size_of::<MIB_ICMP>());

    let fp = fopen(b"/proc/net/snmp\0".as_ptr() as *const i8, b"r\0".as_ptr() as *const i8);
    if fp.is_null() {
        return NO_ERROR;
    }

    let mut buf: [u8; 512] = [0; 512];
    let hdr = b"Icmp:";
    let hdr_len = hdr.len();

    // Search for line starting with "Icmp:"
    loop {
        let ptr = fgets(buf.as_mut_ptr() as *mut i8, buf.len() as i32, fp);
        if ptr.is_null() {
            break;
        }
        if _strnicmp(buf.as_ptr() as *const i8, hdr.as_ptr() as *const i8, (hdr_len as u32 as usize)) == 0 {
            break;
        }
    }

    let ptr = fgets(buf.as_mut_ptr() as *mut i8, buf.len() as i32, fp);
    if !ptr.is_null() && _strnicmp(buf.as_ptr() as *const i8, hdr.as_ptr() as *const i8, (hdr_len as u32 as usize)) == 0 {
        let mut ptr = (buf.as_ptr() as *const i8).add(hdr_len + 1);
        let mut end_ptr: *mut i8 = core::ptr::null_mut();

        // InMsgs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwMsgs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InErrors
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwErrors = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InDestUnreachs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwDestUnreachs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InTimeExcds
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwTimeExcds = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InParmProbs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwParmProbs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InSrcQuenchs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwSrcQuenchs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InRedirects
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwRedirects = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InEchoReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwEchoReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InTimestamps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwTimestamps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InTimestampReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwTimestampReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InAddrMasks
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwAddrMasks = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // InAddrMaskReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpInStats.dwAddrMaskReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutMsgs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwMsgs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutErrors
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwErrors = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutDestUnreachs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwDestUnreachs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutTimeExcds
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwTimeExcds = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutParmProbs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwParmProbs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutSrcQuenchs
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwSrcQuenchs = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutRedirects
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwRedirects = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutEchoReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwEchoReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutTimestamps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwTimestamps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutTimestampReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwTimestampReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutAddrMasks
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwAddrMasks = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
        // OutAddrMaskReps
        if !ptr.is_null() && *ptr != 0 {
            let val = strtoul(ptr, &mut end_ptr, 10);
            if end_ptr != core::ptr::null_mut() {
                (*stats).stats.icmpOutStats.dwAddrMaskReps = val as u32;
                ptr = end_ptr as *const i8;
            }
        }
    }

    fclose(fp);
    NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn getIPStats(tcp_file: HANDLE, p_stats: *mut MIB_IPSTATS) -> u32 {
    let ent = _TDIEntityID {
        tei_entity: CL_NL_ENTITY,
        tei_instance: 0,
    };

    let status = tdiGetMibForIpEntity(tcp_file, &ent as *const _ as *mut _, p_stats);

    if !NT_SUCCESS(status) {
        return RtlNtStatusToDosError(status);
    }

    0 // NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn getTCPStats(TcpFile: HANDLE, pStats: *mut MIB_TCPSTATS) -> DWORD {
    let mut ent: TDIEntityID = TDIEntityID {
        tei_entity: CO_TL_ENTITY,
        tei_instance: 0,
    };
    let Status: NTSTATUS = tdiGetMibForTcpEntity(TcpFile, &mut ent as *mut TDIEntityID, pStats);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn getUDPStats(
    TcpFile: HANDLE,
    pStats: *mut MIB_UDPSTATS,
) -> u32 {
    let ent = TDIEntityID {
        tei_entity: CL_TL_ENTITY,
        tei_instance: 0,
    };

    let status = tdiGetMibForUdpEntity(TcpFile, core::ptr::addr_of!(ent), pStats);

    if !NT_SUCCESS(status) {
        return RtlNtStatusToDosError(status);
    }

    NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn getNumRoutes() -> u32 {
    let mut numEntities: u32 = 0;
    let mut numRoutes: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return 0;
    }

    status = tdiGetEntityIDSet(tcpFile, core::ptr::addr_of_mut!(entitySet), core::ptr::addr_of_mut!(numEntities));
    if !NT_SUCCESS(status) {
        closeTcpFile(tcpFile);
        return 0;
    }

    for i in 0..numEntities as usize {
        if isIpEntity(tcpFile, entitySet.add(i)) != 0 {
            let mut ipstats: MIB_IPSTATS = core::mem::zeroed();
            memset(core::ptr::addr_of_mut!(ipstats) as *mut core::ffi::c_void, 0, core::mem::size_of::<MIB_IPSTATS>());
            status = tdiGetMibForIpEntity(tcpFile, entitySet.add(i), core::ptr::addr_of_mut!(ipstats));
            if !NT_SUCCESS(status) {
                numRoutes = 0;
                break;
            }
            numRoutes += ipstats.dwNumRoutes;
        }
    }

    tdiFreeThingSet((entitySet as *mut c_void));
    closeTcpFile(tcpFile);

    numRoutes
}
#[no_mangle]
pub unsafe extern "C" fn HexDump(Data: PCHAR, Len: DWORD) {
    for i in 0..Len {
        if i & 0xf == 0 {
            if i != 0 {
                fprintf(stderr(), b"\n\0".as_ptr() as *const i8);
            }
            fprintf(stderr(), b"%08x:\0".as_ptr() as *const i8, i);
        }
        let byte = *(Data as *const u8).offset(i as isize) & 0xffu8;
        fprintf(stderr(), b" %02x\0".as_ptr() as *const i8, byte as u32);
    }
    fprintf(stderr(), b"\n\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn getRouteTable() -> *mut RouteTable {
    let total_num_routes = getNumRoutes();
    let mut tcp_file: HANDLE = core::ptr::null_mut();
    let status = openTcpFile(&mut tcp_file, FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return core::ptr::null_mut();
    }
    let heap = GetProcessHeap();
    let alloc_size = if total_num_routes > 0 {
        core::mem::size_of::<RouteTable>() + core::mem::size_of::<RouteEntry>() * (total_num_routes as usize - 1)
    } else {
        core::mem::size_of::<RouteTable>()
    };
    let out_route_table = HeapAlloc(heap, 0, ((alloc_size) as SIZE_T)) as *mut RouteTable;
    if out_route_table.is_null() {
        closeTcpFile(tcp_file);
        return core::ptr::null_mut();
    }
    (*out_route_table).numRoutes = total_num_routes;

    let mut routes_added: u32 = 0;
    let mut i: u32 = 0;
    while routes_added < (*out_route_table).numRoutes {
        let mut ent: TDIEntityID = core::mem::zeroed();
        getNthIpEntity(tcp_file, i, &mut ent);

        let mut route_set: *mut IPRouteEntry = core::ptr::null_mut();
        let mut num_routes_entity: u32 = 0;
        tdiGetRoutesForIpEntity(tcp_file, &mut ent, &mut route_set, &mut num_routes_entity);
        if route_set.is_null() {
            closeTcpFile(tcp_file);
            HeapFree(heap, 0, out_route_table as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }

        let routes_ptr = core::ptr::addr_of_mut!((*out_route_table).routes) as *mut RouteEntry;
        for j in 0..num_routes_entity {
            let route_num = j as usize + routes_added as usize;
            let dest = routes_ptr.add(route_num);
            let route_entry = &*route_set.offset(j as isize);
            (*dest).dest = route_entry.ire_dest;
            (*dest).mask = route_entry.ire_mask;
            (*dest).gateway = route_entry.ire_gw;
            (*dest).ifIndex = route_entry.ire_index;
            (*dest).metric = route_entry.ire_metric1;
        }

        if !route_set.is_null() {
            tdiFreeThingSet((route_set as *mut c_void));
        }

        routes_added += num_routes_entity;
        i += 1;
    }

    closeTcpFile(tcp_file);
    out_route_table
}
#[no_mangle]
pub unsafe extern "C" fn getNumArpEntries() -> u32 {
    let mut numEntities: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut totalNumber: u32 = 0;
    let mut status: NTSTATUS;
    let mut IpArpTable: *mut MIB_IPNETROW = core::ptr::null_mut();
    let mut returnSize: u32 = 0;

    status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return 0;
    }

    status = tdiGetEntityIDSet(tcpFile, core::ptr::addr_of_mut!(entitySet), core::ptr::addr_of_mut!(numEntities));

    i = 0;
    while i < numEntities {
        let entity_ptr = entitySet.add(i as usize);
        if isInterface(entity_ptr) != 0 && hasArp(tcpFile, entity_ptr) != 0 {
            status = tdiGetSetOfThings(
                tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                IP_MIB_ARPTABLE_ENTRY_ID,
                AT_ENTITY,
                (*entity_ptr).tei_instance,
                0,
                core::mem::size_of::<MIB_IPNETROW>() as u32,
                core::ptr::addr_of_mut!(IpArpTable) as *mut PVOID,
                &mut returnSize,
            );

            if status == STATUS_SUCCESS {
                totalNumber += returnSize;
            }
            if !IpArpTable.is_null() {
                tdiFreeThingSet(IpArpTable as PVOID);
                IpArpTable = core::ptr::null_mut();
            }
        }
        i += 1;
    }

    closeTcpFile(tcpFile);
    if !entitySet.is_null() {
        tdiFreeThingSet(entitySet as PVOID);
    }
    totalNumber
}
#[no_mangle]
pub unsafe extern "C" fn getArpTable() -> PMIB_IPNETTABLE {
    let mut numEntities: u32 = 0;
    let mut returnSize: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let totalNumber: u32 = getNumArpEntries();
    let mut status: NTSTATUS;
    let IpArpTable: *mut MIB_IPNETTABLE;
    let mut AdapterArpTable: *mut MIB_IPNETROW;
    let mut CurrIdx: i32 = 0;

    status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return core::ptr::null_mut();
    }

    IpArpTable = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<u32>() + core::mem::size_of::<MIB_IPNETROW>() * totalNumber as usize) as SIZE_T,
    ) as *mut MIB_IPNETTABLE;
    if IpArpTable.is_null() {
        closeTcpFile(tcpFile);
        return core::ptr::null_mut();
    }

    status = tdiGetEntityIDSet(tcpFile, core::ptr::addr_of_mut!(entitySet), core::ptr::addr_of_mut!(numEntities));

    for i in 0..numEntities as i32 {
        if isInterface(entitySet.add(i as usize)) != 0
            && hasArp(tcpFile, entitySet.add(i as usize)) != 0
        {
            AdapterArpTable = core::ptr::null_mut();
            status = tdiGetSetOfThings(
                tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                IP_MIB_ARPTABLE_ENTRY_ID,
                AT_ENTITY,
                (*entitySet.add(i as usize)).tei_instance,
                0,
                core::mem::size_of::<MIB_IPNETROW>() as u32,
                core::ptr::addr_of_mut!(AdapterArpTable) as *mut *mut c_void,
                core::ptr::addr_of_mut!(returnSize),
            );
            if status == STATUS_SUCCESS {
                for TmpIdx in 0..returnSize as i32 {
                    core::ptr::copy_nonoverlapping(
                        AdapterArpTable.add(TmpIdx as usize),
                        (*IpArpTable).table.as_mut_ptr().add(CurrIdx as usize),
                        1,
                    );
                    CurrIdx += 1;
                }
                tdiFreeThingSet(AdapterArpTable as *mut c_void);
            }
        }
    }

    closeTcpFile(tcpFile);
    tdiFreeThingSet(entitySet as *mut c_void);
    (*IpArpTable).dwNumEntries = CurrIdx as u32;
    IpArpTable
}
#[no_mangle]
pub unsafe extern "C" fn getNumUdpEntries() -> u32 {
    let mut tcpFile: *mut core::ffi::c_void = core::ptr::null_mut();
    let status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return 0;
    }

    let mut numEntities: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let status2 = tdiGetEntityIDSet(tcpFile, core::ptr::addr_of_mut!(entitySet), core::ptr::addr_of_mut!(numEntities));

    let mut totalNumber: u32 = 0;

    for i in 0..numEntities {
        let entity = &*entitySet.wrapping_add(i as usize);
        if entity.tei_entity == CL_TL_ENTITY && hasArp(tcpFile, entitySet.wrapping_add(i as usize)) != 0 {
            let mut IpUdpTable: *mut MIB_UDPROW = core::ptr::null_mut();
            let mut returnSize: u32 = 0;

            let status3 = tdiGetSetOfThings(
                tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                IP_MIB_ARPTABLE_ENTRY_ID,
                CL_TL_ENTITY,
                entity.tei_instance,
                0,
                core::mem::size_of::<MIB_UDPROW>() as u32,
                core::ptr::addr_of_mut!(IpUdpTable) as *mut *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(returnSize),
            );

            if status3 == STATUS_SUCCESS {
                totalNumber += returnSize;
            }

            if !IpUdpTable.is_null() {
                tdiFreeThingSet((IpUdpTable as *mut c_void));
                IpUdpTable = core::ptr::null_mut();
            }
        }
    }

    closeTcpFile(tcpFile);
    if !entitySet.is_null() {
        tdiFreeThingSet((entitySet as *mut c_void));
    }

    totalNumber
}
#[no_mangle]
pub unsafe extern "C" fn getUdpTable(Class: usize) -> *mut core::ffi::c_void {
    let totalNumber = getNumUdpEntries();

    let mut tcpFile: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut status: i32;
    status = openTcpFile(&mut tcpFile as *mut *mut core::ffi::c_void, FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return core::ptr::null_mut();
    }

    let call = &UdpTcpTableCall[Class];
    let allocSize = call.UdpOffset as usize + (call.UdpSize as usize) * (totalNumber as usize);
    let IpUdpTable = HeapAlloc(GetProcessHeap(), 0, ((allocSize) as SIZE_T)) as *mut MIB_UDPTABLE;
    if IpUdpTable.is_null() {
        closeTcpFile(tcpFile);
        return core::ptr::null_mut();
    }

    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut numEntities: u32 = 0;
    status = tdiGetEntityIDSet(
        tcpFile,
        core::ptr::addr_of_mut!(entitySet),
        core::ptr::addr_of_mut!(numEntities),
    );

    let mut AdapterUdpTable: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut CurrIdx: u32 = 0;

    for i in 0..numEntities as usize {
        if (*entitySet.add(i)).tei_entity == CL_TL_ENTITY
            && hasArp(tcpFile, entitySet.add(i)) != 0
        {
            let mut returnSize: u32 = 0;
            status = tdiGetSetOfThings(
                tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                call.TOIID,
                CL_TL_ENTITY,
                (*entitySet.add(i)).tei_instance,
                0,
                (call.UdpSize as u32),
                core::ptr::addr_of_mut!(AdapterUdpTable),
                core::ptr::addr_of_mut!(returnSize),
            );

            if status == STATUS_SUCCESS {
                for TmpIdx in 0..returnSize as usize {
                    let dst = (IpUdpTable as usize
                        + call.UdpOffset as usize
                        + (call.UdpSize as usize) * (CurrIdx as usize))
                        as *mut u8;
                    let src = (AdapterUdpTable as usize
                        + (call.UdpSize as usize) * TmpIdx)
                        as *const u8;
                    core::ptr::copy_nonoverlapping(
                        src,
                        dst,
                        call.UdpSize as usize,
                    );
                    CurrIdx += 1;
                }
                tdiFreeThingSet(AdapterUdpTable);
                AdapterUdpTable = core::ptr::null_mut();
            }
        }
    }

    closeTcpFile(tcpFile);
    tdiFreeThingSet((entitySet as *mut c_void));
    (*(IpUdpTable as *mut u32)) = CurrIdx; // dwNumEntries is the first field
    IpUdpTable as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "C" fn getNumTcpEntries() -> u32 {
    let mut numEntities: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut i: u32;
    let mut totalNumber: u32 = 0;
    let mut status: NTSTATUS;
    let mut IpTcpTable: *mut MIB_TCPROW = core::ptr::null_mut();
    let mut returnSize: u32 = 0;

    status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return 0;
    }

    status = tdiGetEntityIDSet(tcpFile, core::ptr::addr_of_mut!(entitySet), core::ptr::addr_of_mut!(numEntities));

    for i in 0..numEntities {
        let entity = entitySet.add(i as usize);
        if (*entity).tei_entity == CO_TL_ENTITY && hasArp(tcpFile, entity) != 0 {
            status = tdiGetSetOfThings(
                tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                IP_MIB_ARPTABLE_ENTRY_ID,
                CO_TL_ENTITY,
                (*entity).tei_instance,
                0,
                core::mem::size_of::<MIB_TCPROW>() as u32,
                core::ptr::addr_of_mut!(IpTcpTable) as *mut *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(returnSize),
            );

            if status == STATUS_SUCCESS {
                totalNumber += returnSize;
            }
            if !IpTcpTable.is_null() {
                tdiFreeThingSet(IpTcpTable as *mut core::ffi::c_void);
                IpTcpTable = core::ptr::null_mut();
            }
        }
    }

    closeTcpFile(tcpFile);
    if !entitySet.is_null() {
        tdiFreeThingSet(entitySet as *mut core::ffi::c_void);
    }
    totalNumber
}
#[no_mangle]
pub unsafe extern "C" fn getTcpTable(Class: u32) -> *mut core::ffi::c_void {
    let mut numEntities: u32 = 0;
    let mut returnSize: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut totalNumber: u32;
    let mut TmpIdx: u32;
    let mut CurrIdx: u32 = 0;
    let mut status: NTSTATUS;
    let mut IpTcpTable: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut AdapterTcpTable: *mut core::ffi::c_void = core::ptr::null_mut();

    totalNumber = getNumTcpEntries();

    status = openTcpFile(&raw mut tcpFile, FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return core::ptr::null_mut();
    }

    IpTcpTable = HeapAlloc(GetProcessHeap(), 0,
        ((UdpTcpTableCall[Class as usize].TcpOffset
            + UdpTcpTableCall[Class as usize].TcpSize * totalNumber as usize) as SIZE_T));
    if IpTcpTable.is_null() {
        closeTcpFile(tcpFile);
        return core::ptr::null_mut();
    }

    status = tdiGetEntityIDSet(tcpFile, &raw mut entitySet, &raw mut numEntities);

    i = 0;
    while i < numEntities {
        if (*(entitySet.add(i as usize))).tei_entity == CO_TL_ENTITY && hasArp(tcpFile, entitySet.add(i as usize)) != 0 {
            status = tdiGetSetOfThings(tcpFile,
                INFO_CLASS_PROTOCOL,
                INFO_TYPE_PROVIDER,
                UdpTcpTableCall[Class as usize].TOIID,
                CO_TL_ENTITY,
                (*(entitySet.add(i as usize))).tei_instance,
                0,
                (UdpTcpTableCall[Class as usize].TcpSize as u32),
                &raw mut AdapterTcpTable,
                &raw mut returnSize);

            if status == STATUS_SUCCESS {
                TmpIdx = 0;
                while TmpIdx < returnSize {
                    CopyMemory(Add2Ptr(IpTcpTable, UdpTcpTableCall[Class as usize].TcpOffset + UdpTcpTableCall[Class as usize].TcpSize * CurrIdx as usize),
                               Add2Ptr(AdapterTcpTable, UdpTcpTableCall[Class as usize].TcpSize * TmpIdx as usize),
                               UdpTcpTableCall[Class as usize].TcpSize);
                    TmpIdx += 1;
                    CurrIdx += 1;
                }
                tdiFreeThingSet(AdapterTcpTable);
            }
        }
        i += 1;
    }

    closeTcpFile(tcpFile);
    tdiFreeThingSet((entitySet as *mut c_void));
    (*(IpTcpTable as *mut MIB_TCPTABLE)).dwNumEntries = CurrIdx;
    IpTcpTable
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (C) 2003 Art Yerkes
//  * A reimplementation of ifenum.c by Juan Lang
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  *
//  * This file is implemented on the IOCTL_TCP_QUERY_INFORMATION_EX ioctl on
//  * tcpip.sys
//  */
//
// #include "iphlpapi_private.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi);
//
// #ifndef TCPS_ESTABLISHED
// # define TCPS_ESTABLISHED TCP_ESTABLISHED
// #endif
// #ifndef TCPS_SYN_SENT
// # define TCPS_SYN_SENT TCP_SYN_SENT
// #endif
// #ifndef TCPS_SYN_RECEIVED
// # define TCPS_SYN_RECEIVED TCP_SYN_RECV
// #endif
// #ifndef TCPS_FIN_WAIT_1
// # define TCPS_FIN_WAIT_1 TCP_FIN_WAIT1
// #endif
// #ifndef TCPS_FIN_WAIT_2
// # define TCPS_FIN_WAIT_2 TCP_FIN_WAIT2
// #endif
// #ifndef TCPS_TIME_WAIT
// # define TCPS_TIME_WAIT TCP_TIME_WAIT
// #endif
// #ifndef TCPS_CLOSED
// # define TCPS_CLOSED TCP_CLOSE
// #endif
// #ifndef TCPS_CLOSE_WAIT
// # define TCPS_CLOSE_WAIT TCP_CLOSE_WAIT
// #endif
// #ifndef TCPS_LAST_ACK
// # define TCPS_LAST_ACK TCP_LAST_ACK
// #endif
// #ifndef TCPS_LISTEN
// # define TCPS_LISTEN TCP_LISTEN
// #endif
// #ifndef TCPS_CLOSING
// # define TCPS_CLOSING TCP_CLOSING
// #endif
//
// BOOL isIpEntity( HANDLE tcpFile, TDIEntityID *ent ) {
//     return (ent->tei_entity == CL_NL_ENTITY ||
//             ent->tei_entity == CO_NL_ENTITY);
// }
//
// NTSTATUS getNthIpEntity( HANDLE tcpFile, DWORD index, TDIEntityID *ent ) {
//     DWORD numEntities = 0;
//     DWORD numRoutes = 0;
//     TDIEntityID *entitySet = 0;
//     NTSTATUS status = tdiGetEntityIDSet( tcpFile, &entitySet, &numEntities );
//     int i;
//
//     if( !NT_SUCCESS(status) )
//         return status;
//
//     for( i = 0; i < numEntities; i++ ) {
//         if( isIpEntity( tcpFile, &entitySet[i] ) ) {
//             TRACE!("Entity %d is an IP Entity\n", i);
//             if( numRoutes == index ) break;
//             else numRoutes++;
//         }
//     }
//
//     if( numRoutes == index && i < numEntities ) {
//         TRACE!("Index %lu is entity #%d - %04x:%08x\n", index, i,
//               entitySet[i].tei_entity, entitySet[i].tei_instance);
//         memcpy( ent, &entitySet[i], sizeof(*ent) );
//         tdiFreeThingSet( entitySet );
//         return STATUS_SUCCESS;
//     } else {
//         tdiFreeThingSet( entitySet );
//         return STATUS_UNSUCCESSFUL;
//     }
// }
//
// static NTSTATUS
// tdiGetMibForIpEntity(
//    HANDLE tcpFile,
//    TDIEntityID *ent,
//    MIB_IPSTATS *entry)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX req = TCP_REQUEST_QUERY_INFORMATION_INIT;
//     NTSTATUS Status;
//     DWORD returnSize;
//
//     ZeroMemory(entry, sizeof(*entry));
//
//     TRACE!("TdiGetMibForIpEntity(tcpFile 0x%p, entityId 0x%x)\n",
//           tcpFile, ent->tei_instance);
//
//     req.ID.toi_class                = INFO_CLASS_PROTOCOL;
//     req.ID.toi_type                 = INFO_TYPE_PROVIDER;
//     req.ID.toi_id                   = IP_MIB_STATS_ID;
//     req.ID.toi_entity               = *ent;
//
//     Status = DeviceIoControl(tcpFile,
//                              IOCTL_TCP_QUERY_INFORMATION_EX,
//                              &req,
//                              sizeof(req),
//                              entry,
//                              sizeof(*entry),
//                              &returnSize,
//                              NULL);
//
//     TRACE!("TdiGetMibForIpEntity() => status = 0x%08lx, entry = {\n"
//            "  ipsi_forwarding ............ %lu\n"
//            "  ipsi_defaultttl ............ %lu\n"
//            "  ipsi_inreceives ............ %lu\n"
//            "  ipsi_indelivers ............ %lu\n"
//            "  ipsi_outrequests ........... %lu\n"
//            "  ipsi_routingdiscards ....... %lu\n"
//            "  ipsi_outdiscards ........... %lu\n"
//            "  ipsi_outnoroutes ........... %lu\n"
//            "  ipsi_numif ................. %lu\n"
//            "  ipsi_numaddr ............... %lu\n"
//            "  ipsi_numroutes ............. %lu\n"
//            "}\n",
//           Status,
//           entry->dwForwarding,
//           entry->dwDefaultTTL,
//           entry->dwInReceives,
//           entry->dwInDelivers,
//           entry->dwOutRequests,
//           entry->dwRoutingDiscards,
//           entry->dwOutDiscards,
//           entry->dwOutNoRoutes,
//           entry->dwNumIf,
//           entry->dwNumAddr,
//           entry->dwNumRoutes);
//
//     return Status;
// }
//
// static NTSTATUS
// tdiGetMibForTcpEntity(
//    HANDLE tcpFile,
//    TDIEntityID *ent,
//    MIB_TCPSTATS *entry)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX req = TCP_REQUEST_QUERY_INFORMATION_INIT;
//     NTSTATUS Status;
//     DWORD returnSize;
//
//     ZeroMemory(entry, sizeof(*entry));
//
//     TRACE!("TdiGetMibForTcpEntity(tcpFile 0x%p, entityId 0x%x)\n",
//           tcpFile, ent->tei_instance);
//
//     req.ID.toi_class                = INFO_CLASS_PROTOCOL;
//     req.ID.toi_type                 = INFO_TYPE_PROVIDER;
//     req.ID.toi_id                   = TCP_MIB_STAT_ID;
//     req.ID.toi_entity               = *ent;
//
//     Status = DeviceIoControl(tcpFile,
//                              IOCTL_TCP_QUERY_INFORMATION_EX,
//                              &req,
//                              sizeof(req),
//                              entry,
//                              sizeof(*entry),
//                              &returnSize,
//                              NULL);
//
//     TRACE!("TdiGetMibForTcpEntity() => status = 0x%08lx, entry = {\n"
//            "  dwInSegs ............. %lu\n"
//            "  dwOutSegs ............ %lu\n"
//            "  dwNumConns ........... %lu\n"
//            "}\n",
//           Status,
//           entry->dwInSegs,
//           entry->dwOutSegs,
//           entry->dwNumConns);
//
//     return Status;
// }
//
// static NTSTATUS
// tdiGetMibForUdpEntity(
//    HANDLE tcpFile,
//    TDIEntityID *ent,
//    MIB_UDPSTATS *entry)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX req = TCP_REQUEST_QUERY_INFORMATION_INIT;
//     NTSTATUS Status;
//     DWORD returnSize;
//
//     ZeroMemory(entry, sizeof(*entry));
//
//     TRACE!("TdiGetMibForUdpEntity(tcpFile 0x%p, entityId 0x%x)\n",
//           tcpFile, ent->tei_instance);
//
//     req.ID.toi_class                = INFO_CLASS_PROTOCOL;
//     req.ID.toi_type                 = INFO_TYPE_PROVIDER;
//     req.ID.toi_id                   = UDP_MIB_STAT_ID;
//     req.ID.toi_entity               = *ent;
//
//     Status = DeviceIoControl(tcpFile,
//                              IOCTL_TCP_QUERY_INFORMATION_EX,
//                              &req,
//                              sizeof(req),
//                              entry,
//                              sizeof(*entry),
//                              &returnSize,
//                              NULL);
//
//     TRACE!("TdiGetMibForUdpEntity() => status = 0x%08lx, entry = {\n"
//            "  dwInDatagrams ....... %lu\n"
//            "  dwOutDatagrams ...... %lu\n"
//            "  dwNumAddrs .......... %lu\n"
//            "}\n",
//           Status,
//           entry->dwInDatagrams,
//           entry->dwOutDatagrams,
//           entry->dwNumAddrs);
//
//     return Status;
// }
//
// NTSTATUS tdiGetRoutesForIpEntity
// ( HANDLE tcpFile, TDIEntityID *ent, IPRouteEntry **routes, PDWORD numRoutes ) {
//     NTSTATUS status = STATUS_SUCCESS;
//
//     TRACE!("TdiGetRoutesForIpEntity(tcpFile 0x%p, entityId 0x%x)\n",
//           tcpFile, ent->tei_instance);
//
//     status = tdiGetSetOfThings(tcpFile,
//                                INFO_CLASS_PROTOCOL,
//                                INFO_TYPE_PROVIDER,
//                                IP_MIB_ARPTABLE_ENTRY_ID,
//                                CL_NL_ENTITY,
//                                ent->tei_instance,
//                                0,
//                                sizeof(IPRouteEntry),
//                                (PVOID *)routes,
//                                numRoutes);
//
//     return status;
// }
//
// NTSTATUS tdiGetIpAddrsForIpEntity
// ( HANDLE tcpFile, TDIEntityID *ent, IPAddrEntry **addrs, PDWORD numAddrs ) {
//     NTSTATUS status;
//
//     TRACE!("TdiGetIpAddrsForIpEntity(tcpFile 0x%p, entityId 0x%x)\n",
//           tcpFile, ent->tei_instance);
//
//     status = tdiGetSetOfThings(tcpFile,
//                                INFO_CLASS_PROTOCOL,
//                                INFO_TYPE_PROVIDER,
//                                IP_MIB_ADDRTABLE_ENTRY_ID,
//                                CL_NL_ENTITY,
//                                ent->tei_instance,
//                                0,
//                                sizeof(IPAddrEntry),
//                                (PVOID *)addrs,
//                                numAddrs);
//
//     return status;
// }
//
// DWORD getInterfaceStatsByName(const char *name, PMIB_IFROW entry)
// {
//   if (!name)
//     return ERROR_INVALID_PARAMETER;
//   if (!entry)
//     return ERROR_INVALID_PARAMETER;
//
//   return NO_ERROR;
// }
//
// DWORD getInterfaceStatsByIndex(DWORD index, PMIB_IFROW entry)
// {
//     return ERROR_INVALID_PARAMETER;
// }
//
// DWORD getICMPStats(MIB_ICMP *stats)
// {
//   FILE *fp;
//
//   if (!stats)
//     return ERROR_INVALID_PARAMETER;
//
//   memset(stats, 0, sizeof(MIB_ICMP));
//   /* get most of these stats from /proc/net/snmp, no error if can't */
//   fp = fopen("/proc/net/snmp", "r");
//   if (fp) {
//     const char hdr[] = "Icmp:";
//     char buf[512] = { 0 }, *ptr;
//
//     do {
//       ptr = fgets(buf, sizeof(buf), fp);
//     } while (ptr && _strnicmp(buf, hdr, sizeof(hdr) - 1));
//     if (ptr) {
//       /* last line was a header, get another */
//       ptr = fgets(buf, sizeof(buf), fp);
//       if (ptr && _strnicmp(buf, hdr, sizeof(hdr) - 1) == 0) {
//         char *endPtr;
//
//         ptr += sizeof(hdr);
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwMsgs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwErrors = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwDestUnreachs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwTimeExcds = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwParmProbs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwSrcQuenchs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwRedirects = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwEchoReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwTimestamps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwTimestampReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwAddrMasks = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpInStats.dwAddrMaskReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwMsgs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwErrors = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwDestUnreachs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwTimeExcds = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwParmProbs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwSrcQuenchs = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwRedirects = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwEchoReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwTimestamps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwTimestampReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwAddrMasks = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//         if (ptr && *ptr) {
//           stats->stats.icmpOutStats.dwAddrMaskReps = strtoul(ptr, &endPtr, 10);
//           ptr = endPtr;
//         }
//       }
//     }
//     fclose(fp);
//   }
//   return NO_ERROR;
// }
//
// DWORD getIPStats(
//      HANDLE TcpFile,
//      PMIB_IPSTATS pStats)
// {
//     TDIEntityID ent;
//     NTSTATUS Status;
//
//     ent.tei_entity = CL_NL_ENTITY;
//     ent.tei_instance = 0;
//
//     Status = tdiGetMibForIpEntity(TcpFile, &ent, pStats);
//
//     if (!NT_SUCCESS(Status))
//         return RtlNtStatusToDosError(Status);
//
//     return NO_ERROR;
// }
//
// DWORD getTCPStats(
//      HANDLE TcpFile,
//      MIB_TCPSTATS *pStats)
// {
//     TDIEntityID ent;
//     NTSTATUS Status;
//
//     ent.tei_entity = CO_TL_ENTITY;
//     ent.tei_instance = 0;
//
//     Status = tdiGetMibForTcpEntity(TcpFile, &ent, pStats);
//
//     if (!NT_SUCCESS(Status))
//         return RtlNtStatusToDosError(Status);
//
//     return NO_ERROR;
// }
//
// DWORD getUDPStats(
//      HANDLE TcpFile,
//      MIB_UDPSTATS *pStats)
// {
//     TDIEntityID ent;
//     NTSTATUS Status;
//
//     ent.tei_entity = CL_TL_ENTITY;
//     ent.tei_instance = 0;
//
//     Status = tdiGetMibForUdpEntity(TcpFile, &ent, pStats);
//
//     if (!NT_SUCCESS(Status))
//         return RtlNtStatusToDosError(Status);
//
//     return NO_ERROR;
// }
//
// DWORD getNumRoutes(void)
// {
//     DWORD numEntities, numRoutes = 0;
//     TDIEntityID *entitySet;
//     HANDLE tcpFile;
//     int i;
//     NTSTATUS status;
//
//     TRACE!("called.\n");
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//     if (!NT_SUCCESS(status)) {
//         ERR!("tdiGetEntityIDSet returned 0x%08lx\n", status);
//         closeTcpFile( tcpFile );
//         return 0;
//     }
//
//     for (i = 0; i < numEntities; i++) {
//         if (isIpEntity(tcpFile, &entitySet[i])) {
//             MIB_IPSTATS ipstats;
//             memset(&ipstats, 0, sizeof(ipstats));
//             status = tdiGetMibForIpEntity(tcpFile, &entitySet[i], &ipstats);
//             if (!NT_SUCCESS(status)) {
//                 ERR!("tdiGetMibForIpEntity returned 0x%08lx, for i = %d\n", status, i);
//                 numRoutes = 0;
//                 break;
//             }
//             numRoutes += ipstats.dwNumRoutes;
//         }
//     }
//
//     TRACE!("numRoutes = %lu\n", numRoutes);
//
//     tdiFreeThingSet(entitySet);
//     closeTcpFile(tcpFile);
//
//     return numRoutes;
// }
//
// VOID HexDump( PCHAR Data, DWORD Len ) {
//     int i;
//
//     for( i = 0; i < Len; i++ ) {
//         if( !(i & 0xf) ) {
//             if( i ) fprintf(stderr,"\n");
//             fprintf(stderr,"%08x:", i);
//         }
//         fprintf( stderr, " %02x", Data[i] & 0xff );
//     }
//     fprintf(stderr,"\n");
// }
//
// RouteTable *getRouteTable(void)
// {
//     RouteTable *out_route_table;
//     DWORD numRoutes = getNumRoutes(), routesAdded = 0;
//     TDIEntityID ent;
//     HANDLE tcpFile;
//     NTSTATUS status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     int i;
//
//     if (!NT_SUCCESS(status))
//         return 0;
//
//     TRACE!("GETTING ROUTE TABLE\n");
//
//     out_route_table = HeapAlloc(GetProcessHeap(), 0,
//                                 sizeof(RouteTable) +
//                                 (sizeof(RouteEntry) * (numRoutes - 1)));
//     if (!out_route_table) {
//         closeTcpFile(tcpFile);
//         return NULL;
//     }
//
//     out_route_table->numRoutes = numRoutes;
//
//     for (i = 0; routesAdded < out_route_table->numRoutes; i++) {
//         int j;
//         IPRouteEntry *route_set;
//
//         getNthIpEntity(tcpFile, i, &ent);
//
//         tdiGetRoutesForIpEntity(tcpFile, &ent, &route_set, &numRoutes);
//         if (!route_set) {
//             closeTcpFile(tcpFile);
//             HeapFree(GetProcessHeap(), 0, out_route_table);
//             return 0;
//         }
//
//         TRACE!("%lu routes in instance %d\n", numRoutes, i);
//
//         for (j = 0; j < numRoutes; j++) {
//             int routeNum = j + routesAdded;
//             out_route_table->routes[routeNum].dest =
//                 route_set[j].ire_dest;
//             out_route_table->routes[routeNum].mask =
//                 route_set[j].ire_mask;
//             out_route_table->routes[routeNum].gateway =
//                 route_set[j].ire_gw;
//             out_route_table->routes[routeNum].ifIndex =
//                 route_set[j].ire_index;
//             out_route_table->routes[routeNum].metric =
//                 route_set[j].ire_metric1;
//         }
//
//         if (route_set) tdiFreeThingSet(route_set);
//
//         routesAdded += numRoutes;
//     }
//
//     closeTcpFile(tcpFile);
//     TRACE!("status = 0x%08lx, out_route_table = 0x%p\n", status, out_route_table);
//     return out_route_table;
// }
//
// DWORD getNumArpEntries(void)
// {
//     DWORD numEntities;
//     TDIEntityID *entitySet = NULL;
//     HANDLE tcpFile;
//     int i, totalNumber = 0;
//     NTSTATUS status;
//     PMIB_IPNETROW IpArpTable = NULL;
//     DWORD returnSize;
//
//     TRACE!("called.\n");
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (isInterface(&entitySet[i]) && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 IP_MIB_ARPTABLE_ENTRY_ID,
//                 AT_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 sizeof(MIB_IPNETROW),
//                 (PVOID *)&IpArpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) totalNumber += returnSize;
//             if (IpArpTable) {
//                 tdiFreeThingSet(IpArpTable);
//                 IpArpTable = NULL;
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     if (entitySet) tdiFreeThingSet(entitySet);
//     return totalNumber;
// }
//
// PMIB_IPNETTABLE getArpTable(void)
// {
//     DWORD numEntities, returnSize;
//     TDIEntityID *entitySet;
//     HANDLE tcpFile;
//     int i, totalNumber, TmpIdx, CurrIdx = 0;
//     NTSTATUS status;
//     PMIB_IPNETTABLE IpArpTable = NULL;
//     PMIB_IPNETROW AdapterArpTable = NULL;
//
//     TRACE!("called.\n");
//
//     totalNumber = getNumArpEntries();
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     IpArpTable = HeapAlloc(GetProcessHeap(), 0,
//         sizeof(DWORD) + (sizeof(MIB_IPNETROW) * totalNumber));
//     if (!IpArpTable) {
//         closeTcpFile(tcpFile);
//         return NULL;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (isInterface(&entitySet[i]) && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 IP_MIB_ARPTABLE_ENTRY_ID,
//                 AT_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 sizeof(MIB_IPNETROW),
//                 (PVOID *)&AdapterArpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) {
//                 for (TmpIdx = 0; TmpIdx < returnSize; TmpIdx++, CurrIdx++)
//                     IpArpTable->table[CurrIdx] = AdapterArpTable[TmpIdx];
//                 tdiFreeThingSet(AdapterArpTable);
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     tdiFreeThingSet(entitySet);
//     IpArpTable->dwNumEntries = CurrIdx;
//     return IpArpTable;
// }
//
// struct _TABLE_CALL
// {
//     DWORD TOIID;
//     SIZE_T UdpSize;
//     SIZE_T TcpSize;
//     SIZE_T UdpOffset;
//     SIZE_T TcpOffset;
// } UdpTcpTableCall[] = {
//     {IP_MIB_ARPTABLE_ENTRY_ID, sizeof(MIB_UDPROW), sizeof(MIB_TCPROW), FIELD_OFFSET(MIB_UDPTABLE, table), FIELD_OFFSET(MIB_TCPTABLE, table)},
//     {IP_MIB_ADDRTABLE_ENTRY_ID, sizeof(MIB_UDPROW_OWNER_PID), sizeof(MIB_TCPROW_OWNER_PID), FIELD_OFFSET(MIB_UDPTABLE_OWNER_PID, table), FIELD_OFFSET(MIB_TCPTABLE_OWNER_PID, table)},
//     {IP_SPECIFIC_MODULE_ENTRY_ID, sizeof(MIB_UDPROW_OWNER_MODULE), sizeof(MIB_TCPROW_OWNER_MODULE), FIELD_OFFSET(MIB_UDPTABLE_OWNER_MODULE, table), FIELD_OFFSET(MIB_TCPTABLE_OWNER_MODULE, table)},
// };
//
// #define Add2Ptr(PTR, INC) (PVOID)((ULONG_PTR)(PTR) + (INC))
//
// DWORD getNumUdpEntries(void)
// {
//     DWORD numEntities;
//     TDIEntityID *entitySet = NULL;
//     HANDLE tcpFile;
//     int i, totalNumber = 0;
//     NTSTATUS status;
//     PMIB_UDPROW IpUdpTable = NULL;
//     DWORD returnSize;
//
//     TRACE!("called.\n");
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (entitySet[i].tei_entity == CL_TL_ENTITY && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 IP_MIB_ARPTABLE_ENTRY_ID,
//                 CL_TL_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 sizeof(MIB_UDPROW),
//                 (PVOID *)&IpUdpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) totalNumber += returnSize;
//             if (IpUdpTable) {
//                 tdiFreeThingSet(IpUdpTable);
//                 IpUdpTable = NULL;
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     if (entitySet) tdiFreeThingSet(entitySet);
//     return totalNumber;
// }
//
// PVOID getUdpTable(CLASS_TABLE Class)
// {
//     DWORD numEntities, returnSize;
//     TDIEntityID *entitySet;
//     HANDLE tcpFile;
//     int i, totalNumber, TmpIdx, CurrIdx = 0;
//     NTSTATUS status;
//     PMIB_UDPTABLE IpUdpTable = NULL;
//     PVOID AdapterUdpTable = NULL;
//
//     TRACE!("called.\n");
//
//     totalNumber = getNumUdpEntries();
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     IpUdpTable = HeapAlloc(GetProcessHeap(), 0,
//         UdpTcpTableCall[Class].UdpOffset + (UdpTcpTableCall[Class].UdpSize * totalNumber));
//     if (!IpUdpTable) {
//         closeTcpFile(tcpFile);
//         return NULL;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (entitySet[i].tei_entity == CL_TL_ENTITY && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 UdpTcpTableCall[Class].TOIID,
//                 CL_TL_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 UdpTcpTableCall[Class].UdpSize,
//                 &AdapterUdpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) {
//                 for (TmpIdx = 0; TmpIdx < returnSize; TmpIdx++, CurrIdx++)
//                     CopyMemory(Add2Ptr(IpUdpTable, UdpTcpTableCall[Class].UdpOffset + UdpTcpTableCall[Class].UdpSize * CurrIdx),
//                                Add2Ptr(AdapterUdpTable, UdpTcpTableCall[Class].UdpSize * TmpIdx),
//                                UdpTcpTableCall[Class].UdpSize);
//                 tdiFreeThingSet(AdapterUdpTable);
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     tdiFreeThingSet(entitySet);
//     IpUdpTable->dwNumEntries = CurrIdx;
//     return IpUdpTable;
// }
//
// DWORD getNumTcpEntries(void)
// {
//     DWORD numEntities;
//     TDIEntityID *entitySet = NULL;
//     HANDLE tcpFile;
//     int i, totalNumber = 0;
//     NTSTATUS status;
//     PMIB_TCPROW IpTcpTable = NULL;
//     DWORD returnSize;
//
//     TRACE!("called.\n");
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (entitySet[i].tei_entity == CO_TL_ENTITY && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 IP_MIB_ARPTABLE_ENTRY_ID,
//                 CO_TL_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 sizeof(MIB_TCPROW),
//                 (PVOID *)&IpTcpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) totalNumber += returnSize;
//             if (IpTcpTable) {
//                 tdiFreeThingSet(IpTcpTable);
//                 IpTcpTable = NULL;
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     if (entitySet) tdiFreeThingSet(entitySet);
//     return totalNumber;
// }
//
// PVOID getTcpTable(CLASS_TABLE Class)
// {
//     DWORD numEntities, returnSize;
//     TDIEntityID *entitySet;
//     HANDLE tcpFile;
//     int i, totalNumber, TmpIdx, CurrIdx = 0;
//     NTSTATUS status;
//     PMIB_TCPTABLE IpTcpTable = NULL;
//     PVOID AdapterTcpTable = NULL;
//
//     TRACE!("called.\n");
//
//     totalNumber = getNumTcpEntries();
//
//     status = openTcpFile(&tcpFile, FILE_READ_DATA);
//     if (!NT_SUCCESS(status))
//     {
//         ERR!("openTcpFile returned 0x%08lx\n", status);
//         return 0;
//     }
//
//     IpTcpTable = HeapAlloc(GetProcessHeap(), 0,
//         UdpTcpTableCall[Class].TcpOffset + (UdpTcpTableCall[Class].TcpSize * totalNumber));
//     if (!IpTcpTable) {
//         closeTcpFile(tcpFile);
//         return NULL;
//     }
//
//     status = tdiGetEntityIDSet(tcpFile, &entitySet, &numEntities);
//
//     for (i = 0; i < numEntities; i++) {
//         if (entitySet[i].tei_entity == CO_TL_ENTITY && hasArp(tcpFile, &entitySet[i]))
//         {
//             status = tdiGetSetOfThings(tcpFile,
//                 INFO_CLASS_PROTOCOL,
//                 INFO_TYPE_PROVIDER,
//                 UdpTcpTableCall[Class].TOIID,
//                 CO_TL_ENTITY,
//                 entitySet[i].tei_instance,
//                 0,
//                 UdpTcpTableCall[Class].TcpSize,
//                 &AdapterTcpTable,
//                 &returnSize);
//
//             if (status == STATUS_SUCCESS) {
//                 for (TmpIdx = 0; TmpIdx < returnSize; TmpIdx++, CurrIdx++)
//                     CopyMemory(Add2Ptr(IpTcpTable, UdpTcpTableCall[Class].TcpOffset + UdpTcpTableCall[Class].TcpSize * CurrIdx),
//                                Add2Ptr(AdapterTcpTable, UdpTcpTableCall[Class].TcpSize * TmpIdx),
//                                UdpTcpTableCall[Class].TcpSize);
//                 tdiFreeThingSet(AdapterTcpTable);
//             }
//         }
//     }
//
//     closeTcpFile(tcpFile);
//     tdiFreeThingSet(entitySet);
//     IpTcpTable->dwNumEntries = CurrIdx;
//     return IpTcpTable;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
