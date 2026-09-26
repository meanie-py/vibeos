use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut __wine_dbch_iphlpapi: __wine_debug_channel = __wine_debug_channel {
    flags: 0xFFu8,
    name: unsafe { core::mem::transmute(*b"iphlpapi\0\0\0\0\0\0\0") },
};
#[no_mangle]
pub unsafe extern "C" fn interfaceMapInit() {}
#[no_mangle]
pub unsafe extern "C" fn interfaceMapFree() {
    // Ditto.
}
#[no_mangle]
pub unsafe extern "C" fn tdiGetMibForIfEntity(
    tcpFile: HANDLE,
    ent: *mut _TDIEntityID,
    entry: *mut _IFEntrySafelySized,
) -> NTSTATUS {
    let mut req: _TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut return_size: u32 = 0;

    (*&mut req).ID.toi_class = INFO_CLASS_PROTOCOL;
    (*&mut req).ID.toi_type = INFO_TYPE_PROVIDER;
    (*&mut req).ID.toi_id = IF_MIB_STATS_ID;
    (*&mut req).ID.toi_entity = core::ptr::read(ent);

    let b_result: BOOL = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &mut req as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<_TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        entry as *mut core::ffi::c_void,
        core::mem::size_of::<_IFEntrySafelySized>() as u32,
        &mut return_size as *mut u32,
        core::ptr::null_mut(),
    );

    if b_result == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn isInterface(if_maybe: *mut TDIEntityID) -> BOOL {
    ((*if_maybe).tei_entity == IF_ENTITY) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn isLoopback(tcpFile: HANDLE, loop_maybe: *mut _TDIEntityID) -> i32 {
    let mut entryInfo: _IFEntrySafelySized = core::mem::zeroed();
    let status = tdiGetMibForIfEntity(
        tcpFile,
        loop_maybe,
        core::ptr::addr_of_mut!(entryInfo),
    );
    (NT_SUCCESS(status) && (entryInfo.ent.as_ref().if_type == IFENT_SOFTWARE_LOOPBACK)) as i32
}
#[no_mangle]
pub unsafe extern "C" fn hasArp(tcpFile: HANDLE, arp_maybe: *mut TDIEntityID) -> BOOL {
    let mut req: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut returnSize: u32 = 0;
    let mut type_: u32 = 0;

    req.ID.toi_class = INFO_CLASS_GENERIC;
    req.ID.toi_type = INFO_TYPE_PROVIDER;
    req.ID.toi_id = ENTITY_TYPE_ID;
    req.ID.toi_entity.tei_entity = AT_ENTITY;
    req.ID.toi_entity.tei_instance = (*arp_maybe).tei_instance;

    status = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX,
        &req as *const _ as *mut c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as u32,
        &mut type_ as *mut _ as *mut c_void,
        core::mem::size_of::<u32>() as u32,
        &mut returnSize,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(status) {
        return 0;
    }

    (type_ == AT_ARP) as i32
}
unsafe extern "C" fn getInterfaceInfoSet(
    tcpFile: HANDLE,
    infoSet: *mut *mut IFInfo,
    numInterfaces: *mut u32,
) -> NTSTATUS {
    let mut numEntities: u32 = 0;
    let mut entIDSet: *mut TDIEntityID = core::ptr::null_mut();
    let mut status = tdiGetEntityIDSet(tcpFile, &mut entIDSet, &mut numEntities);

    if !NT_SUCCESS(status) {
        return status;
    }

    let infoSetInt = HeapAlloc(
        GetProcessHeap(),
        0,
        ((core::mem::size_of::<IFInfo>() * numEntities as usize) as SIZE_T),
    ) as *mut IFInfo;

    if infoSetInt.is_null() {
        tdiFreeThingSet((entIDSet as *mut c_void));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut curInterf: i32 = 0;

    for i in 0..numEntities as usize {
        if isInterface(entIDSet.add(i)) != 0 {
            (*infoSetInt.add(curInterf as usize)).entity_id = core::ptr::read(entIDSet.add(i));

            status = tdiGetMibForIfEntity(
                tcpFile,
                entIDSet.add(i),
                &mut (*infoSetInt.add(curInterf as usize)).if_info,
            );

            if NT_SUCCESS(status) {
                let mut numAddrs: u32 = 0;
                let mut addrs: *mut IPAddrEntry = core::ptr::null_mut();
                let mut ip_ent: TDIEntityID = core::mem::zeroed();

                status = getNthIpEntity(tcpFile, (curInterf as u32), &mut ip_ent);
                if NT_SUCCESS(status) {
                    status = tdiGetIpAddrsForIpEntity(tcpFile, &mut ip_ent, &mut addrs, &mut numAddrs);
                }

                let mut j: u32 = 0;
                while NT_SUCCESS(status) && j < numAddrs {
                    if (*addrs.add(j as usize)).iae_index
                        == (*infoSetInt.add(curInterf as usize)).if_info.ent.as_ref().if_index
                    {
                        core::ptr::copy_nonoverlapping(
                            addrs.add(j as usize) as *const u8,
                            &mut (*infoSetInt.add(curInterf as usize)).ip_addr as *mut _ as *mut u8,
                            core::mem::size_of::<IPAddrEntry>(),
                        );
                        curInterf += 1;
                        break;
                    }
                    j += 1;
                }

                if NT_SUCCESS(status) {
                    tdiFreeThingSet((addrs as *mut c_void));
                }
            }
        }
    }

    tdiFreeThingSet((entIDSet as *mut c_void));

    if NT_SUCCESS(status) {
        *infoSet = infoSetInt;
        *numInterfaces = curInterf as u32;
    } else {
        HeapFree(GetProcessHeap(), 0, (infoSetInt as *mut c_void));
    }

    status
}
unsafe extern "C" fn getNumInterfacesInt(onlyNonLoopback: BOOL) -> DWORD {
    let mut numInterfaces: DWORD = 0;
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut numEntities: DWORD = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut status: NTSTATUS;

    status = openTcpFile(&mut tcpFile, FILE_READ_DATA);
    if !NT_SUCCESS(status) {
        return 0;
    }

    status = tdiGetEntityIDSet(tcpFile, &mut entitySet, &mut numEntities);
    if !NT_SUCCESS(status) {
        closeTcpFile(tcpFile);
        return 0;
    }

    for i in 0..numEntities as usize {
        let current_entity = entitySet.add(i);
        if isInterface(current_entity) != 0
            && (onlyNonLoopback == 0 || isLoopback(tcpFile, current_entity) == 0)
        {
            numInterfaces += 1;
        }
    }

    closeTcpFile(tcpFile);
    tdiFreeThingSet((entitySet as *mut c_void));
    numInterfaces
}
#[no_mangle]
pub unsafe extern "C" fn getNumInterfaces() -> DWORD {
    getNumInterfacesInt((0u8 as i32))
}
#[no_mangle]
pub unsafe extern "C" fn getNumNonLoopbackInterfaces() -> DWORD {
    getNumInterfacesInt((1u8 as i32))
}
#[no_mangle]
pub unsafe extern "C" fn getNthInterfaceEntity(tcpFile: HANDLE, index: u32, ent: *mut TDIEntityID) -> u32 {
    let mut numEntities: u32 = 0;
    let mut numInterfaces: u32 = 0;
    let mut entitySet: *mut TDIEntityID = core::ptr::null_mut();
    let mut i: i32;

    let status = tdiGetEntityIDSet(tcpFile, &mut entitySet as *mut *mut TDIEntityID, &mut numEntities);
    if !NT_SUCCESS(status) {
        return status as u32;
    }

    i = 0;
    while (i as u32) < numEntities {
        if isInterface(&mut (*entitySet.offset(i as isize)) as *mut TDIEntityID) != 0 {
            if numInterfaces == index {
                break;
            } else {
                numInterfaces += 1;
            }
        }
        i += 1;
    }

    if numInterfaces == index && (i as u32) < numEntities {
        core::ptr::copy_nonoverlapping(
            &(*entitySet.offset(i as isize)) as *const TDIEntityID,
            ent,
            1,
        );
        tdiFreeThingSet((entitySet as *mut c_void));
        (STATUS_SUCCESS as u32)
    } else {
        tdiFreeThingSet((entitySet as *mut c_void));
        (STATUS_UNSUCCESSFUL as u32)
    }
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceInfoByIndex(tcpFile: HANDLE, index: u32, info: *mut IFInfo) -> NTSTATUS {
    let mut ifInfo: *mut IFInfo = core::ptr::null_mut();
    let mut numInterfaces: u32 = 0;
    let status = getInterfaceInfoSet(
        tcpFile,
        core::ptr::addr_of_mut!(ifInfo),
        core::ptr::addr_of_mut!(numInterfaces),
    );

    if NT_SUCCESS(status) {
        let mut i: u32 = 0;
        while i < numInterfaces {
            if (*ifInfo.offset(i as isize)).if_info.ent.as_ref().if_index == index {
                memcpy(
                    info as *mut core::ffi::c_void,
                    ifInfo.offset(i as isize) as *const core::ffi::c_void,
                    core::mem::size_of::<IFInfo>(),
                );
                break;
            }
            i += 1;
        }

        HeapFree(GetProcessHeap(), 0, ifInfo as *mut core::ffi::c_void);

        if i < numInterfaces {
            STATUS_SUCCESS
        } else {
            STATUS_UNSUCCESSFUL
        }
    } else {
        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceInfoByName(
    tcpFile: HANDLE,
    name: *mut core::ffi::c_char,
    info: *mut IFInfo,
) -> NTSTATUS {
    let mut ifInfo: *mut IFInfo = core::ptr::null_mut();
    let mut numInterfaces: u32 = 0;
    let status = getInterfaceInfoSet(
        tcpFile,
        core::ptr::addr_of_mut!(ifInfo),
        core::ptr::addr_of_mut!(numInterfaces),
    );

    if NT_SUCCESS(status) {
        let mut i = 0;
        for j in 0..numInterfaces as usize {
            i = j;
            let if_entry = &(*ifInfo.add(i)).if_info.ent;
            if strncmp(
                if_entry.as_ref().if_descr.as_ptr() as *const i8,
                name as *const i8,
                if_entry.as_ref().if_descrlen as usize,
            ) == 0
            {
                memcpy(
                    info as *mut core::ffi::c_void,
                    ifInfo.add(i) as *const core::ffi::c_void,
                    core::mem::size_of::<IFInfo>(),
                );
                break;
            }
        }

        HeapFree(GetProcessHeap(), 0, ifInfo as *mut core::ffi::c_void);

        if (i as u32) < numInterfaces {
            STATUS_SUCCESS
        } else {
            STATUS_UNSUCCESSFUL
        }
    } else {
        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceNameByIndex(index: u32) -> PCHAR {
    let mut ifInfo: IFInfo = core::mem::zeroed();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut interfaceName: PCHAR = core::ptr::null_mut();

    let status = openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA);
    if NT_SUCCESS(status) {
        let status = getInterfaceInfoByIndex(tcpFile, index, core::ptr::addr_of_mut!(ifInfo));
        if NT_SUCCESS(status) {
            let descr_len = ifInfo.if_info.ent.as_ref().if_descrlen as usize;
            let alloc_size = descr_len + 1;
            interfaceName = HeapAlloc(GetProcessHeap(), 0, alloc_size as SIZE_T) as PCHAR;
            if !interfaceName.is_null() {
                let src = core::ptr::addr_of!(ifInfo.if_info.ent.as_ref().if_descr) as *const u8;
                core::ptr::copy_nonoverlapping(src, interfaceName as *mut u8, descr_len);
                *interfaceName.add(descr_len) = 0;
            }
        }
        closeTcpFile(tcpFile);
    }
    interfaceName
}
#[no_mangle]
pub unsafe extern "C" fn consumeInterfaceName(name: *const u8) {
    HeapFree(
        GetProcessHeap(),
        0,
        name as *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceIndexByName(name: *const core::ffi::c_char, index: *mut DWORD) -> NTSTATUS {
    let mut ifInfo: IFInfo = core::mem::zeroed();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut status = openTcpFile(&mut tcpFile, FILE_READ_DATA);
    if NT_SUCCESS(status) {
        status = getInterfaceInfoByName(tcpFile, name as *mut core::ffi::c_char, &mut ifInfo);
        if NT_SUCCESS(status) {
            *index = ifInfo.if_info.ent.as_ref().if_index;
        }
        closeTcpFile(tcpFile);
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceIndexTableInt(nonLoopbackOnly: i32) -> *mut InterfaceIndexTable {
    let mut numInterfaces: u32 = 0;
    let mut curInterface: u32 = 0;
    let mut ifInfo: *mut IFInfo = core::ptr::null_mut();
    let mut ret: *mut InterfaceIndexTable = core::ptr::null_mut();
    let mut tcpFile: HANDLE = core::ptr::null_mut();

    let status: NTSTATUS = openTcpFile(&mut tcpFile as *mut HANDLE, FILE_READ_DATA);
    if NT_SUCCESS(status) {
        let status2 = getInterfaceInfoSet(
            tcpFile,
            &mut ifInfo as *mut *mut IFInfo,
            &mut numInterfaces as *mut u32,
        );
        if NT_SUCCESS(status2) {
            let size = core::mem::size_of::<InterfaceIndexTable>()
                + (numInterfaces.wrapping_sub(1) as usize) * core::mem::size_of::<u32>();
            ret = calloc(1, size) as *mut InterfaceIndexTable;
            if !ret.is_null() {
                (*ret).numAllocated = numInterfaces;
                for i in 0..numInterfaces as usize {
                    if nonLoopbackOnly == 0
                        || isLoopback(
                            tcpFile,
                            core::ptr::addr_of_mut!((*ifInfo.offset((i as isize))).entity_id),
                        ) == 0
                    {
                        // `indexes` is a C FLEXIBLE ARRAY MEMBER, which bindgen renders as
                        // `[DWORD; 1]`; the table is HeapAlloc'd with room for numInterfaces
                        // entries. Rust `[i]` would bounds-check-PANIC for every i >= 1 (i.e.
                        // on any machine with two interfaces) — go through the pointer.
                        *(*ret).indexes.as_mut_ptr().add(curInterface as usize) =
                            (*ifInfo.offset((i as isize))).if_info.ent.as_ref().if_index;
                        curInterface += 1;
                    }
                }
                (*ret).numIndexes = curInterface;
            }
            tdiFreeThingSet((ifInfo as *mut c_void));
        }
        closeTcpFile(tcpFile);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceIndexTable() -> *mut InterfaceIndexTable {
    getInterfaceIndexTableInt(0)
}
#[no_mangle]
pub unsafe extern "C" fn getNonLoopbackInterfaceIndexTable() -> *mut InterfaceIndexTable {
    getInterfaceIndexTableInt(1i32)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceIPAddrByName(name: *const i8) -> u32 {
    0u32
}
#[no_mangle]
pub unsafe extern "C" fn getIPAddrEntryForIf(
    tcpFile: HANDLE,
    name: *mut core::ffi::c_char,
    index: u32,
    ifInfo: *mut IFInfo,
) -> NTSTATUS {
    let status = if !name.is_null() {
        getInterfaceInfoByName(tcpFile, name, ifInfo)
    } else {
        getInterfaceInfoByIndex(tcpFile, index, ifInfo)
    };

    if !NT_SUCCESS(status) {
        // empty
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn getAddrByIndexOrName(name: *mut core::ffi::c_char, index: u32, addrType: IPHLPAddrType) -> u32 {
    let mut ifInfo: IFInfo = core::mem::zeroed();
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut status = STATUS_SUCCESS;
    let mut addrOut: u32 = INADDR_ANY as u32;

    status = unsafe { openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA) };

    if NT_SUCCESS(status) {
        status = unsafe { getIPAddrEntryForIf(tcpFile, name, index, core::ptr::addr_of_mut!(ifInfo)) };
        if NT_SUCCESS(status) {
            match addrType {
                IPHLPAddrType_IPAAddr => addrOut = (*core::ptr::addr_of!(ifInfo.ip_addr)).iae_addr,
                IPHLPAddrType_IPABcast => addrOut = (*core::ptr::addr_of!(ifInfo.ip_addr)).iae_bcastaddr,
                IPHLPAddrType_IPAMask => addrOut = (*core::ptr::addr_of!(ifInfo.ip_addr)).iae_mask,
                IPHLPAddrType_IFMtu => addrOut = (*core::ptr::addr_of!(ifInfo.if_info.ent)).as_ref().if_mtu,
                IPHLPAddrType_IFStatus => addrOut = (*core::ptr::addr_of!(ifInfo.if_info.ent)).as_ref().if_operstatus,
                _ => {}
            }
        }
        unsafe { closeTcpFile(tcpFile) };
    }

    addrOut
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceIPAddrByIndex(index: u32) -> u32 {
    getAddrByIndexOrName(core::ptr::null_mut(), index, IPAAddr)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceBCastAddrByName(name: *const core::ffi::c_char) -> u32 {
    getAddrByIndexOrName(name as *mut core::ffi::c_char, 0, IPABcast)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceBCastAddrByIndex(index: u32) -> u32 {
    getAddrByIndexOrName(core::ptr::null_mut(), index, _IPHLPAddrType_IPABcast)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceMaskByName(name: *const core::ffi::c_char) -> u32 {
    getAddrByIndexOrName(name as *mut core::ffi::c_char, 0, _IPHLPAddrType_IPAMask)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceMaskByIndex(index: u32) -> u32 {
    getAddrByIndexOrName(core::ptr::null_mut(), index, IPHLPAddrType_IPAMask)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfacePhysicalFromInfo(
    info: *mut IFInfo,
    len: *mut DWORD,
    addr: *mut u8,
    type_: *mut DWORD,
) {
    *len = (*info).if_info.ent.as_ref().if_physaddrlen;
    memcpy(
        addr as *mut core::ffi::c_void,
        &(*info).if_info.ent.as_ref().if_physaddr as *const _ as *const core::ffi::c_void,
        *len as usize,
    );
    *type_ = (*info).if_info.ent.as_ref().if_type;
}
#[no_mangle]
pub unsafe extern "C" fn getInterfacePhysicalByName(
    name: *const core::ffi::c_char,
    len: *mut u32,
    addr: *mut u8,
    type_: *mut u32,
) -> u32 {
    let mut tcpFile: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut info: IFInfo = core::mem::zeroed();
    let status = openTcpFile(&mut tcpFile, FILE_READ_DATA) as u32;

    if NT_SUCCESS(status as i32) {
        let status = getInterfaceInfoByName(tcpFile, name as *mut core::ffi::c_char, &mut info) as u32;
        if NT_SUCCESS(status as i32) {
            getInterfacePhysicalFromInfo(&mut info, len, addr, type_);
        }
        closeTcpFile(tcpFile);
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn getInterfacePhysicalByIndex(
    index: u32,
    len: *mut u32,
    addr: *mut u8,
    type_: *mut u32,
) -> NTSTATUS {
    let mut tcp_file: HANDLE = core::ptr::null_mut();
    let mut info: IFInfo = core::mem::zeroed();
    let mut status = openTcpFile(core::ptr::addr_of_mut!(tcp_file), FILE_READ_DATA);
    if NT_SUCCESS(status) {
        status = getInterfaceInfoByIndex(tcp_file, index, &mut info);
        if NT_SUCCESS(status) {
            getInterfacePhysicalFromInfo(&mut info, len, addr, type_);
        }
        closeTcpFile(tcp_file);
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceMtuByName(name: *const core::ffi::c_char, mtu: *mut u32) -> u32 {
    *mtu = getAddrByIndexOrName(name as *mut core::ffi::c_char, 0, _IPHLP_ADDR_TYPE_IFMtu);
    (STATUS_SUCCESS as u32)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceMtuByIndex(index: u32, mtu: *mut u32) -> u32 {
    *mtu = getAddrByIndexOrName(core::ptr::null_mut(), index, IPHLPAddrType_IFMtu);
    (STATUS_SUCCESS as u32)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceStatusByName(name: *const core::ffi::c_char, status: *mut u32) -> u32 {
    *status = getAddrByIndexOrName(name as *mut core::ffi::c_char, 0, _IPHLPAddrType_IFStatus);
    (STATUS_SUCCESS as u32)
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceStatusByIndex(index: u32, status: *mut u32) -> u32 {
    *status = getAddrByIndexOrName(core::ptr::null_mut(), index, IFStatus);
    STATUS_SUCCESS as u32
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceEntryByName(name: *const core::ffi::c_char, entry: PMIB_IFROW) -> u32 {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut info: IFInfo = core::mem::zeroed();
    let mut status = openTcpFile(&mut tcpFile, FILE_READ_DATA);
    if NT_SUCCESS(status) {
        status = getInterfaceInfoByName(tcpFile, name as *mut core::ffi::c_char, &mut info);
        if NT_SUCCESS(status) {
            core::ptr::copy_nonoverlapping(
                &info.if_info as *const _ as *const u8,
                // ifenum_reactos.c:595/620 `&entry->wszName[MAX_INTERFACE_NAME_LEN]` — a ONE-PAST-THE-END
                // address, legal in C, used to reach the if_info that follows the name array inside
                // the containing struct. Indexing a [WCHAR; 256] at 256 in Rust is out of bounds
                // (`unconditional_panic`), so take the address by pointer arithmetic instead.
                (*entry).wszName.as_mut_ptr().add(MAX_INTERFACE_NAME_LEN) as *mut u8,
                core::mem::size_of_val(&info.if_info),
            );
        }
        closeTcpFile(tcpFile);
    }
    status as u32
}
#[no_mangle]
pub unsafe extern "C" fn getInterfaceEntryByIndex(index: u32, entry: PMIB_IFROW) -> u32 {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut info: IFInfo = core::mem::zeroed();
    let status = openTcpFile(&mut tcpFile, FILE_READ_DATA);
    if NT_SUCCESS(status) {
        let status = getInterfaceInfoByIndex(tcpFile, index, &mut info);
        if NT_SUCCESS(status) {
            // one-past-the-end address (see above) — pointer arithmetic, not indexing.
            let dst = (*entry).wszName.as_mut_ptr().add(MAX_INTERFACE_NAME_LEN);
            let src = core::ptr::addr_of_mut!(info.if_info);
            let size = core::mem::size_of_val(&info.if_info);
            memcpy(dst as *mut core::ffi::c_void, src as *const core::ffi::c_void, size);
        }
        closeTcpFile(tcpFile);
    }
    status as u32
}
#[no_mangle]
pub unsafe extern "C" fn toIPAddressString(addr: u32, string: *mut core::ffi::c_char) -> *mut core::ffi::c_char {
    let mut iAddr: in_addr = core::mem::zeroed();
    iAddr.S_un.S_addr = addr;

    if !string.is_null() {
        strncpy(string, inet_ntoa(iAddr), 16usize);
    }

    inet_ntoa(iAddr)
}
#[no_mangle]
pub unsafe extern "C" fn addIPAddress(Address: u32, Mask: u32, IfIndex: u32, NteContext: *mut u32, NteInstance: *mut u32) -> NTSTATUS {
    #[repr(C)]
    struct IP_SET_DATA {
        NteContext: u32,
        NewAddress: u32,
        NewNetmask: u32,
    }

    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut data: IP_SET_DATA = core::mem::zeroed();

    let status = openTcpFile(&mut tcpFile, FILE_READ_DATA | FILE_WRITE_DATA);
    if !NT_SUCCESS(status) {
        return status;
    }

    data.NteContext = IfIndex;
    data.NewAddress = Address;
    data.NewNetmask = Mask;

    let status = NtDeviceIoControlFile(
        tcpFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut iosb as *mut _IO_STATUS_BLOCK,
        IOCTL_SET_IP_ADDRESS,
        &mut data as *mut IP_SET_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<IP_SET_DATA>() as u32,
        &mut data as *mut IP_SET_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<IP_SET_DATA>() as u32,
    );

    closeTcpFile(tcpFile);

    if NT_SUCCESS(status) {
        *NteContext = iosb.Information as u32;
        *NteInstance = data.NewAddress;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn deleteIpAddress(nte_context: u32) -> NTSTATUS {
    let mut tcp_file: HANDLE = core::ptr::null_mut();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    let status = openTcpFile(core::ptr::addr_of_mut!(tcp_file), FILE_READ_DATA | FILE_WRITE_DATA);
    if !NT_SUCCESS(status) {
        return status;
    }

    let status = NtDeviceIoControlFile(
        tcp_file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(iosb),
        IOCTL_DELETE_IP_ADDRESS,
        core::ptr::addr_of!(nte_context) as *mut core::ffi::c_void,
        2u32,
        core::ptr::null_mut(),
        0u32,
    );

    closeTcpFile(tcp_file);
    status
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
//  * Implementation notes
//  * - Our bretheren use IOCTL_TCP_QUERY_INFORMATION_EX to get information
//  *   from tcpip.sys and IOCTL_TCP_SET_INFORMATION_EX to set info (such as
//  *   the route table).  These ioctls mirror WsControl exactly in usage.
//  * - This iphlpapi does not rely on any reactos-only features.
//  * - This implementation is meant to be largely correct.  I am not, however,
//  *   paying any attention to performance.  It can be done faster, and
//  *   someone should definately optimize this code when speed is more of a
//  *   priority than it is now.
//  *
//  * Edited implementation notes from the original -- Basically edited to add
//  *   information and prune things which are not accurate about this file.
//  * Interface index fun:
//  * - Windows may rely on an index being cleared in the topmost 8 bits in some
//  *   APIs; see GetFriendlyIfIndex and the mention of "backward compatible"
//  *   indexes.  It isn't clear which APIs might fail with non-backward-compatible
//  *   indexes, but I'll keep them bits clear just in case.
//  * FIXME:
//  * - We don't support IPv6 addresses here yet -- I moved the upper edge
//  *   functions into iphlpv6.c (arty)
//  */
// #include "iphlpapi_private.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi);
//
// /* Functions */
//
// /* I'm a bit skittish about maintaining this info in memory, as I'd rather
//  * not add any mutex or critical section blockers to these functions.  I've
//  * encountered far too many windows functions that contribute to deadlock
//  * by not announcing themselves. */
// void interfaceMapInit(void)
// {
//     /* For now, nothing */
// }
//
// void interfaceMapFree(void)
// {
//     /* Ditto. */
// }
//
// NTSTATUS tdiGetMibForIfEntity
// ( HANDLE tcpFile, TDIEntityID *ent, IFEntrySafelySized *entry ) {
//     TCP_REQUEST_QUERY_INFORMATION_EX req = TCP_REQUEST_QUERY_INFORMATION_INIT;
//     NTSTATUS status = STATUS_SUCCESS;
//     DWORD returnSize;
//
//     WARN!("TdiGetMibForIfEntity(tcpFile %p,entityId %p)\n",
//            tcpFile, ent->tei_instance);
//
//     req.ID.toi_class                = INFO_CLASS_PROTOCOL;
//     req.ID.toi_type                 = INFO_TYPE_PROVIDER;
//     req.ID.toi_id                   = IF_MIB_STATS_ID;
//     req.ID.toi_entity               = *ent;
//
//     status = DeviceIoControl( tcpFile,
//                               IOCTL_TCP_QUERY_INFORMATION_EX,
//                               &req,
//                               sizeof(req),
//                               entry,
//                               sizeof(*entry),
//                               &returnSize,
//                               NULL );
//
//     if(!status)
//     {
//             WARN!("IOCTL Failed\n");
//             return STATUS_UNSUCCESSFUL;
//     }
//
//     TRACE!("TdiGetMibForIfEntity() => {\n"
//            "  if_index ....................... %x\n"
//            "  if_type ........................ %x\n"
//            "  if_mtu ......................... %d\n"
//            "  if_speed ....................... %x\n"
//            "  if_physaddrlen ................. %d\n",
//            entry->ent.if_index,
//            entry->ent.if_type,
//            entry->ent.if_mtu,
//            entry->ent.if_speed,
//            entry->ent.if_physaddrlen);
//     TRACE!("  if_physaddr .................... %02x:%02x:%02x:%02x:%02x:%02x\n"
//            "  if_descr ....................... %*s\n",
//            entry->ent.if_physaddr[0] & 0xff,
//            entry->ent.if_physaddr[1] & 0xff,
//            entry->ent.if_physaddr[2] & 0xff,
//            entry->ent.if_physaddr[3] & 0xff,
//            entry->ent.if_physaddr[4] & 0xff,
//            entry->ent.if_physaddr[5] & 0xff,
//            entry->ent.if_descrlen,
//            entry->ent.if_descr);
//     TRACE!("} status %08x\n",status);
//
//     return STATUS_SUCCESS;
// }
//
// BOOL isInterface( TDIEntityID *if_maybe ) {
//     return
//         if_maybe->tei_entity == IF_ENTITY;
// }
//
// BOOL isLoopback( HANDLE tcpFile, TDIEntityID *loop_maybe ) {
//     IFEntrySafelySized entryInfo;
//     NTSTATUS status;
//
//     status = tdiGetMibForIfEntity( tcpFile,
//                                    loop_maybe,
//                                    &entryInfo );
//
//     return NT_SUCCESS(status) &&
//            (entryInfo.ent.if_type == IFENT_SOFTWARE_LOOPBACK);
// }
//
// BOOL hasArp( HANDLE tcpFile, TDIEntityID *arp_maybe ) {
//     TCP_REQUEST_QUERY_INFORMATION_EX req = TCP_REQUEST_QUERY_INFORMATION_INIT;
//     NTSTATUS status = STATUS_SUCCESS;
//     DWORD returnSize, type;
//
//     req.ID.toi_class                = INFO_CLASS_GENERIC;
//     req.ID.toi_type                 = INFO_TYPE_PROVIDER;
//     req.ID.toi_id                   = ENTITY_TYPE_ID;
//     req.ID.toi_entity.tei_entity    = AT_ENTITY;
//     req.ID.toi_entity.tei_instance  = arp_maybe->tei_instance;
//
//     status = DeviceIoControl( tcpFile,
//                               IOCTL_TCP_QUERY_INFORMATION_EX,
//                               &req,
//                               sizeof(req),
//                               &type,
//                               sizeof(type),
//                               &returnSize,
//                               NULL );
//     if( !NT_SUCCESS(status) ) return FALSE;
//
//     /* AT_ARP corresponds to an individual TDI entity type */
//     return (type == AT_ARP);
// }
//
// static NTSTATUS getInterfaceInfoSet( HANDLE tcpFile,
//                                      IFInfo **infoSet,
//                                      PDWORD numInterfaces ) {
//     DWORD numEntities;
//     TDIEntityID *entIDSet = NULL;
//     NTSTATUS status = tdiGetEntityIDSet( tcpFile, &entIDSet, &numEntities );
//     IFInfo *infoSetInt = 0;
//     int curInterf = 0, i;
//
//     if (!NT_SUCCESS(status)) {
//         ERR!("getInterfaceInfoSet: tdiGetEntityIDSet() failed: 0x%lx\n", status);
//         return status;
//     }
//
//     infoSetInt = HeapAlloc( GetProcessHeap(), 0,
//                             sizeof(IFInfo) * numEntities );
//
//     if( infoSetInt ) {
//         for( i = 0; i < numEntities; i++ ) {
//             if( isInterface( &entIDSet[i] ) ) {
//                 infoSetInt[curInterf].entity_id = entIDSet[i];
//                 status = tdiGetMibForIfEntity
//                     ( tcpFile,
//                       &entIDSet[i],
//                       &infoSetInt[curInterf].if_info );
//                 TRACE!("tdiGetMibForIfEntity: %08x\n", status);
//                 if( NT_SUCCESS(status) ) {
//                     DWORD numAddrs;
//                     IPAddrEntry *addrs;
//                     TDIEntityID ip_ent;
//                     int j;
//
//                     status = getNthIpEntity( tcpFile, curInterf, &ip_ent );
//                     if( NT_SUCCESS(status) )
//                         status = tdiGetIpAddrsForIpEntity
//                             ( tcpFile, &ip_ent, &addrs, &numAddrs );
//                     for( j = 0; NT_SUCCESS(status) && j < numAddrs; j++ ) {
//                         TRACE!("ADDR %d: index %d (target %d)\n", j, addrs[j].iae_index, infoSetInt[curInterf].if_info.ent.if_index);
//                         if( addrs[j].iae_index ==
//                             infoSetInt[curInterf].if_info.ent.if_index ) {
//                             memcpy( &infoSetInt[curInterf].ip_addr,
//                                     &addrs[j],
//                                     sizeof( addrs[j] ) );
//                             curInterf++;
//                             break;
//                         }
//                     }
//                     if ( NT_SUCCESS(status) )
//                         tdiFreeThingSet(addrs);
//                 }
//             }
//         }
//
//         tdiFreeThingSet(entIDSet);
//
//         if (NT_SUCCESS(status)) {
//             *infoSet = infoSetInt;
//             *numInterfaces = curInterf;
//         } else {
//             HeapFree(GetProcessHeap(), 0, infoSetInt);
//         }
//
//         return status;
//     } else {
//         tdiFreeThingSet(entIDSet);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
// }
//
// static DWORD getNumInterfacesInt(BOOL onlyNonLoopback)
// {
//     DWORD numEntities, numInterfaces = 0;
//     TDIEntityID *entitySet;
//     HANDLE tcpFile;
//     NTSTATUS status;
//     int i;
//
//     status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( !NT_SUCCESS(status) ) {
//         WARN!("getNumInterfaces: failed %08x\n", status );
//         return 0;
//     }
//
//     status = tdiGetEntityIDSet( tcpFile, &entitySet, &numEntities );
//
//     if( !NT_SUCCESS(status) ) {
//         WARN!("getNumInterfaces: failed %08x\n", status );
//         closeTcpFile( tcpFile );
//         return 0;
//     }
//
//     for( i = 0; i < numEntities; i++ ) {
//         if( isInterface( &entitySet[i] ) &&
//             (!onlyNonLoopback || !isLoopback( tcpFile, &entitySet[i] )) )
//             numInterfaces++;
//     }
//
//     TRACE!("getNumInterfaces: success: %d %d %08x\n",
//            onlyNonLoopback, numInterfaces, status );
//
//     closeTcpFile( tcpFile );
//
//     tdiFreeThingSet( entitySet );
//
//     return numInterfaces;
// }
//
// DWORD getNumInterfaces(void)
// {
//     return getNumInterfacesInt( FALSE );
// }
//
// DWORD getNumNonLoopbackInterfaces(void)
// {
//     return getNumInterfacesInt( TRUE );
// }
//
// DWORD getNthInterfaceEntity( HANDLE tcpFile, DWORD index, TDIEntityID *ent ) {
//     DWORD numEntities = 0;
//     DWORD numInterfaces = 0;
//     TDIEntityID *entitySet = 0;
//     NTSTATUS status = tdiGetEntityIDSet( tcpFile, &entitySet, &numEntities );
//     int i;
//
//     if( !NT_SUCCESS(status) )
//         return status;
//
//     for( i = 0; i < numEntities; i++ ) {
//         if( isInterface( &entitySet[i] ) ) {
//             if( numInterfaces == index ) break;
//             else numInterfaces++;
//         }
//     }
//
//     TRACE!("Index %d is entity #%d - %04x:%08x\n", index, i,
//            entitySet[i].tei_entity, entitySet[i].tei_instance );
//
//     if( numInterfaces == index && i < numEntities ) {
//         memcpy( ent, &entitySet[i], sizeof(*ent) );
//         tdiFreeThingSet( entitySet );
//         return STATUS_SUCCESS;
//     } else {
//         tdiFreeThingSet( entitySet );
//         return STATUS_UNSUCCESSFUL;
//     }
// }
//
// NTSTATUS getInterfaceInfoByIndex( HANDLE tcpFile, DWORD index, IFInfo *info ) {
//     IFInfo *ifInfo;
//     DWORD numInterfaces;
//     NTSTATUS status = getInterfaceInfoSet( tcpFile, &ifInfo, &numInterfaces );
//     int i;
//
//     if( NT_SUCCESS(status) )
//     {
//         for( i = 0; i < numInterfaces; i++ ) {
//             if( ifInfo[i].if_info.ent.if_index == index ) {
//                 memcpy( info, &ifInfo[i], sizeof(*info) );
//                 break;
//             }
//         }
//
//         HeapFree(GetProcessHeap(), 0, ifInfo);
//
//         return i < numInterfaces ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
//     }
//
//     return status;
// }
//
// NTSTATUS getInterfaceInfoByName( HANDLE tcpFile, char *name, IFInfo *info ) {
//     IFInfo *ifInfo;
//     DWORD numInterfaces;
//     int i;
//     NTSTATUS status = getInterfaceInfoSet( tcpFile, &ifInfo, &numInterfaces );
//
//     if( NT_SUCCESS(status) )
//     {
//         for( i = 0; i < numInterfaces; i++ ) {
//             if( !strncmp((PCHAR)ifInfo[i].if_info.ent.if_descr, name, ifInfo[i].if_info.ent.if_descrlen) ) {
//                 memcpy( info, &ifInfo[i], sizeof(*info) );
//                 break;
//             }
//         }
//
//         HeapFree(GetProcessHeap(), 0,ifInfo);
//
//         return i < numInterfaces ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
//     }
//
//     return status;
// }
//
// /* Note that the result of this operation must be freed later */
//
// const char *getInterfaceNameByIndex(DWORD index)
// {
//     IFInfo ifInfo;
//     HANDLE tcpFile;
//     char *interfaceName = NULL;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByIndex( tcpFile, index, &ifInfo );
//
//         if( NT_SUCCESS(status) ) {
//             interfaceName = HeapAlloc( GetProcessHeap(), 0,
//                                        ifInfo.if_info.ent.if_descrlen + 1 );
//             if( interfaceName ) {
//               memcpy(interfaceName, ifInfo.if_info.ent.if_descr, ifInfo.if_info.ent.if_descrlen);
//               interfaceName[ifInfo.if_info.ent.if_descrlen] = '\0';
//             }
//         }
//
//         closeTcpFile( tcpFile );
//     }
//
//     return interfaceName;
// }
//
// void consumeInterfaceName(const char *name) {
//     HeapFree( GetProcessHeap(), 0, (char *)name );
// }
//
// DWORD getInterfaceIndexByName(const char *name, PDWORD index)
// {
//     IFInfo ifInfo;
//     HANDLE tcpFile;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByName( tcpFile, (char *)name, &ifInfo );
//
//         if( NT_SUCCESS(status) ) {
//             *index = ifInfo.if_info.ent.if_index;
//         }
//
//         closeTcpFile( tcpFile );
//     }
//
//     return status;
// }
//
// InterfaceIndexTable *getInterfaceIndexTableInt( BOOL nonLoopbackOnly ) {
//   DWORD numInterfaces, curInterface = 0;
//   int i;
//   IFInfo *ifInfo;
//   InterfaceIndexTable *ret = 0;
//   HANDLE tcpFile;
//   NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//   if( NT_SUCCESS(status) ) {
//       status = getInterfaceInfoSet( tcpFile, &ifInfo, &numInterfaces );
//
//       TRACE!("InterfaceInfoSet: %08x, %04x:%08x\n",
//              status,
//              ifInfo->entity_id.tei_entity,
//              ifInfo->entity_id.tei_instance);
//
//       if( NT_SUCCESS(status) ) {
//           ret = (InterfaceIndexTable *)
//               calloc(1,
//                      sizeof(InterfaceIndexTable) +
//                      (numInterfaces - 1) * sizeof(DWORD));
//
//           if (ret) {
//               ret->numAllocated = numInterfaces;
//               TRACE!("NumInterfaces = %d\n", numInterfaces);
//
//               for( i = 0; i < numInterfaces; i++ ) {
//                   TRACE!("Examining interface %d\n", i);
//                   if( !nonLoopbackOnly ||
//                       !isLoopback( tcpFile, &ifInfo[i].entity_id ) ) {
//                       TRACE!("Interface %d matches (%d)\n", i, curInterface);
//                       ret->indexes[curInterface++] =
//                           ifInfo[i].if_info.ent.if_index;
//                   }
//               }
//
//               ret->numIndexes = curInterface;
//           }
//
//           tdiFreeThingSet( ifInfo );
//       }
//       closeTcpFile( tcpFile );
//   }
//
//   return ret;
// }
//
// InterfaceIndexTable *getInterfaceIndexTable(void) {
//     return getInterfaceIndexTableInt( FALSE );
// }
//
// InterfaceIndexTable *getNonLoopbackInterfaceIndexTable(void) {
//     return getInterfaceIndexTableInt( TRUE );
// }
//
// DWORD getInterfaceIPAddrByName(const char *name)
// {
//     return INADDR_ANY;
// }
//
// NTSTATUS getIPAddrEntryForIf(HANDLE tcpFile,
//                              char *name,
//                              DWORD index,
//                              IFInfo *ifInfo) {
//     NTSTATUS status =
//         name ?
//         getInterfaceInfoByName( tcpFile, name, ifInfo ) :
//         getInterfaceInfoByIndex( tcpFile, index, ifInfo );
//
//     if (!NT_SUCCESS(status)) {
//         ERR!("getIPAddrEntryForIf returning %lx\n", status);
//     }
//
//     return status;
// }
//
// DWORD getAddrByIndexOrName( char *name, DWORD index, IPHLPAddrType addrType ) {
//     IFInfo ifInfo;
//     HANDLE tcpFile;
//     NTSTATUS status = STATUS_SUCCESS;
//     DWORD addrOut = INADDR_ANY;
//
//     status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( NT_SUCCESS(status) ) {
//         status = getIPAddrEntryForIf( tcpFile, name, index, &ifInfo );
//         if( NT_SUCCESS(status) ) {
//             switch( addrType ) {
//             case IPAAddr:  addrOut = ifInfo.ip_addr.iae_addr; break;
//             case IPABcast: addrOut = ifInfo.ip_addr.iae_bcastaddr; break;
//             case IPAMask:  addrOut = ifInfo.ip_addr.iae_mask; break;
//             case IFMtu:    addrOut = ifInfo.if_info.ent.if_mtu; break;
//             case IFStatus: addrOut = ifInfo.if_info.ent.if_operstatus; break;
//             }
//         }
//         closeTcpFile( tcpFile );
//     }
//
//     return addrOut;
// }
//
// DWORD getInterfaceIPAddrByIndex(DWORD index) {
//     return getAddrByIndexOrName( 0, index, IPAAddr );
// }
//
// DWORD getInterfaceBCastAddrByName(const char *name) {
//     return getAddrByIndexOrName( (char *)name, 0, IPABcast );
// }
//
// DWORD getInterfaceBCastAddrByIndex(DWORD index) {
//     return getAddrByIndexOrName( 0, index, IPABcast );
// }
//
// DWORD getInterfaceMaskByName(const char *name) {
//     return getAddrByIndexOrName( (char *)name, 0, IPAMask );
// }
//
// DWORD getInterfaceMaskByIndex(DWORD index) {
//     return getAddrByIndexOrName( 0, index, IPAMask );
// }
//
// void getInterfacePhysicalFromInfo( IFInfo *info,
//                                    PDWORD len, PBYTE addr, PDWORD type ) {
//     *len = info->if_info.ent.if_physaddrlen;
//     memcpy( addr, info->if_info.ent.if_physaddr, *len );
//     *type = info->if_info.ent.if_type;
// }
//
// DWORD getInterfacePhysicalByName(const char *name, PDWORD len, PBYTE addr,
//                                  PDWORD type)
// {
//     HANDLE tcpFile;
//     IFInfo info;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByName( tcpFile, (char *)name, &info );
//         if( NT_SUCCESS(status) )
//             getInterfacePhysicalFromInfo( &info, len, addr, type );
//         closeTcpFile( tcpFile );
//     }
//
//     return status;
// }
//
// DWORD getInterfacePhysicalByIndex(DWORD index, PDWORD len, PBYTE addr,
//  PDWORD type)
// {
//     HANDLE tcpFile;
//     IFInfo info;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByIndex( tcpFile, index, &info );
//         if( NT_SUCCESS(status) )
//             getInterfacePhysicalFromInfo( &info, len, addr, type );
//         closeTcpFile( tcpFile );
//     }
//
//     return status;
// }
//
// DWORD getInterfaceMtuByName(const char *name, PDWORD mtu) {
//     *mtu = getAddrByIndexOrName( (char *)name, 0, IFMtu );
//     return STATUS_SUCCESS;
// }
//
// DWORD getInterfaceMtuByIndex(DWORD index, PDWORD mtu) {
//     *mtu = getAddrByIndexOrName( 0, index, IFMtu );
//     return STATUS_SUCCESS;
// }
//
// DWORD getInterfaceStatusByName(const char *name, PDWORD status) {
//     *status = getAddrByIndexOrName( (char *)name, 0, IFStatus );
//     return STATUS_SUCCESS;
// }
//
// DWORD getInterfaceStatusByIndex(DWORD index, PDWORD status)
// {
//     *status = getAddrByIndexOrName( 0, index, IFStatus );
//     return STATUS_SUCCESS;
// }
//
// DWORD getInterfaceEntryByName(const char *name, PMIB_IFROW entry)
// {
//     HANDLE tcpFile;
//     IFInfo info;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     TRACE!("Called.\n");
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByName( tcpFile, (char *)name, &info );
//
//         if( NT_SUCCESS(status) ) {
//             memcpy( &entry->wszName[MAX_INTERFACE_NAME_LEN],
//                     &info.if_info,
//                     sizeof(info.if_info) );
//         }
//
//         TRACE!("entry->bDescr = %s\n", entry->bDescr);
//
//         closeTcpFile( tcpFile );
//     }
//
//     return status;
// }
//
// DWORD getInterfaceEntryByIndex(DWORD index, PMIB_IFROW entry)
// {
//     HANDLE tcpFile;
//     IFInfo info;
//     NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA );
//
//     TRACE!("Called.\n");
//
//     if( NT_SUCCESS(status) ) {
//         status = getInterfaceInfoByIndex( tcpFile, index, &info );
//
//         if( NT_SUCCESS(status) ) {
//             memcpy( &entry->wszName[MAX_INTERFACE_NAME_LEN],
//                     &info.if_info,
//                     sizeof(info.if_info) );
//         }
//
//         closeTcpFile( tcpFile );
//     }
//
//     return status;
// }
//
// char *toIPAddressString(unsigned int addr, char string[16])
// {
//     struct in_addr iAddr;
//
//     iAddr.s_addr = addr;
//
//     if (string)
//         strncpy(string, inet_ntoa(iAddr), 16);
//
//     return inet_ntoa(iAddr);
// }
//
// NTSTATUS addIPAddress( IPAddr Address, IPMask Mask, DWORD IfIndex,
//                        PULONG NteContext, PULONG NteInstance )
// {
//   HANDLE tcpFile;
//   NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA | FILE_WRITE_DATA );
//   IP_SET_DATA Data;
//   IO_STATUS_BLOCK Iosb;
//
//   TRACE!("Called.\n");
//
//   if( !NT_SUCCESS(status) ) return status;
//
//   Data.NteContext = IfIndex;
//   Data.NewAddress = Address;
//   Data.NewNetmask = Mask;
//
//   status = NtDeviceIoControlFile( tcpFile,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   &Iosb,
//                                   IOCTL_SET_IP_ADDRESS,
//                                   &Data,
//                                   sizeof(Data),
//                                   &Data,
//                                   sizeof(Data) );
//
//   closeTcpFile( tcpFile );
//
//   if( NT_SUCCESS(status) ) {
//       *NteContext = Iosb.Information;
//       *NteInstance = Data.NewAddress;
//   }
//
//   if (!NT_SUCCESS(status)) {
//       ERR!("addIPAddress for if %d returning 0x%lx\n", IfIndex, status);
//   }
//
//   return status;
//
// }
//
// NTSTATUS deleteIpAddress( ULONG NteContext )
// {
//   HANDLE tcpFile;
//   NTSTATUS status = openTcpFile( &tcpFile, FILE_READ_DATA | FILE_WRITE_DATA );
//   IO_STATUS_BLOCK Iosb;
//
//   TRACE!("Called.\n");
//
//   if( !NT_SUCCESS(status) ) return status;
//
//   status = NtDeviceIoControlFile( tcpFile,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   &Iosb,
//                                   IOCTL_DELETE_IP_ADDRESS,
//                                   &NteContext,
//                                   sizeof(USHORT),
//                                   NULL,
//                                   0 );
//
//   closeTcpFile( tcpFile );
//
//   if (!NT_SUCCESS(status)) {
//       ERR!("deleteIpAddress(%lu) returning 0x%lx\n", NteContext, status);
//   }
//
//   return status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
