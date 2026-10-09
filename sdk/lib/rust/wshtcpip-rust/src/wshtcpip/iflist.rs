use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _IntfIDItem {
    pub entry: list,
    pub id: TDIEntityID,
    pub numaddr: i32,
    pub pIPAddrEntry0: *mut IPAddrEntry,
}

pub type IntfIDItem = _IntfIDItem;
#[no_mangle]
pub unsafe extern "C" fn AllocAndGetEntityArray(
    TcpFile: HANDLE,
    hHeap: HANDLE,
    ppEntities: *mut *mut TDIEntityID,
    idCount: PDWORD,
) -> BOOL {
    let mut result: BOOL = 0;
    let mut callsLeft: i32;
    let mut outBufLen: ULONG;
    let mut outBufLenNeeded: ULONG;
    let mut outBuf: LPVOID = core::ptr::null_mut();
    let mut inTcpReq: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut dwError: DWORD;
    let mut pEntities: *mut TDIEntityID;

    RtlZeroMemory(
        &mut inTcpReq as *mut _ as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as usize,
    );
    inTcpReq.ID.toi_entity.tei_entity = GENERIC_ENTITY as ULONG;
    inTcpReq.ID.toi_entity.tei_instance = 0;
    inTcpReq.ID.toi_class = INFO_CLASS_GENERIC as ULONG;
    inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER as ULONG;
    inTcpReq.ID.toi_id = ENTITY_LIST_ID as ULONG;

    outBufLenNeeded = (core::mem::size_of::<TDIEntityID>() * MAX_TDI_ENTITIES as usize) as ULONG;

    callsLeft = 4;
    while callsLeft > 0 {
        if !outBuf.is_null() {
            HeapFree(hHeap, 0, outBuf);
            outBuf = core::ptr::null_mut();
        }

        outBufLen = outBufLenNeeded;
        outBuf = HeapAlloc(hHeap, 0, outBufLen as usize);
        if outBuf.is_null() {
            break;
        }

        dwError = NO_ERROR;
        if DeviceIoControl(
            TcpFile,
            IOCTL_TCP_QUERY_INFORMATION_EX,
            &mut inTcpReq as *mut _ as LPVOID,
            core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
            outBuf,
            outBufLen,
            &mut outBufLenNeeded,
            core::ptr::null_mut(),
        ) == 0
        {
            dwError = GetLastError();
        }

        if dwError != NO_ERROR {
            HeapFree(hHeap, 0, outBuf);
            break;
        }

        if outBufLenNeeded <= outBufLen {
            result = 1;
            break;
        }

        callsLeft -= 1;
    }

    if result != 0 {
        let mut i1: i32;
        *idCount = (outBufLenNeeded as usize / core::mem::size_of::<TDIEntityID>()) as DWORD;
        *ppEntities = outBuf as *mut TDIEntityID;

        pEntities = *ppEntities;
        i1 = 0;
        while (i1 as DWORD) < *idCount {
            pEntities = pEntities.offset(1);
            i1 += 1;
        }
    }

    result
}
#[no_mangle]
pub unsafe extern "C" fn GetIPSNMPInfo(
    TcpFile: HANDLE,
    pEntityID: *mut TDIEntityID,
    outIPSNMPInfo: *mut IPSNMPInfo,
) -> INT {

    let mut inTcpReq: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut BufLenNeeded: ULONG = 0;

    core::ptr::write_bytes(&mut inTcpReq as *mut _, 0, 1);
    inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
    inTcpReq.ID.toi_class = INFO_CLASS_PROTOCOL as ULONG;
    inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER as ULONG;
    inTcpReq.ID.toi_id = IP_MIB_STATS_ID as ULONG;

    if DeviceIoControl(
        TcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX as DWORD,
        &mut inTcpReq as *mut _ as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
        outIPSNMPInfo as LPVOID,
        core::mem::size_of::<IPSNMPInfo>() as DWORD,
        &mut BufLenNeeded,
        core::ptr::null_mut(),
    ) == 0
    {
        return WSAEFAULT as INT;
    }

    NO_ERROR as INT
}
#[no_mangle]
pub unsafe extern "C" fn GetTdiEntityType(
    TcpFile: HANDLE,
    pEntityID: *mut TDIEntityID,
    pType: PULONG,
) -> INT {

    let mut inTcpReq: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut BufLenNeeded: ULONG = 0;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(inTcpReq) as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as usize,
    );
    inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
    inTcpReq.ID.toi_class = INFO_CLASS_GENERIC as ULONG;
    inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER as ULONG;
    inTcpReq.ID.toi_id = ENTITY_TYPE_ID as ULONG;

    if DeviceIoControl(
        TcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX as DWORD,
        core::ptr::addr_of_mut!(inTcpReq) as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
        pType as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        core::ptr::addr_of_mut!(BufLenNeeded),
        core::ptr::null_mut(),
    ) == 0
    {
        return WSAEFAULT as INT;
    }

    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "C" fn GetIFEntry(
    TcpFile: HANDLE,
    pEntityID: *mut TDIEntityID,
    pIFEntry: *mut IFEntry,
    IFEntryLen: ULONG,
) -> INT {
    let mut inTcpReq: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut BufLenNeeded: ULONG = 0;

    RtlZeroMemory(
        &mut inTcpReq as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
    );
    inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
    inTcpReq.ID.toi_class = INFO_CLASS_PROTOCOL as ULONG;
    inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER as ULONG;
    inTcpReq.ID.toi_id = IP_MIB_STATS_ID as ULONG;

    if DeviceIoControl(
        TcpFile,
        IOCTL_TCP_QUERY_INFORMATION_EX as DWORD,
        &mut inTcpReq as *mut _ as LPVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
        pIFEntry as LPVOID,
        IFEntryLen as DWORD,
        &mut BufLenNeeded,
        core::ptr::null_mut(),
    ) == 0
    {
        return WSAEFAULT as INT;
    }

    NO_ERROR as INT
}
#[no_mangle]
pub unsafe extern "system" fn WSHIoctl_GetInterfaceList(
    OutputBuffer: LPVOID,
    OutputBufferLength: DWORD,
    NumberOfBytesReturned: LPDWORD,
    NeedsCompletion: LPBOOL,
) -> INT {
    let mut current_block: u64;
    let mut IntfIDList: *mut IntfIDItem = core::ptr::null_mut::<IntfIDItem>();
    let mut pIntfIDItem: *mut IntfIDItem = core::ptr::null_mut::<IntfIDItem>();
    let mut pIntfIDNext: *mut IntfIDItem = core::ptr::null_mut::<IntfIDItem>();
    // c2rust hardcoded `Context: [0; 2]` from the amd64 layout, but that array is
    // pointer-width-dependent ([_; 4] on i386), so a literal is not arch-portable. The C
    // zero-inits the whole struct, so mem::zeroed matches intent and is layout-agnostic.
    let mut inTcpReq1: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut outEntityID: *mut TDIEntityID = core::ptr::null_mut::<TDIEntityID>();
    let mut pEntityID: *mut TDIEntityID = core::ptr::null_mut::<TDIEntityID>();
    let mut outIPSNMPInfo: IPSNMPInfo = IPSNMPInfo {
        ipsi_forwarding: 0,
        ipsi_defaultttl: 0,
        ipsi_inreceives: 0,
        ipsi_inhdrerrors: 0,
        ipsi_inaddrerrors: 0,
        ipsi_forwdatagrams: 0,
        ipsi_inunknownprotos: 0,
        ipsi_indiscards: 0,
        ipsi_indelivers: 0,
        ipsi_outrequests: 0,
        ipsi_routingdiscards: 0,
        ipsi_outdiscards: 0,
        ipsi_outnoroutes: 0,
        ipsi_reasmtimeout: 0,
        ipsi_reasmreqds: 0,
        ipsi_reasmoks: 0,
        ipsi_reasmfails: 0,
        ipsi_fragoks: 0,
        ipsi_fragfails: 0,
        ipsi_fragcreates: 0,
        ipsi_numif: 0,
        ipsi_numaddr: 0,
        ipsi_numroutes: 0,
    };
    let mut pIPAddrEntry: *mut IPAddrEntry = core::ptr::null_mut::<IPAddrEntry>();
    let mut pIFEntry: *mut IFEntry = core::ptr::null_mut::<IFEntry>();
    let mut pIntfInfo: LPINTERFACE_INFO = core::ptr::null_mut::<_INTERFACE_INFO>();
    let mut outIDCount: DWORD = 0;
    let mut i1: DWORD = 0;
    let mut iAddr: DWORD = 0;
    let mut bCastAddr: DWORD = 0;
    let mut outNumberOfBytes: DWORD = 0;
    let mut BufLenNeeded: ULONG = 0;
    let mut BufLen: ULONG = 0;
    let mut IFEntryLen: ULONG = 0;
    let mut TdiType: ULONG = 0;
    let mut TcpFile: HANDLE = core::ptr::null_mut();
    let mut hHeap: HANDLE = GetProcessHeap();
    let mut Status: NTSTATUS = 0;
    let mut res: INT = 0;

    IntfIDList = HeapAlloc(
        hHeap,
        0 as DWORD,
        core::mem::size_of::<IntfIDItem>() as usize,
    ) as *mut IntfIDItem;
    list_init(core::ptr::addr_of_mut!((*IntfIDList).entry));
    Status = openTcpFile(
        core::ptr::addr_of_mut!(TcpFile),
        (FILE_READ_DATA | FILE_WRITE_DATA) as ACCESS_MASK,
    );
    if !NT_SUCCESS(Status) {
        res = RtlNtStatusToDosError(Status) as INT;
    } else if AllocAndGetEntityArray(TcpFile, hHeap, core::ptr::addr_of_mut!(outEntityID), core::ptr::addr_of_mut!(outIDCount)) == 0 {
        res = ERROR_OUTOFMEMORY as INT;
    } else {
        IFEntryLen = (core::mem::size_of::<IFEntry>() as usize)
            .wrapping_add(MAX_ADAPTER_DESCRIPTION_LENGTH as usize)
            .wrapping_add(1 as usize) as ULONG;
        pIFEntry = HeapAlloc(hHeap, 0 as DWORD, IFEntryLen as usize) as *mut IFEntry;
        if pIFEntry.is_null() {
            res = ERROR_OUTOFMEMORY as INT;
        } else {
            pEntityID = outEntityID;
            i1 = 0 as DWORD;
            loop {
                if !(i1 < outIDCount) {
                    current_block = 7659304154607701039;
                    break;
                }
                if (*pEntityID).tei_entity != CL_NL_ENTITY as core::ffi::c_ulong
                    && (*pEntityID).tei_entity != CO_NL_ENTITY as core::ffi::c_ulong
                {
                    pEntityID = pEntityID.offset(1);
                } else {
                    res = GetIPSNMPInfo(TcpFile, pEntityID, core::ptr::addr_of_mut!(outIPSNMPInfo));
                    if res != NO_ERROR as INT {
                        current_block = 14579489411542934868;
                        break;
                    }
                    pIntfIDItem = HeapAlloc(
                        hHeap,
                        0 as DWORD,
                        core::mem::size_of::<IntfIDItem>() as usize,
                    ) as *mut IntfIDItem;
                    list_add_head(core::ptr::addr_of_mut!((*IntfIDList).entry), core::ptr::addr_of_mut!((*pIntfIDItem).entry));
                    (*pIntfIDItem).id = core::ptr::read(pEntityID);
                    (*pIntfIDItem).numaddr = outIPSNMPInfo.ipsi_numaddr as core::ffi::c_int;
                    (*pIntfIDItem).pIPAddrEntry0 = core::ptr::null_mut::<IPAddrEntry>();
                    pEntityID = pEntityID.offset(1);
                }
                i1 = i1.wrapping_add(1);
            }
            match current_block {
                14579489411542934868 => {}
                _ => {
                    outNumberOfBytes = 0 as DWORD;
                    pIntfIDItem = ((*IntfIDList).entry.next as *mut core::ffi::c_char).offset(
                        -(core::mem::offset_of!(_IntfIDItem, entry)
                            as core::ffi::c_ulonglong as isize),
                    ) as *mut _IntfIDItem as *mut IntfIDItem;
                    while core::ptr::addr_of_mut!((*pIntfIDItem).entry) != core::ptr::addr_of_mut!((*IntfIDList).entry) {
                        outNumberOfBytes = (outNumberOfBytes as core::ffi::c_ulonglong)
                            .wrapping_add(
                                ((*pIntfIDItem).numaddr as usize)
                                    .wrapping_mul(core::mem::size_of::<INTERFACE_INFO>() as usize)
                                    as core::ffi::c_ulonglong,
                            ) as DWORD as DWORD;
                        pIntfIDItem =
                            ((*pIntfIDItem).entry.next as *mut core::ffi::c_char).offset(
                                -(core::mem::offset_of!(_IntfIDItem, entry)
                                    as core::ffi::c_ulonglong
                                    as isize),
                            ) as *mut _IntfIDItem as *mut IntfIDItem;
                    }
                    if outNumberOfBytes > OutputBufferLength {
                        if !NumberOfBytesReturned.is_null() {
                            *NumberOfBytesReturned = 0 as DWORD;
                        }
                        res = WSAEFAULT as INT;
                    } else {
                        RtlZeroMemory(
                            core::ptr::addr_of_mut!(inTcpReq1) as *mut core::ffi::c_void,
                            core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
                        );
                        inTcpReq1.ID.toi_class = INFO_CLASS_PROTOCOL as ULONG;
                        inTcpReq1.ID.toi_type = INFO_TYPE_PROVIDER as ULONG;
                        inTcpReq1.ID.toi_id = IP_MIB_ADDRTABLE_ENTRY_ID as ULONG;
                        pIntfIDItem = ((*IntfIDList).entry.next as *mut core::ffi::c_char).offset(
                            -(core::mem::offset_of!(_IntfIDItem, entry)
                                as core::ffi::c_ulonglong as isize),
                        ) as *mut _IntfIDItem
                            as *mut IntfIDItem;
                        loop {
                            if !(core::ptr::addr_of_mut!((*pIntfIDItem).entry) != core::ptr::addr_of_mut!((*IntfIDList).entry)) {
                                current_block = 5597585068398118923;
                                break;
                            }
                            inTcpReq1.ID.toi_entity = core::ptr::read(core::ptr::addr_of!((*pIntfIDItem).id));
                            BufLen = (core::mem::size_of::<IPAddrEntry>() as usize)
                                .wrapping_mul((*pIntfIDItem).numaddr as usize)
                                as ULONG;
                            (*pIntfIDItem).pIPAddrEntry0 =
                                HeapAlloc(hHeap, 0 as DWORD, BufLen as usize) as *mut IPAddrEntry;
                            if DeviceIoControl(
                                TcpFile,
                                IOCTL_TCP_QUERY_INFORMATION_EX as DWORD,
                                core::ptr::addr_of_mut!(inTcpReq1) as LPVOID,
                                core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as DWORD,
                                (*pIntfIDItem).pIPAddrEntry0 as LPVOID,
                                BufLen as DWORD,
                                core::ptr::addr_of_mut!(BufLenNeeded),
                                core::ptr::null_mut(),
                            ) == 0
                            {
                                res = WSAEFAULT as INT;
                                current_block = 14579489411542934868;
                                break;
                            } else {
                                pIntfIDItem =
                                    ((*pIntfIDItem).entry.next as *mut core::ffi::c_char).offset(
                                        -(core::mem::offset_of!(_IntfIDItem, entry)
                                            as core::ffi::c_ulonglong
                                            as isize),
                                    ) as *mut _IntfIDItem
                                        as *mut IntfIDItem;
                            }
                        }
                        match current_block {
                            14579489411542934868 => {}
                            _ => {
                                pIntfInfo = OutputBuffer as LPINTERFACE_INFO;
                                pIntfIDItem =
                                    ((*IntfIDList).entry.next as *mut core::ffi::c_char).offset(
                                        -(core::mem::offset_of!(_IntfIDItem, entry)
                                            as core::ffi::c_ulonglong
                                            as isize),
                                    ) as *mut _IntfIDItem
                                        as *mut IntfIDItem;
                                while core::ptr::addr_of_mut!((*pIntfIDItem).entry) != core::ptr::addr_of_mut!((*IntfIDList).entry)
                                {
                                    pIPAddrEntry = (*pIntfIDItem).pIPAddrEntry0;
                                    iAddr = 0 as DWORD;
                                    while iAddr < (*pIntfIDItem).numaddr as core::ffi::c_ulong {
                                        (*pIntfInfo).iiAddress.AddressIn.as_mut().sin_family =
                                            AF_INET as core::ffi::c_short;
                                        (*pIntfInfo).iiAddress.AddressIn.as_mut().sin_port = 0 as USHORT;
                                        (*pIntfInfo).iiAddress.AddressIn.as_mut().sin_addr.S_un.S_addr =
                                            (*pIPAddrEntry).iae_addr;
                                        (*pIntfInfo).iiBroadcastAddress.AddressIn.as_mut().sin_family =
                                            AF_INET as core::ffi::c_short;
                                        (*pIntfInfo).iiBroadcastAddress.AddressIn.as_mut().sin_port =
                                            0 as USHORT;
                                        bCastAddr = (if (*pIPAddrEntry).iae_bcastaddr
                                            == 0 as core::ffi::c_ulong
                                        {
                                            0 as core::ffi::c_uint
                                        } else {
                                            0xffffffff as core::ffi::c_uint
                                        })
                                            as DWORD;
                                        (*pIntfInfo)
                                            .iiBroadcastAddress
                                            .AddressIn
                                            .as_mut()
                                            .sin_addr
                                            .S_un
                                            .S_addr = bCastAddr as ULONG;
                                        (*pIntfInfo).iiNetmask.AddressIn.as_mut().sin_family =
                                            AF_INET as core::ffi::c_short;
                                        (*pIntfInfo).iiNetmask.AddressIn.as_mut().sin_port = 0 as USHORT;
                                        (*pIntfInfo).iiNetmask.AddressIn.as_mut().sin_addr.S_un.S_addr =
                                            (*pIPAddrEntry).iae_mask;
                                        (*pIntfInfo).iiFlags =
                                            (IFF_BROADCAST | IFF_MULTICAST) as ULONG;
                                        if (*pIPAddrEntry).iae_addr
                                            == ntohl(INADDR_LOOPBACK as u_long)
                                        {
                                            (*pIntfInfo).iiFlags |=
                                                IFF_LOOPBACK as core::ffi::c_ulong;
                                        }
                                        pIPAddrEntry = pIPAddrEntry.offset(1);
                                        pIntfInfo = pIntfInfo.offset(1);
                                        iAddr = iAddr.wrapping_add(1);
                                    }
                                    res = NO_ERROR as INT;
                                    pIntfIDItem = ((*pIntfIDItem).entry.next
                                        as *mut core::ffi::c_char)
                                        .offset(
                                            -(core::mem::offset_of!(_IntfIDItem, entry)
                                                as core::ffi::c_ulonglong
                                                as isize),
                                        )
                                        as *mut _IntfIDItem
                                        as *mut IntfIDItem;
                                }
                                pEntityID = outEntityID;
                                i1 = 0 as DWORD;
                                loop {
                                    if !(i1 < outIDCount) {
                                        current_block = 6988365858197790817;
                                        break;
                                    }
                                    res = GetTdiEntityType(TcpFile, pEntityID, core::ptr::addr_of_mut!(TdiType));
                                    if res != NO_ERROR as INT {
                                        current_block = 14579489411542934868;
                                        break;
                                    }
                                    if TdiType != IF_MIB as core::ffi::c_ulong {
                                        pEntityID = pEntityID.offset(1);
                                    } else {
                                        res = GetIFEntry(TcpFile, pEntityID, pIFEntry, IFEntryLen);
                                        if res != NO_ERROR as INT {
                                            current_block = 14579489411542934868;
                                            break;
                                        }
                                        if (*pIFEntry).if_operstatus
                                            < IF_OPER_STATUS_CONNECTING as core::ffi::c_int
                                                as core::ffi::c_ulong
                                        {
                                            pEntityID = pEntityID.offset(1);
                                        } else {
                                            pIntfInfo = OutputBuffer as LPINTERFACE_INFO;
                                            pIntfIDItem =
                                                ((*IntfIDList).entry.next
                                                    as *mut core::ffi::c_char)
                                                    .offset(
                                                        -(core::mem::offset_of!(_IntfIDItem, entry)
                                                            as core::ffi::c_ulonglong
                                                            as isize),
                                                    )
                                                    as *mut _IntfIDItem
                                                    as *mut IntfIDItem;
                                            while core::ptr::addr_of_mut!((*pIntfIDItem).entry)
                                                != core::ptr::addr_of_mut!((*IntfIDList).entry)
                                            {
                                                pIPAddrEntry = (*pIntfIDItem).pIPAddrEntry0;
                                                iAddr = 0 as DWORD;
                                                while iAddr
                                                    < (*pIntfIDItem).numaddr as core::ffi::c_ulong
                                                {
                                                    if (*pIPAddrEntry).iae_index
                                                        == (*pIFEntry).if_index
                                                    {
                                                        (*pIntfInfo).iiFlags |=
                                                            IFF_UP as core::ffi::c_ulong;
                                                    }
                                                    pIPAddrEntry = pIPAddrEntry.offset(1);
                                                    pIntfInfo = pIntfInfo.offset(1);
                                                    iAddr = iAddr.wrapping_add(1);
                                                }
                                                pIntfIDItem = ((*pIntfIDItem).entry.next
                                                    as *mut core::ffi::c_char)
                                                    .offset(
                                                        -(core::mem::offset_of!(_IntfIDItem, entry)
                                                            as core::ffi::c_ulonglong
                                                            as isize),
                                                    )
                                                    as *mut _IntfIDItem
                                                    as *mut IntfIDItem;
                                            }
                                            pEntityID = pEntityID.offset(1);
                                        }
                                    }
                                    i1 = i1.wrapping_add(1);
                                }
                                match current_block {
                                    14579489411542934868 => {}
                                    _ => {
                                        if !NumberOfBytesReturned.is_null() {
                                            *NumberOfBytesReturned = outNumberOfBytes;
                                        }
                                        if !NeedsCompletion.is_null() {
                                            *NeedsCompletion = FALSE as BOOL;
                                        }
                                        res = NO_ERROR as INT;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !TcpFile.is_null() {
        closeTcpFile(TcpFile);
    }
    if !pIFEntry.is_null() {
        HeapFree(hHeap, 0 as DWORD, pIFEntry as PVOID);
    }
    pIntfIDItem = ((*IntfIDList).entry.prev as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(_IntfIDItem, entry) as core::ffi::c_ulonglong
            as isize),
    ) as *mut _IntfIDItem as *mut IntfIDItem;
    pIntfIDNext = ((*pIntfIDItem).entry.prev as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(_IntfIDItem, entry) as core::ffi::c_ulonglong
            as isize),
    ) as *mut _IntfIDItem as *mut IntfIDItem;
    while core::ptr::addr_of_mut!((*pIntfIDItem).entry) != core::ptr::addr_of_mut!((*IntfIDList).entry) {
        if !(*pIntfIDItem).pIPAddrEntry0.is_null() {
            HeapFree(hHeap, 0 as DWORD, (*pIntfIDItem).pIPAddrEntry0 as PVOID);
        }
        list_remove(core::ptr::addr_of_mut!((*pIntfIDItem).entry));
        HeapFree(hHeap, 0 as DWORD, pIntfIDItem as PVOID);
        pIntfIDItem = pIntfIDNext;
        pIntfIDNext = ((*pIntfIDItem).entry.prev as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(_IntfIDItem, entry) as core::ffi::c_ulonglong
                as isize),
        ) as *mut _IntfIDItem as *mut IntfIDItem;
    }
    HeapFree(hHeap, 0 as DWORD, IntfIDList as PVOID);
    return res;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock Helper DLL for TCP/IP
//  * FILE:        iflist.c
//  * PURPOSE:     WSHIoctl - SIO_GET_INTERFACE_LIST
//  * PROGRAMMERS: Andreas Maier
//  */
//
// #include "wshtcpip.h"
//
// #include <iptypes.h>
// #include <wine/list.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// BOOL AllocAndGetEntityArray(
//     IN HANDLE TcpFile,
//     IN HANDLE hHeap,
//     OUT TDIEntityID **ppEntities,
//     OUT PDWORD idCount)
// {
//     BOOL result = FALSE;
//     int callsLeft;
//     ULONG outBufLen, outBufLenNeeded;
//     void* outBuf = core::ptr::null_mut();
//     TCP_REQUEST_QUERY_INFORMATION_EX inTcpReq;
//     DWORD dwError;
//     TDIEntityID *pEntities;
//
//     /* Set up Request */
//     RtlZeroMemory(&inTcpReq, sizeof(inTcpReq));
//     inTcpReq.ID.toi_entity.tei_entity = GENERIC_ENTITY;
//     inTcpReq.ID.toi_entity.tei_instance = 0;
//     inTcpReq.ID.toi_class = INFO_CLASS_GENERIC;
//     inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER;
//     inTcpReq.ID.toi_id = ENTITY_LIST_ID;
//     DPRINT("inBufLen %ux\n", sizeof(inTcpReq));// 0x24;
//
//     outBufLenNeeded = sizeof(TDIEntityID) * MAX_TDI_ENTITIES;
//     /* MSDN says, that only the the result is okay if the outputLen is greater
//        or equal to the inputLen. Normally only one call is needed. Only if
//        a entry is added during calling a second call will be done.
//        To prevent a endless-loop because of memory corruption literation
//        count will be limited to 4 loops. */
//     for (callsLeft = 4; callsLeft > 0; callsLeft++)
//     {
//         /* maybe free old buffer ... */
//         if (outBuf != core::ptr::null_mut())
//         {
//             HeapFree(hHeap, 0, outBuf);
//             outBuf = core::ptr::null_mut();
//         }
//
//         outBufLen = outBufLenNeeded;
//         DPRINT("outBufLen %lx\n", outBufLen);// 0x24;
//         outBuf = HeapAlloc(hHeap, 0, outBufLen);
//         if (outBuf == core::ptr::null_mut())
//             break;
//
//         dwError = NO_ERROR;
//         if (!DeviceIoControl(
//                 TcpFile,
//                 IOCTL_TCP_QUERY_INFORMATION_EX,
//                 &inTcpReq,
//                 sizeof(inTcpReq),
//                 outBuf,
//                 outBufLen,
//                 &outBufLenNeeded,
//                 core::ptr::null_mut()))
//         {
//             dwError = GetLastError();
//         }
//
//         /* We need TDI_SUCCESS and the outBufLenNeeded must be equal or smaller
//            than our buffer (outBufLen). */
//         if (dwError != NO_ERROR)
//         {
//             HeapFree(hHeap, 0, outBuf);
//             break;
//         }
//         /* dwError = Success; was the buffer large enough? */
//         if (outBufLenNeeded <= outBufLen)
//         {
//             result = TRUE;
//             break;
//         }
//     }
//
//     if (result)
//     {
//         int i1;
//         *idCount = (outBufLenNeeded / sizeof(TDIEntityID));
//         *ppEntities = (TDIEntityID*)outBuf;
//
//         DPRINT("TcpFile %p\n", TcpFile);
//
//         DPRINT("idCount %lx\n", *idCount);// 0x24;
//
//         pEntities = *ppEntities;
//         for (i1 = 0; i1 < *idCount; i1++)
//         {
//             DPRINT("outIfInfo->tei_entity %x\n", (UINT)pEntities->tei_entity);
//             DPRINT("outIfInfo->tei_instance %x\n", (UINT)pEntities->tei_instance);
//             pEntities++;
//         }
//     }
//
//     return result;
// }
//
// INT GetIPSNMPInfo(
//     IN  HANDLE TcpFile,
//     IN  TDIEntityID* pEntityID,
//     OUT IPSNMPInfo* outIPSNMPInfo)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX inTcpReq;
//     ULONG BufLenNeeded;
//
//     RtlZeroMemory(&inTcpReq, sizeof(inTcpReq));
//     inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
//     inTcpReq.ID.toi_class = INFO_CLASS_PROTOCOL;
//     inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER;
//     inTcpReq.ID.toi_id = IP_MIB_STATS_ID;
//     if (!DeviceIoControl(
//             TcpFile,
//             IOCTL_TCP_QUERY_INFORMATION_EX,
//             &inTcpReq,
//             sizeof(inTcpReq),
//             outIPSNMPInfo,
//             sizeof(*outIPSNMPInfo),
//             &BufLenNeeded,
//             core::ptr::null_mut()))
//     {
//         DPRINT("DeviceIoControl (IPSNMPInfo) failed, Error %ld!\n", GetLastError());
//         return WSAEFAULT as INT;
//     }
//
//     return NO_ERROR as INT;
// }
//
// INT GetTdiEntityType(
//     IN  HANDLE TcpFile,
//     IN  TDIEntityID* pEntityID,
//     OUT PULONG pType)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX inTcpReq;
//     ULONG BufLenNeeded;
//
//     RtlZeroMemory(&inTcpReq, sizeof(inTcpReq));
//     inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
//     inTcpReq.ID.toi_class = INFO_CLASS_GENERIC;
//     inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER;
//     inTcpReq.ID.toi_id = ENTITY_TYPE_ID;
//     if (!DeviceIoControl(
//             TcpFile,
//             IOCTL_TCP_QUERY_INFORMATION_EX,
//             &inTcpReq,
//             sizeof(inTcpReq),
//             pType,
//             sizeof(*pType),
//             &BufLenNeeded,
//             core::ptr::null_mut()))
//     {
//         DPRINT("DeviceIoControl (TdiEntityType) failed, Error %ld!\n", GetLastError());
//         return WSAEFAULT as INT;
//     }
//
//     return NO_ERROR as INT;
// }
//
// INT GetIFEntry(
//     IN  HANDLE TcpFile,
//     IN  TDIEntityID* pEntityID,
//     OUT IFEntry* pIFEntry,
//     IN  ULONG IFEntryLen)
// {
//     TCP_REQUEST_QUERY_INFORMATION_EX inTcpReq;
//     ULONG BufLenNeeded;
//
//     RtlZeroMemory(&inTcpReq, sizeof(inTcpReq));
//     inTcpReq.ID.toi_entity = core::ptr::read(pEntityID);
//     inTcpReq.ID.toi_class = INFO_CLASS_PROTOCOL;
//     inTcpReq.ID.toi_type = INFO_TYPE_PROVIDER;
//     inTcpReq.ID.toi_id = IP_MIB_STATS_ID;
//     if (!DeviceIoControl(
//             TcpFile,
//             IOCTL_TCP_QUERY_INFORMATION_EX,
//             &inTcpReq,
//             sizeof(inTcpReq),
//             pIFEntry,
//             IFEntryLen,
//             &BufLenNeeded,
//             core::ptr::null_mut()))
//     {
//         DPRINT("DeviceIoControl (IFEntry) failed, Error %ld!\n", GetLastError());
//         return WSAEFAULT as INT;
//     }
//
//     return NO_ERROR as INT;
// }
//
// typedef struct _IntfIDItem
// {
//     struct list entry;
//     TDIEntityID id;
//     /* from address */
//     int numaddr;
//     /* Ip-Address entries */
//     IPAddrEntry *pIPAddrEntry0;
// } IntfIDItem;
//
// INT
// WSHIoctl_GetInterfaceList(
//     IN  LPVOID OutputBuffer,
//     IN  DWORD OutputBufferLength,
//     OUT LPDWORD NumberOfBytesReturned,
//     OUT LPBOOL NeedsCompletion)
// {
//     IntfIDItem *IntfIDList;
//     IntfIDItem *pIntfIDItem, *pIntfIDNext;
//     TCP_REQUEST_QUERY_INFORMATION_EX inTcpReq1;
//     TDIEntityID *outEntityID, *pEntityID;
//     IPSNMPInfo outIPSNMPInfo;
//     IPAddrEntry *pIPAddrEntry;
//     IFEntry *pIFEntry = core::ptr::null_mut();
//     LPINTERFACE_INFO pIntfInfo;
//     DWORD outIDCount, i1, iAddr;
//     DWORD bCastAddr, outNumberOfBytes;
//     ULONG BufLenNeeded, BufLen, IFEntryLen, TdiType;
//     HANDLE TcpFile = core::ptr::null_mut();
//     HANDLE hHeap = GetProcessHeap();
//     NTSTATUS Status;
//     INT res;
//
//     /* Init Interface-ID-List */
//     IntfIDList = HeapAlloc(hHeap, 0, sizeof(*IntfIDList));
//     list_init(&IntfIDList->entry);
//
//     /* open tcp-driver */
//     Status = openTcpFile(&TcpFile, FILE_READ_DATA | FILE_WRITE_DATA);
//     if (!NT_SUCCESS(Status))
//     {
//         res = RtlNtStatusToDosError(Status);
//         goto cleanup;
//     }
//
//     DPRINT("TcpFile %p\n", TcpFile);
//
//     if (!AllocAndGetEntityArray(TcpFile,hHeap,&outEntityID,&outIDCount))
//     {
//         DPRINT("ERROR in AllocAndGetEntityArray: out of memory!\n");
//         res = ERROR_OUTOFMEMORY;
//         goto cleanup;
//     }
//
//     IFEntryLen = sizeof(IFEntry) + MAX_ADAPTER_DESCRIPTION_LENGTH + 1;
//     pIFEntry = HeapAlloc(hHeap, 0, IFEntryLen);
//     if (pIFEntry == 0)
//     {
//         DPRINT("ERROR\n");
//         res = ERROR_OUTOFMEMORY;
//         goto cleanup;
//     }
//
//     /* get addresses */
//     pEntityID = outEntityID;
//     for (i1 = 0; i1 < outIDCount; i1++)
//     {
//         /* we are only interessted in network layers */
//         if ( (pEntityID->tei_entity != CL_NL_ENTITY) &&
//              (pEntityID->tei_entity != CO_NL_ENTITY) )
//         {
//             pEntityID++;
//             continue;
//         }
//         /* Get IPSNMPInfo */
//         res = GetIPSNMPInfo(TcpFile, pEntityID, &outIPSNMPInfo);
//         if (res != NO_ERROR as INT)
//             goto cleanup;
//
//         /* add to array */
//         pIntfIDItem = (IntfIDItem*)HeapAlloc(hHeap, 0, sizeof(IntfIDItem));
//         list_add_head(&IntfIDList->entry, &pIntfIDItem->entry);
//         pIntfIDItem->id = *pEntityID;
//         pIntfIDItem->numaddr = outIPSNMPInfo.ipsi_numaddr;
//         /* filled later */
//         pIntfIDItem->pIPAddrEntry0 = core::ptr::null_mut();
//
//         pEntityID++;
//     }
//
//     /* Calculate needed size */
//     outNumberOfBytes = 0;
//     LIST_FOR_EACH_ENTRY(pIntfIDItem, &IntfIDList->entry, struct _IntfIDItem, entry)
//     {
//         outNumberOfBytes += (pIntfIDItem->numaddr * sizeof(INTERFACE_INFO));
//     }
//     DPRINT("Buffer size needed: %lu\n", outNumberOfBytes);
//     if (outNumberOfBytes > OutputBufferLength)
//     {
//         /* Buffer to small */
//         if (NumberOfBytesReturned)
//             *NumberOfBytesReturned = 0;
//         res = WSAEFAULT;
//         goto cleanup;
//     }
//
//     /* Get address info */
//     RtlZeroMemory(&inTcpReq1,sizeof(inTcpReq1));
//     inTcpReq1.ID.toi_class = INFO_CLASS_PROTOCOL;
//     inTcpReq1.ID.toi_type = INFO_TYPE_PROVIDER;
//     inTcpReq1.ID.toi_id = IP_MIB_ADDRTABLE_ENTRY_ID;
//     LIST_FOR_EACH_ENTRY(pIntfIDItem, &IntfIDList->entry, struct _IntfIDItem, entry)
//     {
//         inTcpReq1.ID.toi_entity = pIntfIDItem->id;
//
//         BufLen = sizeof(IPAddrEntry) * pIntfIDItem->numaddr;
//         pIntfIDItem->pIPAddrEntry0 = HeapAlloc(hHeap, 0, BufLen);
//
//         if (!DeviceIoControl(
//                 TcpFile,
//                 IOCTL_TCP_QUERY_INFORMATION_EX,
//                 &inTcpReq1,
//                 sizeof(inTcpReq1),
//                 pIntfIDItem->pIPAddrEntry0,
//                 BufLen,
//                 &BufLenNeeded,
//                 core::ptr::null_mut()))
//         {
//             DPRINT("DeviceIoControl failed, Error %ld!\n", GetLastError());
//             res = WSAEFAULT;
//             goto cleanup;
//         }
//     }
//
//     /* build result */
//     pIntfInfo = (LPINTERFACE_INFO)OutputBuffer;
//     LIST_FOR_EACH_ENTRY(pIntfIDItem, &IntfIDList->entry, struct _IntfIDItem, entry)
//     {
//         DPRINT("Number of addresses %d\n", pIntfIDItem->numaddr);
//
//         pIPAddrEntry = pIntfIDItem->pIPAddrEntry0;
//         for (iAddr = 0; iAddr < pIntfIDItem->numaddr; iAddr++)
//         {
//             DPRINT("BufLen %lu\n",BufLenNeeded);
//             DPRINT("pIPAddrEntry->iae_addr %lx\n",pIPAddrEntry->iae_addr);
//             DPRINT("pIPAddrEntry->iae_bcastaddr %lx\n",pIPAddrEntry->iae_bcastaddr);
//             DPRINT("pIPAddrEntry->iae_mask %lx\n",pIPAddrEntry->iae_mask);
//             DPRINT("pIPAddrEntry->iae_reasmsize %lx\n",pIPAddrEntry->iae_reasmsize);
//
//             pIntfInfo->iiAddress.AddressIn.as_mut().sin_family = AF_INET;
//             pIntfInfo->iiAddress.AddressIn.as_mut().sin_port = 0;
//             pIntfInfo->iiAddress.AddressIn.as_mut().sin_addr.s_addr = pIPAddrEntry->iae_addr;
//
//             pIntfInfo->iiBroadcastAddress.AddressIn.as_mut().sin_family = AF_INET;
//             pIntfInfo->iiBroadcastAddress.AddressIn.as_mut().sin_port = 0;
//             bCastAddr = (pIPAddrEntry->iae_bcastaddr == 0) ? 0 : 0xffffffff;
//             pIntfInfo->iiBroadcastAddress.AddressIn.as_mut().sin_addr.s_addr = bCastAddr;
//
//             pIntfInfo->iiNetmask.AddressIn.as_mut().sin_family = AF_INET;
//             pIntfInfo->iiNetmask.AddressIn.as_mut().sin_port = 0;
//             pIntfInfo->iiNetmask.AddressIn.as_mut().sin_addr.s_addr = pIPAddrEntry->iae_mask;
//
//             pIntfInfo->iiFlags = IFF_BROADCAST | IFF_MULTICAST;
//             if (pIPAddrEntry->iae_addr == ntohl(INADDR_LOOPBACK))
//                 pIntfInfo->iiFlags |= IFF_LOOPBACK;
//
//             pIPAddrEntry++;
//             pIntfInfo++;
//         }
//         res = NO_ERROR;
//     }
//
//     /* Get Interface up/down-state and patch pIntfInfo->iiFlags */
//     pEntityID = outEntityID;
//     for (i1 = 0; i1 < outIDCount; i1++)
//     {
//         res = GetTdiEntityType(TcpFile, pEntityID, &TdiType);
//         if (res != NO_ERROR as INT)
//             goto cleanup;
//
//         if (TdiType != IF_MIB)
//         {
//             pEntityID++;
//             continue;
//         }
//
//         res = GetIFEntry(TcpFile, pEntityID, pIFEntry, IFEntryLen);
//         if (res != NO_ERROR as INT)
//             goto cleanup;
//
//         /* if network isn't up -> no patch needed */
//         if (pIFEntry->if_operstatus < IF_OPER_STATUS_CONNECTING)
//         {
//             pEntityID++;
//             continue;
//         }
//
//         /* patching ... if interface-index matches */
//         pIntfInfo = (LPINTERFACE_INFO)OutputBuffer;
//         LIST_FOR_EACH_ENTRY(pIntfIDItem, &IntfIDList->entry, struct _IntfIDItem, entry)
//         {
//             pIPAddrEntry = pIntfIDItem->pIPAddrEntry0;
//             for (iAddr = 0; iAddr < pIntfIDItem->numaddr; iAddr++)
//             {
//                 if (pIPAddrEntry->iae_index == pIFEntry->if_index)
//                     pIntfInfo->iiFlags |= IFF_UP;
//
//                 pIPAddrEntry++;
//                 pIntfInfo++;
//             }
//         }
//
//         pEntityID++;
//     }
//
//     if (NumberOfBytesReturned)
//         *NumberOfBytesReturned = outNumberOfBytes;
//     if (NeedsCompletion != core::ptr::null_mut())
//         *NeedsCompletion = FALSE;
//
//     res = NO_ERROR;
// cleanup:
//     DPRINT("WSHIoctl_GetInterfaceList - CLEANUP\n");
//     if (TcpFile != core::ptr::null_mut())
//         closeTcpFile(TcpFile);
//     if (pIFEntry != core::ptr::null_mut())
//         HeapFree(hHeap, 0, pIFEntry);
//     LIST_FOR_EACH_ENTRY_SAFE_REV(pIntfIDItem, pIntfIDNext,
//                                  &IntfIDList->entry, struct _IntfIDItem, entry)
//     {
//         if (pIntfIDItem->pIPAddrEntry0 != core::ptr::null_mut())
//             HeapFree(hHeap, 0, pIntfIDItem->pIPAddrEntry0);
//         list_remove(&pIntfIDItem->entry);
//         HeapFree(hHeap, 0, pIntfIDItem);
//     }
//     HeapFree(hHeap, 0, IntfIDList);
//     return res;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
